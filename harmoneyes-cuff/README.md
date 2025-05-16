# Harmoneyes Cuff

This directory contains the firmware for the Harmoneyes cuff.

## Hardware

The hardware for the Harmoneyes cuff is based around the [Adafruit QT Py RP2040](https://www.adafruit.com/product/4900) microcontroller.

## Flashing

### Firmware
To flash the firmware to the microcontroller hold the boot button as you plug it into your computer (through the on-board USB-C port) then run the following:
```bash
cargo run
```

### Console
The device is configured to expose a serial console when running. This console will output all of the logs from the device.