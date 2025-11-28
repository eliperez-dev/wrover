#!/bin/bash

if [ "$1" = "s3" ]; then
  cp sdkconfig.defaults.s3 sdkconfig.defaults
  sed -i 's/xtensa-esp32-espidf/xtensa-esp32s3-espidf/g' .cargo/config.toml
  sed -i 's/MCU="esp32"/MCU="esp32s3"/g' .cargo/config.toml
  echo "Switched to ESP32-S3"
elif [ "$1" = "wrover" ]; then
  cp sdkconfig.defaults.wrover sdkconfig.defaults
  sed -i 's/xtensa-esp32s3-espidf/xtensa-esp32-espidf/g' .cargo/config.toml
  sed -i 's/MCU="esp32s3"/MCU="esp32"/g' .cargo/config.toml
  echo "Switched to ESP32 (WROVER)"
else
  echo "Usage: $0 [s3|wrover]"
fi
