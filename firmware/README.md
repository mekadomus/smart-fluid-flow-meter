# ESP32

The firmware for `smart-fluid-flow-meter`.

Reads measurements from [YF-S201](http://www.mantech.co.za/datasheets/products/yf-s201_sea.pdf) and publishes them to the back-end.

## Configure

A few settings can be configured to modify the resulting executable. To set these configurations you need to create the configuration file:

```
cp sdkconfig.sample src/sdkconfig.defaults
```

And then update `src/sdkconfig.defaults` as desired

## Update git submodules

```
git submodule update --init --recursive
```

## Compile

To generate a docker image and build the code:

```
make build
```

## Upload to ESP32 dev board

Assumes the device is connected at `/dev/ttyACM0`.

```
make upload
```

## Monitor serial port

Assumes the device is connected at `/dev/ttyACM0`.

```
make serial
```
