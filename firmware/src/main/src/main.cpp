// Standard library
#include <optional>

// Esp-idf
#include <driver/gpio.h>
#include <esp_log.h>

// Project components
#include "backend-service.hpp"
#include "button.hpp"
#include "esp_idf_wifi_manager.hpp"
#include "fluid-meter.hpp"

#define GREEN_LED GPIO_NUM_32
#define YELLOW_LED GPIO_NUM_33
#define RED_LED GPIO_NUM_14
#define RESET_BUTTON GPIO_NUM_18
#define FLOW_SENSOR GPIO_NUM_4

/**
 * Different status the system can be in
 */
enum SystemStatus {
  BOOTING,
  FACTORY_SETTINGS,
  WIFI_CONFIGURED,
  SENDING_REQUEST,
  REQUEST_FAILED,
};

/**
 * Used for logging
 */
const char* TAG = "smart-fluid-flow-meter";

/**
 * Measurements will be posted after this amount of time has passed since last
 * post
 */
const int MS_BETWEEN_POSTS = 1'200'000;  // 20 minutes

/**
 * WiFi configuration
 */
wm_config wifi_config;

/**
 * factory_reset task handle
 */
TaskHandle_t factory_reset_handle;

/**
 * post_measurements task handle
 */
TaskHandle_t post_measurements_handle;

/**
 * status_leds task handle
 */
TaskHandle_t status_leds_handle;

/**
 * Button to factory reset wifi settings
 */
Button reset_button = Button(RESET_BUTTON);

/**
 * Wifi manager
 */
EspIdfWifiManager* wm = nullptr;

/**
 * Backend service
 */
BackendService* bs = nullptr;

/**
 * Fluid meter
 */
FluidMeter* fluid_meter = nullptr;

/**
 * Last time measurements were posted to backend
 */
uint64_t last_post = 0;

/**
 * Max numnber of times to retry a failed request
 */
const uint8_t MAX_RETRIES = 5;

/**
 * Curent status of the system. We use this to decide which LEDs to turn on
 */
SystemStatus current_status = BOOTING;

/**
 * Since every time we call `get_volume`, the meter starts counting from 0, we
 * use this variable to hold measurements that couldn't be sent (probably
 * because of network issues)
 */
float litters_memory = 0;

/**
 * Saves the wifi config and shuts down access point
 */
void save_config(wm_config in) {
  wifi_config = in;
  bs = BackendService::get_instance(in.ssid, in.password);
  wm->shutdown_ap();
  current_status = WIFI_CONFIGURED;
  ESP_LOGI(TAG, "Wifi credentials set in eeprom");
}

/**
 * Monitors reset button and performs hard reset if conditions are met
 */
void factory_reset(void* pvParameters) {
  (void)pvParameters;

  while (true) {
    if (!wifi_config.ssid.empty() && reset_button.is_long_pressed(5000)) {
      ESP_LOGI(TAG, "Resetting meter");
      wm->clear_config();
      wifi_config.ssid = "";
      delete bs;
      bs = nullptr;
      std::optional<wm_config> config_opt = wm->get_config(save_config);
    }
  }
}

/**
 * Reads measurements from fluid meter and posts them to backend on the
 * specified cadence
 */
void post_measurements(void* pvParameters) {
  (void)pvParameters;

  while (true) {
    uint64_t current_millis = esp_timer_get_time() / 1000;
    if ((current_millis - last_post) > MS_BETWEEN_POSTS) {
      last_post = current_millis;
      const float litters = fluid_meter->get_volume() + litters_memory;
      if (bs == nullptr) {
        ESP_LOGE(TAG, "Backend service is nullptr");
      } else {
        uint8_t retry_count = 0;
        bool success = false;
        while (!success && retry_count < MAX_RETRIES) {
          current_status = SENDING_REQUEST;
          int status_code = bs->post_measurement(wifi_config.id, litters);
          switch (status_code) {
            case 200:
              success = true;
              current_status = WIFI_CONFIGURED;
              ESP_LOGI(TAG, "Request succeeded");
              break;
            default:
              retry_count++;
              ESP_LOGE(TAG, "Request failed");
              current_status = REQUEST_FAILED;
          }
        }

        if (success) {
          litters_memory = 0;
        } else {
          litters_memory = litters;
        }
      }
    }
  }
}

/**
 * Turns the correct LEDs based on the status of the system
 */
void status_leds() {
  switch (current_status) {
    case BOOTING:
      gpio_set_level(RED_LED, 0);
      gpio_set_level(GREEN_LED, 0);
      gpio_set_level(YELLOW_LED, 1);
      break;
    case FACTORY_SETTINGS:
      gpio_set_level(RED_LED, 1);
      gpio_set_level(GREEN_LED, 1);
      gpio_set_level(YELLOW_LED, 1);
      break;
    case WIFI_CONFIGURED:
      gpio_set_level(RED_LED, 0);
      gpio_set_level(GREEN_LED, 1);
      gpio_set_level(YELLOW_LED, 0);
      break;
    case SENDING_REQUEST:
      gpio_set_level(RED_LED, 0);
      gpio_set_level(GREEN_LED, 1);
      gpio_set_level(YELLOW_LED, 1);
      break;
    case REQUEST_FAILED:
      gpio_set_level(RED_LED, 1);
      gpio_set_level(GREEN_LED, 0);
      gpio_set_level(YELLOW_LED, 0);
      break;
    default:
      gpio_set_level(RED_LED, 0);
      gpio_set_level(GREEN_LED, 0);
      gpio_set_level(YELLOW_LED, 0);
  }
}

/**
 * Task that runs status_leds continuously
 */
void status_leds_task(void* pvParameters) {
  (void)pvParameters;

  while (true) {
    status_leds();
  }
}

extern "C" void app_main() {
  gpio_reset_pin(GREEN_LED);
  gpio_set_direction(GREEN_LED, GPIO_MODE_OUTPUT);
  gpio_reset_pin(YELLOW_LED);
  gpio_set_direction(YELLOW_LED, GPIO_MODE_OUTPUT);
  gpio_reset_pin(RED_LED);
  gpio_set_direction(RED_LED, GPIO_MODE_OUTPUT);

  status_leds();

  wm = EspIdfWifiManager::get_instance("my-esp32-ssid", "APassword");

  wifi_config.ssid = "";
  std::optional<wm_config> config_opt = wm->get_config(save_config);

  if (config_opt.has_value()) {
    save_config(config_opt.value());
  } else {
    current_status = FACTORY_SETTINGS;
  }

  fluid_meter = FluidMeter::get_instance(FLOW_SENSOR);

  xTaskCreate(factory_reset, "factory_reset", 4096, nullptr, tskIDLE_PRIORITY,
              &factory_reset_handle);
  xTaskCreate(post_measurements, "post_measurements", 4096, nullptr,
              tskIDLE_PRIORITY, &post_measurements_handle);
  xTaskCreate(status_leds_task, "status_leds", 4096, nullptr, tskIDLE_PRIORITY,
              &status_leds_handle);
}
