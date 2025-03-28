#pragma once

#include <driver/gpio.h>

class Button {
 public:
  /**
   * Reads button state in the given pin. Buttons are treated as normally high
   * (i.e. When they are not pressed, their value is 1, when they are pressed,
   * their value is 0)
   *
   * @param pin - Pin where events are going to be read from
   */
  Button(const gpio_num_t pin);

  /**
   * Returns true if the button is currently being pressed
   */
  bool is_pressed();

  /**
   * Returns true if the button has been pressed for at least `millis` amount of
   * miliseconds
   */
  bool is_long_pressed(const int ms);

 private:
  const gpio_num_t button_pin;
  unsigned int pressed_since = 0;
};
