#include "backend-service.hpp"

#include <cstdint>
#include <cstring>

// esp-idf
#include "esp_crt_bundle.h"
#include "esp_event.h"
#include "esp_http_client.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "esp_wifi.h"

// Only exists so it can be overwritten by test
#include "configs.h"

BackendService* BackendService::instance = nullptr;
SemaphoreHandle_t BackendService::IP_SEMPH = NULL;
const char* BackendService::MEASUREMENT_API = "/v1/measurement";
const char* BackendService::TAG = "backend-service";
esp_netif_t* BackendService::wifi_if = NULL;

BackendService::~BackendService() {
  deinit_wifi();
  instance = nullptr;
}

esp_err_t BackendService::http_event_handler(esp_http_client_event_t* evt) {
  static char* output_buffer;
  static int output_len;
  switch (evt->event_id) {
    case HTTP_EVENT_ERROR:
      ESP_LOGI(TAG, "HTTP_EVENT_ERROR");
      break;
    case HTTP_EVENT_ON_CONNECTED:
      ESP_LOGI(TAG, "HTTP_EVENT_ON_CONNECTED");
      break;
    case HTTP_EVENT_HEADER_SENT:
      ESP_LOGI(TAG, "HTTP_EVENT_HEADER_SENT");
      break;
    case HTTP_EVENT_ON_HEADER:
      ESP_LOGI(TAG, "HTTP_EVENT_ON_HEADER, key=%s, value=%s", evt->header_key,
               evt->header_value);
      break;
    case HTTP_EVENT_ON_DATA: {
      ESP_LOGI(TAG, "HTTP_EVENT_ON_DATA, len=%d", evt->data_len);
      int copy_len = 0;
      int content_len = esp_http_client_get_content_length(evt->client);
      if (output_buffer == NULL) {
        // We initialize output_buffer with 0 because it is used by strlen() and
        // similar functions therefore should be null terminated.
        output_buffer = (char*)calloc(content_len + 1, sizeof(char));
        output_len = 0;
        if (output_buffer == NULL) {
          ESP_LOGE(TAG, "Failed to allocate memory for output buffer");
          return ESP_FAIL;
        }
      }
      copy_len = std::min(evt->data_len, (content_len - output_len));
      if (copy_len) {
        memcpy(output_buffer + output_len, evt->data, copy_len);
      }
      output_len += copy_len;
      break;
    }
    case HTTP_EVENT_ON_FINISH:
      ESP_LOGI(TAG, "HTTP_EVENT_ON_FINISH");
      if (output_buffer != NULL) {
        ESP_LOGI(TAG, "%s", output_buffer);
        free(output_buffer);
        output_buffer = NULL;
      }
      output_len = 0;
      break;
    case HTTP_EVENT_DISCONNECTED:
      ESP_LOGI(TAG, "HTTP_EVENT_DISCONNECTED");
      if (output_buffer != NULL) {
        free(output_buffer);
        output_buffer = NULL;
      }
      output_len = 0;
      break;
    case HTTP_EVENT_REDIRECT:
      ESP_LOGI(TAG, "HTTP_EVENT_REDIRECT");
      break;
  }
  return ESP_OK;
}

std::string BackendService::get_ssid() {
  return std::string(ssid);
}

int BackendService::post_measurement(const std::string& deviceId,
                                     const float litters) {
  init_wifi();

  esp_http_client_config_t config = {};
  char url[100];
  sprintf(url, "%s%s", CONFIG_BACKEND_URL, MEASUREMENT_API);
  config.url = url;
  config.method = HTTP_METHOD_POST;
  config.crt_bundle_attach = esp_crt_bundle_attach;
  config.event_handler = BackendService::http_event_handler;

  esp_http_client_handle_t client = esp_http_client_init(&config);
  char post_data[120];
  sprintf(post_data, "{\"device_id\":\"%s\",\"measurement\":\"%f\"}",
          deviceId.c_str(), litters);
  esp_http_client_set_post_field(client, post_data, strlen(post_data));
  esp_http_client_set_header(client, "Content-Type", "application/json");
  esp_err_t err = esp_http_client_perform(client);

  int status_code = -1;
  if (err == ESP_OK) {
    status_code = esp_http_client_get_status_code(client);
    if (status_code != 200) {
      ESP_LOGE(TAG, "Got %d code", status_code);
    }
  } else {
    ESP_LOGE(TAG, "Error with https request: %s", esp_err_to_name(err));
  }

  ESP_ERROR_CHECK(esp_http_client_cleanup(client));
  deinit_wifi();

  return status_code;
}

void BackendService::got_ip_handler(void* arg,
                                    esp_event_base_t event_base,
                                    int32_t event_id,
                                    void* event_data) {
  (void)arg;
  (void)event_base;
  (void)event_id;
  (void)event_data;

  ESP_LOGI(TAG, "Got IP address");
  xSemaphoreGive(IP_SEMPH);
}

void BackendService::wifi_disconnected_handler(void* arg,
                                               esp_event_base_t event_base,
                                               int32_t event_id,
                                               void* event_data) {
  (void)arg;
  (void)event_base;
  (void)event_id;
  (void)event_data;

  ESP_LOGI(TAG, "Wifi disconnected");
  xSemaphoreGive(IP_SEMPH);
}

void BackendService::init_wifi() {
  // Start wifi stack
  esp_netif_init();
  esp_event_loop_create_default();
  wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
  esp_wifi_init(&cfg);
  esp_netif_inherent_config_t esp_netif_config =
      ESP_NETIF_INHERENT_DEFAULT_WIFI_STA();
  wifi_if = esp_netif_create_wifi(WIFI_IF_STA, &esp_netif_config);
  esp_wifi_set_default_wifi_sta_handlers();
  esp_wifi_set_mode(WIFI_MODE_STA);
  esp_wifi_start();

  IP_SEMPH = xSemaphoreCreateBinary();

  wifi_config_t wifi_config = {};
  snprintf((char*)wifi_config.sta.ssid, 32, "%s", ssid);
  snprintf((char*)wifi_config.sta.password, 32, "%s", password);
  ESP_LOGI(TAG, "Connecting to %s...", wifi_config.sta.ssid);
  esp_wifi_set_config(WIFI_IF_STA, &wifi_config);

  ESP_ERROR_CHECK(esp_event_handler_register(
      IP_EVENT, IP_EVENT_STA_GOT_IP, &BackendService::got_ip_handler, NULL));
  ESP_ERROR_CHECK(esp_event_handler_register(
      WIFI_EVENT, WIFI_EVENT_STA_DISCONNECTED,
      &BackendService::wifi_disconnected_handler, NULL));

  esp_wifi_connect();

  xSemaphoreTake(IP_SEMPH, portMAX_DELAY);
}

void BackendService::deinit_wifi() {
  ESP_LOGI(TAG, "Turning off wifi");
  ESP_ERROR_CHECK(esp_wifi_stop());
  ESP_ERROR_CHECK(esp_wifi_deinit());
  ESP_ERROR_CHECK(esp_wifi_clear_default_wifi_driver_and_handlers(wifi_if));
  esp_netif_destroy(wifi_if);
  wifi_if = NULL;
}

BackendService::BackendService(const std::string& ssid_in,
                               const std::string& password_in) {
  std::strcpy(ssid, ssid_in.c_str());
  std::strcpy(password, password_in.c_str());
  ESP_LOGW(TAG, "SSID: %s", ssid_in.c_str());
}

BackendService* BackendService::get_instance(const std::string& ssid,
                                             const std::string& password) {
  if (BackendService::instance == nullptr) {
    BackendService::instance = new BackendService(ssid, password);
  }

  return BackendService::instance;
}
