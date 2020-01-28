# Neotron 340ST BIOS

![Build Status](https://github.com/neotron-compute/Neotron-340ST-BIOS/workflows/Build/badge.svg "Github Action Build Status")

![Format Status](https://github.com/neotron-compute/Neotron-340ST-BIOS/workflows/Format/badge.svg "Github Action Format Check Status")

This is the BIOS for the Neotron 340ST. It implements the Neotron BIOS API (a hardware-abstraction layer used by the Neotron OS).

## Hardware

The Neotron 340ST is a sibling version of the [Neotron 32](https://github.com/neotron-compute/Neotron-32-BIOS). It has been modified to run on the powerful STM32 F7 Discovery board.

## Status

This BIOS is a work in progress. Bits of the Neotron 32 firmware will be ported over one at a time. The todo list is:

* [ ] Get it booting
* [ ] USB Serial UART (blocking)
* [ ] Time Keeping
* [ ] USB Serial UART (with timeouts)
* [ ] SD Card
* [ ] Graphics Mode on the 480×272 LCD
* [ ] Text Mode on the 480×272 LCD
* [ ] Support for an external VGA monitor
* [ ] Ethernet
* [ ] Audio Synthesiser
* [ ] USB Host Support

## Memory Map

The Neotron 340ST has 1024 KiB of Flash and 340 KiB of internal SRAM. There is also a 16 MiB external NOR Flash, which is unused at the moment, and an external 8 MiB SDRAM which is used as VRAM. The Flash layout is:

* First 512 KiB of Flash for the BIOS
* Next 256 KiB of Flash for the OS
* Next 256 KiB of Flash for the Shell

The RAM layout is flexible - the BIOS takes as much as it needs, then passes the OS the definitions of how much RAM is available and where it is located. The OS then dynamically allocates almost everything it needs from that. The BIOS is also responsible for configuring the stack, and moving the interrupt vector table to RAM.

## Changelog

### Unreleased Changes ([Source](https://github.com/neotron-compute/Neotron-340ST-BIOS/tree/master))

* Literally nothing

## Licence

    Neotron-340ST-BIOS Copyright (c) The Neotron Developers, 2020

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you shall be licensed as above, without
any additional terms or conditions.
