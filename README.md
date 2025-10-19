**Migrated to [CODEBERG](https://codeberg.org/stronnag/msp_gps_toy)**


















# MSP GPS / INAV example

## Overview

This rust program exercises INAV's  `MSP SET_RAW_GPS` / `MSP2_SENSOR_GPS` and `MSP2_SENSOR_RANGEFINDER` APIs.

## Prerequisites

* A supported FC with a modern INAV (say > 4.0)

## Caveats


## Building

* Clone this repository
* Build the test application

 ```
 make
 ```

This should result in a `target/release/msp_gps_toy` application. See `Makefile` for other targets / cross-compilation etc.

## Usage

```
$ msp_gps_toy [options] device-node
Version: 0.2.0

Options:
    -v, --version       Show version
    -r, --raw-gps       Use MSP_SET_RAW GPS (vice MSP2_SENSOR_GPS)
    -h, --help          print this help menu
```

`device-node` represents a serial device node, matched to an INAV/MSP port (e.g. `/dev/ttyACM0`, `COM17`, `/dev/cuaU0`, `/dev/pts/4` etc. No auto-detection of serial ports is performed. The parameter is mandatory.

## Example

```
$ msp_gps_toy /dev/pts/3
MSP    : 2.5
Name   : BENCHYMCTESTY
F/W    : INAV
Version: 8.0.0
Build  : May  1 2024 06:45:13 (52564d6f)
Board  : SITL
Analog : 0.0 volts,  0 amps
Message: MSP2_SENSOR_GPS
```
You can check the GPS status in any MSP visualisation tool, using another MSP port.
```
                                MSP Test Viewer
             v0.40.0 on Arch Linux 6.8.9-zen1-2-zen x86_64, (rust)

Port    : localhost:5760
MW Vers : ---
Name    : BENCHYMCTESTY
API Vers: 2.5 (MSP v2)
FC      : INAV
FC Vers : 8.0.0
Build   : May  1 2024 06:45:13 (52564d6f)
Board   : SITL
WP Info : 0 of 120, valid false
Uptime  : 148s
Power   : 0.0 volts,  0 amps
GPS     : fix 2, sats 21, 35.760773° 140.378632° 85m, 10m/s 247° hdop 1.00
Arming  : NavUnsafe RCLink (0x40800)
IO Stats: 18990 messages in 9.5s (1998.0/s) (unknown: 0, crc 0)
Debug   : ---
```

You can check the rangefinder in the Configurator / Sensors tab.

## Licence

Whatever approximates to none / public domain in your locale. 0BSD (Zero clause BSD)  if an actual license is required by law.

(c) 2023 Jonathan Hudson
