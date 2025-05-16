#pragma once

#include <driver/gpio.h>
#include <freertos/FreeRTOS.h>

/**
 * This is a singleton so the Service Interrupt Routine can be a method
 */
class FluidMeter {
 public:
  /**
   * Delete these constructors to make sure there can only be one instance
   */
  FluidMeter(FluidMeter const&) = delete;
  void operator=(FluidMeter const&) = delete;

  /**
   * Reads measurements from given pin
   *
   * @param pin - Pin where measurements are going to be read from
   */
  static FluidMeter* get_instance(const gpio_num_t pin);

  /**
   * Returns the number of litters that have been measured since last run
   */
  float get_volume();

 private:
  const gpio_num_t meter_pin;

  static FluidMeter* instance;

  /**
   * This can be configured with the CONFIG_SENSOR_... configurations
   */
  static float conversion_factor;

  /**
   * It should only be modified from the queue task
   */
  static int total_pulses;

  /**
   * Used for serializing the interrupts and keeping `total_pulses` safe
   */
  static QueueHandle_t queue;

  /**
   * Private constructor. Used internally to create the singleton
   */
  FluidMeter(const gpio_num_t pin);

  /**
   * Fluid meter interrupt callback. Counts the pulses emitted by the meter
   */
  static void count_pulses(void* args);

  /**
   * Consumes data from queue
   */
  static void queue_task(void* params);
};
