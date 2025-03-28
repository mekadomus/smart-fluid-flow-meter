#include "button.hpp"

#include <driver/gpio.h>
#include <esp_timer.h>

Button::Button(const gpio_num_t pin) : button_pin{pin} {
  gpio_reset_pin(pin);
  gpio_set_direction(pin, GPIO_MODE_INPUT);
  gpio_set_pull_mode(pin, GPIO_PULLUP_ONLY);
}

bool Button::is_pressed() {
  return !gpio_get_level(button_pin);
}

bool Button::is_long_pressed(const int ms) {
  if (is_pressed()) {
    uint64_t current_millis = esp_timer_get_time() / 1000;
    if (pressed_since == 0) {
      pressed_since = current_millis;
    }

    return (current_millis - pressed_since) > ms;
  } else {
    pressed_since = 0;
    return false;
  }
}
