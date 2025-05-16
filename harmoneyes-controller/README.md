# Harmoneyes Controller

This directory contains the firmware for the Harmoneyes controller.

## Hardware

The hardware for the Harmoneyes controller is based around the [Adafruit Feather nRF52840 Express](https://www.adafruit.com/product/4062) microcontroller.

## Flashing

### Erasing
To erase the microcontroller (to reinstall the softdevice or troubleshoot errors) run the following:
```bash
probe-rs erase --chip nrf52840_xxAA
```

### Softdevice
To flash the softdevice to a blank microcontroller run the following:
```bash
probe-rs download --verify --binary-format hex --chip nRF52840_xxAA softdevice/softdevice-s140-v7.3.0.hex
```

### Firmware
To flash the firmware to the microcontroller run the following:
```bash
cargo run
```