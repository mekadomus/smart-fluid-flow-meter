#include "fluid-meter.hpp"

#include <driver/gpio.h>

int FluidMeter::total_pulses = 0;
FluidMeter* FluidMeter::instance = nullptr;
QueueHandle_t FluidMeter::queue;

FluidMeter::FluidMeter(const gpio_num_t pin)
    : meter_pin{pin}, conversion_factor{690} {
  queue = xQueueCreate(10, sizeof(char));
  xTaskCreate(queue_task, "queue_task", 2048, NULL, 10, NULL);

  gpio_config_t interrupt_config = {
      .pin_bit_mask = 1ULL << meter_pin,
      .mode = GPIO_MODE_INPUT,
      .pull_up_en = GPIO_PULLUP_DISABLE,
      .pull_down_en = GPIO_PULLDOWN_DISABLE,
      .intr_type = GPIO_INTR_POSEDGE,
  };
  gpio_config(&interrupt_config);

  gpio_install_isr_service(0);
  gpio_isr_handler_add(meter_pin, this->count_pulses, nullptr);
}

FluidMeter* FluidMeter::get_instance(const gpio_num_t pin) {
  if (FluidMeter::instance == nullptr) {
    FluidMeter::instance = new FluidMeter(pin);
  }

  return FluidMeter::instance;
}

void FluidMeter::queue_task(void* params) {
  char c;
  while (true) {
    if (xQueueReceive(queue, &c, portMAX_DELAY)) {
      FluidMeter::total_pulses++;
    }
  }
}

void FluidMeter::count_pulses(void* args) {
  char c = 1;
  xQueueSendFromISR(queue, &c, nullptr);
}

float FluidMeter::get_volume() {
  int pulses;

  // Protect total_pulses by disabling interrupts
  portDISABLE_INTERRUPTS();
  pulses = this->total_pulses;
  this->total_pulses = 0;
  portENABLE_INTERRUPTS();

  return pulses / conversion_factor;
}
