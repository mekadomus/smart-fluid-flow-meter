#!/bin/bash

work_directory=$(pwd)

. /esp-idf/export.sh
idf.py set-target esp32
mkdir -p /esp32/build
cd /esp32/build
cmake ..
cd $work_directory

exec "$@"
