# Smart Fluid Flow Meter

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/mekadomus/smart-fluid-flow-meter/tree/master.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/mekadomus/smart-fluid-flow-meter/tree/master)

Hardware and software for a system that monitors flow of fluids (most likely water) and reports it to a back-end that can emit alerts based on configured preferences.

- [backend](/backend/) - Contains the back end server where measurements will be posted
- [case](/case/) - 3d model of the PCB case
- [firmware](/firmware/) - The firmware code that takes care of reading the sensor and posting measurements to the back-end
- [frontend](/frontend/) - The front end that can be used to manage accounts and devices
- [electronics](/electronics/) - Electronic circuit schematic and pcb

## User documentation

### LED indicators

- **Yellow** - Device is booting
- **Green, yellow and red** - Device set to factory settings and waiting for configuration (See below for instructions to configure)
- **Green** - Device is configured and behaving correctly
- **Green and yellow** - Device is in the middle of sending a request to the back-end
- **Red** - Last request to back-end failed. This could be because configuration is incorrect, or the configured modem is unreachable
- **None** - This should not happen, it means something is very wrong

### Configuration

When the device is in factory settings (Green, yellow and red LEDs on), the firmware will start an access point named `my-esp32-ssid`. Connect to the access point using the password `APassword`.

Once connected, navigate to the url: `sffm.mekadomus.com`. You should get a screen like the following:

![Configure device screen](/docs/assets/config-screen.png)
<config screen>

Fill the data as follows:
- *Wifi Network* - The name of the network the device will connect to
- *Wifi Password* - The password for the given SSID
- *Device key* - The unique identifier for this device. This ID will be validated by the backend, so it must be a valid one

After submitting the form, you should get the following confirmation screen:

![Configuration saved screen](/docs/assets/saved-screen.png)

The device will proceed to disable the access point and will start reading measurements from the sensor and sending them to the backend every  `MS_BETWEEN_POSTS`.

### Factory reset

If you need to re-configure the device, press and hold the device button for 5 seconds. After some time the green, yellow and red LEDs should all turn on.
