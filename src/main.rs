//! # Neotron 340ST BIOS
//!
//! This is the BIOS for the Neotron 340ST. The Neotron 340ST is a retro-style
//! home computer based upon the STM32F7-DISCOVERY board, which is powered by
//! the STM32F746G SoC and its 216 MHz Cortex-M7 processor core. It has 340
//! KiB of internal SRAM (hence the name) and 1024 KiB of Flash.
//!
//! ## Basic Operation
//!
//! We initialise the bare-minimum of hardware required to provide a console,
//! and then jump to the Operating System. We currently assume the Operating
//! System is located at address 0x0008_0000 (giving 512 KiB for the BIOS and
//! 512 KiB for the OS). The BIOS takes the top 64 KiB of RAM for stack and
//! drivers. The OS is given the bottom 256 KiB of RAM to share between itself
//! and any applications it loads.
//!
//! ## Hardware
//!
//! * STM32F746G System-on-Chip
//!     * Cortex-M7 @ 216 MHz
//!     * 340 KiB SRAM
//!     * 1024 KiB Flash ROM
//! * 8 MiB SDRAM
//! * 8 MiB NOR Flash
//! * Stereo audio output
//! * SD Card connector
//! * 480 x 272 resolution 16-bit colour LCD
//! * USB Serial interface
//!
//! ## License
//!
//!     Neotron-340ST-BIOS Copyright (c) The Neotron Developers, 2020
//!
//!     This program is free software: you can redistribute it and/or modify
//!     it under the terms of the GNU General Public License as published by
//!     the Free Software Foundation, either version 3 of the License, or
//!     (at your option) any later version.
//!
//!     This program is distributed in the hope that it will be useful,
//!     but WITHOUT ANY WARRANTY; without even the implied warranty of
//!     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//!     GNU General Public License for more details.
//!
//!     You should have received a copy of the GNU General Public License
//!     along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![no_main]
#![no_std]
#![deny(missing_docs)]

// ===========================================================================
// Sub-Modules
// ===========================================================================

// None

// ===========================================================================
// Imports
// ===========================================================================

use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};
use cortex_m_rt::entry;
use hal::{
	device,
	prelude::*,
	serial::{self, Serial},
};
use nb::block;
use neotron_common_bios as common;
use stm32f7xx_hal as hal;

// ===========================================================================
// Types
// ===========================================================================

type AF7 = hal::gpio::Alternate<hal::gpio::AF7>;

/// This holds our system state - all our HAL drivers, etc.
#[allow(dead_code)]
pub struct BoardInner {
	/// USB Virtual COM-Port. Connect the USB mini-B connector to your PC to view.
	usb_uart: hal::serial::Serial<
		device::USART1,
		(hal::gpio::gpioa::PA9<AF7>, hal::gpio::gpiob::PB7<AF7>),
	>,
}

// ===========================================================================
// Static Variables and Constants
// ===========================================================================

/// Records the number of seconds that have elapsed since the epoch (2000-01-01T00:00:00Z).
static SECONDS_SINCE_EPOCH: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Records the number of frames that have elapsed since second last rolled over.
static FRAMES_SINCE_SECOND: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

/// The BIOS version string
static BIOS_VERSION: &str = concat!(
	"Neotron 340ST BIOS, version ",
	env!("CARGO_PKG_VERSION"),
	"\0"
);

/// The table of API calls we provide the OS
static API_CALLS: common::Api = common::Api {
	api_version_get,
	bios_version_get,
	serial_configure,
	serial_get_info,
	serial_write,
	time_get,
	time_set,
};

/// Holds the global state for the motherboard
static GLOBAL_BOARD: spin::Mutex<Option<BoardInner>> = spin::Mutex::new(None);

// ===========================================================================
// Public Functions
// ===========================================================================

impl core::fmt::Write for BoardInner {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for b in s.bytes() {
			block!(self.usb_uart.write(b)).unwrap();
		}
		Ok(())
	}
}

/// Entry point to the BIOS. This is called from the reset vector by
/// `cortex-m-rt`.
#[entry]
fn main() -> ! {
	// Grab the singletons
	let p = device::Peripherals::take().unwrap();
	// Reset and Clock Controller
	let rcc = p.RCC.constrain();
	// Full speed ahead!
	let clocks = rcc.cfgr.sysclk(216.mhz()).freeze();
	// Get the GPIO objects
	let gpioa = p.GPIOA.split();
	let gpiob = p.GPIOB.split();
	// VCP UART is on PB7 (VCP RX) and PA9 (VCP TX).
	let tx = gpioa.pa9.into_alternate_af7();
	let rx = gpiob.pb7.into_alternate_af7();
	// Construct a serial port
	let usb_uart = Serial::new(
		p.USART1,
		(tx, rx),
		clocks,
		serial::Config {
			baud_rate: 115_200.bps(),
			oversampling: serial::Oversampling::By16,
		},
	);

	let mut board = BoardInner { usb_uart };

	// Say hello to the nice users.
	writeln!(
		board,
		"{} booting...",
		&BIOS_VERSION[..BIOS_VERSION.len() - 1]
	)
	.unwrap();

	*GLOBAL_BOARD.lock() = Some(board);

	let code: &common::OsStartFn = unsafe { ::core::mem::transmute(0x0808_0000) };

	code(&API_CALLS);
}

/// Get the API version this crate implements
pub extern "C" fn api_version_get() -> u32 {
	common::API_VERSION
}

/// Get this BIOS version as a string.
pub extern "C" fn bios_version_get() -> common::ApiString<'static> {
	BIOS_VERSION.into()
}

/// Re-configure the UART. We default to 115200/8N1 on UART1, and the other
/// UARTs default to disabled.
pub extern "C" fn serial_configure(
	device: u8,
	_serial_config: common::serial::Config,
) -> common::Result<()> {
	match device {
		_ => common::Result::Err(common::Error::InvalidDevice),
	}
}

/// Get infomation about the UARTs available in ths system.
///
/// We have four UARTs, but we only expose three of them. The keyboard/mouse
/// interface UART is kept internal to the BIOS.
pub extern "C" fn serial_get_info(device: u8) -> common::Option<common::serial::DeviceInfo> {
	match device {
		_ => common::Option::None,
	}
}

/// Write some text to a UART.
pub extern "C" fn serial_write(
	device: u8,
	data: common::ApiByteSlice,
	_timeout: common::Option<common::Timeout>,
) -> common::Result<usize> {
	if let Some(ref mut board) = *crate::GLOBAL_BOARD.lock() {
		// TODO: Add a timer to the board and use it to handle the timeout.
		// Match on the result of write:
		// * if we get an error, return it.
		// * if we get a WouldBlock, spin (or WFI?).
		// * if we get Ok, carry on.
		let data = data.as_slice();
		match device {
			0 => {
				for b in data.iter().cloned() {
					block!(board.usb_uart.write(b)).unwrap();
				}
			}
			_ => {
				return common::Result::Err(common::Error::InvalidDevice);
			}
		}
		common::Result::Ok(data.len())
	} else {
		panic!("HW Lock fail");
	}
}

/// Get the current wall time.
pub extern "C" fn time_get() -> common::Time {
	let (seconds_since_epoch, frames_since_second) = loop {
		let seconds_since_epoch = SECONDS_SINCE_EPOCH.load(core::sync::atomic::Ordering::Acquire);
		// There is a risk that the second will roll over while we do the read.
		let frames_since_second = FRAMES_SINCE_SECOND.load(core::sync::atomic::Ordering::Acquire);
		// So we read the second value twice.
		let seconds_since_epoch2 = SECONDS_SINCE_EPOCH.load(core::sync::atomic::Ordering::Acquire);
		// And if it's the same, we're all good.
		if seconds_since_epoch2 == seconds_since_epoch {
			break (seconds_since_epoch, frames_since_second);
		}
	};
	common::Time {
		seconds_since_epoch,
		frames_since_second,
	}
}

/// Set the current wall time.
pub extern "C" fn time_set(new_time: common::Time) {
	// This should stop us rolling over for a second
	FRAMES_SINCE_SECOND.store(0, core::sync::atomic::Ordering::Release);
	// Now it should be safe to update the time
	SECONDS_SINCE_EPOCH.store(
		new_time.seconds_since_epoch,
		core::sync::atomic::Ordering::Release,
	);
	FRAMES_SINCE_SECOND.store(
		new_time.frames_since_second,
		core::sync::atomic::Ordering::Release,
	);
	// todo: Write the new time to the RTC (which is only accurate to the second)
}

// ===========================================================================
// Private Functions
// ===========================================================================

/// This function is called whenever the BIOS crashes.
#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	// TODO: Print the crash info to the console
	loop {
		atomic::compiler_fence(Ordering::SeqCst);
	}
}

// ===========================================================================
// End Of File
// ===========================================================================
