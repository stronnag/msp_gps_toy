# MSP GPS / INAV example

## Overview

This rust program exercises INAV's  `MSP SET_RAW_GPS` and  `MSP2_SENSOR_GPS` API.

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
Version: 0.1.0

Options:
    -v, --version       Show version
    -s, --sensor        Use MSP2_SENSOR_GPS (vice MSP_SET_RAW GPS)
    -h, --help          print this help menu
```

`device-node` represents a serial device node, matched to an INAV/MSP port (e.g. `/dev/ttyACM0`, `COM17`, `/dev/cuaU0`. No auto-detection of serial ports is performed. The parameter is mandatory.


## Example

```
$ msp_gps_toy -s /dev/ttyACM0
MSP    : 2.5
Name   : BENCHYMCTESTY
F/W    : INAV
Version: 7.0.0
Build  : Jun  3 2023 17:59:16 (a6f3b768)
Board  : WINGFC
Analog : 11.9 volts,  0 amps
Message: MSP2_SENSOR_GPS

```
You can check the GPS status in any MSP visualisation tool, using another MSP port.
```
                                          MSP Test Viewer
                           v0.32.0 on Arch Linux 6.3.5-zen1-1-zen x86_64

Port    : /dev/rfcomm3:115200
MW Vers : ---
Name    : BENCHYMCTESTY
API Vers: 2.5 (MSP v2)
FC      : INAV
FC Vers : 7.0.0
Build   : Jun  3 2023 17:59:16 (a6f3b768)
Board   : WINGFC
WP Info : 0 of 120, valid false
Uptime  : 136s
Power   : 11.8 volts,  0 amps
GPS     : fix 2, sats 20, 35.760216° 140.379013° 15m, 10m/s 247° hdop 1.00
Arming  : NavUnsafe RCLink (0x40800)
IO Stats: 414 messages in 7.7s (53.9/s) (unknown: 1, crc 0)
```

## Licence

Whatever approximates to none / public domain in your locale. 0BSD (Zero clause BSD)  if an actual license is required by law.

(c) 2023 Jonathan Hudson
