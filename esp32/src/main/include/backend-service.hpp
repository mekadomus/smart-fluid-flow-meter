#pragma once

// Standard library
#include <cstdint>
#include <string>

// esp-idf
#include "esp_event.h"
#include "esp_http_client.h"
#include "esp_netif.h"
#include "freertos/FreeRTOS.h"

class BackendService {
 public:
  /**
   * Delete these constructors to make sure there can only be one instance
   */
  BackendService(BackendService const&) = delete;
  void operator=(BackendService const&) = delete;

  ~BackendService();

  /**
   * @param ssid - SSID of the network it will connect to
   * @param password - passwrod of the network it will connect to
   */
  static BackendService* get_instance(const std::string& ssid,
                                      const std::string& password);

  /**
   * Starts the whole wifi stack and posts the request. After the request is
   * done, it tears down the wifi stack to save battery.
   *
   * Returns the status code for the response. If there was an error sending the
   * request, it returns -1
   */
  int post_measurement(const std::string& deviceId, const float litters);

  /**
   * Used for tests
   */
  std::string get_ssid();

 private:
  static const char* MEASUREMENT_API;
  static const char* TAG;

  static BackendService* instance;
  static SemaphoreHandle_t IP_SEMPH;
  static esp_netif_t* wifi_if;

  char ssid[32];
  char password[32];

  BackendService(const std::string& ssid, const std::string& password);

  /**
   * Initializes wifi stack and connects to wifi AP. Without this other methods
   * will fail to send requests
   */
  void init_wifi();

  /**
   * Deinitializes wifi stack to save battery
   */
  void deinit_wifi();

  /**
   * Will be triggered after we get an IP from access point
   */
  static void got_ip_handler(void* arg,
                             esp_event_base_t event_base,
                             int32_t event_id,
                             void* event_data);

  /**
   * Will be triggered if the wifi gets disconnected
   */
  static void wifi_disconnected_handler(void* arg,
                                        esp_event_base_t event_base,
                                        int32_t event_id,
                                        void* event_data);

  /**
   * Handles all HTTP events received after a request
   */
  static esp_err_t http_event_handler(esp_http_client_event_t* evt);
};
