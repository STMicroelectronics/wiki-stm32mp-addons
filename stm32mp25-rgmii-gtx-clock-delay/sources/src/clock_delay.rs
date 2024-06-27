// Copyright 2023 STMicroelectronics
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the
//    distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use anyhow::{Context, Result};

pub(crate) fn access (device: &str, clock_delay: Option<f32>, verbose: bool) -> Result<()> {
	let dt_name = crate::device_tree::get_name(device)?;
	let gpio    = get_gpio(&dt_name)?;
	let address = get_address(&gpio)?;
	let mut value = Value::mmap(&address)?;

	if let Some(clock_delay) = clock_delay {
		value.set(clock_delay)?;
	}

	if verbose {
		println!("device named \"{device}\" is known as \"{dt_name}\" in device-tree");
		println!("↳ its RGMII GTX clock is connected to GPIO {gpio}");
		println!("  ↳ its delay can be accessed at address {address} in /dev/mem");
		println!("    ↳ its value is {:#x} ({} nanoseconds)", value.get()?, value.get_as_ns()?);
	}

	Ok(())
}

pub(crate) fn get_gpio (dt_name: &str) -> Result<Gpio> {
	use std::io::BufRead;

	let path    = "/sys/kernel/debug/pinctrl/";
	let entries = std::fs::read_dir(path)
	              .map_err(|error| anyhow!("can't read directory {path}: {error}"))?;

	let message = "can't find the GPIO connected to the RGMII GTX clock";

	for entry in entries {
		let entry = match entry {
			Err(error) => { log::warn!("{error} while reading {path}"); continue }
			Ok(entry)  => { entry }
		};

		let file_name = entry.file_name();
		let file_name = file_name.to_string_lossy();

		let mut tokens = file_name.split('@');

		let prefix_is_compatible = match tokens.next() {
			Some("soc:pinctrl") => true,
			Some("soc")         => tokens.next() == Some("0:pinctrl"),
			_                   => false,
		};

		if ! prefix_is_compatible {
			continue;
		}

		let pinctrl = format!("pinctrl@{}", tokens.next().unwrap_or("???"));

		let mut path = entry.path();
		path.push("pinconf-pins");

		let handle = std::fs::File::open(path).context("can't open {path}")?;
		let reader = std::io::BufReader::new(handle);
		let needle = format!("{}_RGMII_GTX_CLK", dt_name.to_uppercase());

		for line in reader.lines().flatten() {
			if ! line.contains(&needle) {
				continue;
			}

			let mut tokens = line.split('(');
			let     tokens = tokens.nth(1).ok_or(anyhow!(message))?;
			let mut tokens = tokens.split(')');
			let     token  = tokens.next().ok_or(anyhow!(message))?;
			let mut tokens = token.chars();

			let magic = tokens.next();
			let bank  = tokens.next();
			let line  = tokens.as_str().parse::<u8>().ok();

			if magic != Some('P') || bank.is_none() || line.is_none() {
				continue;
			}

			return Ok(Gpio {
				bank: bank.unwrap(),
				line: line.unwrap(),
				pinctrl,
			});
		}
	}

	bail!(message)
}

fn get_address (gpio: &Gpio) -> Result<Address> {
	let path = format!("/sys/firmware/devicetree/base/__symbols__/gpio{}", gpio.bank.to_lowercase());
	let path = std::fs::read_to_string(path.clone()).map_err(|error| anyhow!("can't read {path}: {error}"))?;
	let path = path.trim_end_matches('\0');

	let anyhow = anyhow!("can't find the address of the GPIO connected to the RGMII GTX clock");

	match path.split('@').last() {
		None          => Err(anyhow),
		Some(address) => {
			match usize::from_str_radix(address, 16) {
				Ok(address) => Ok(Address { base: address + 0x40, offset: gpio.line * 4 }),
				Err(_)      => Err(anyhow),
			}
		}
	}
}

#[derive(Debug)]
struct Value {
	address: *mut u32,
	offset:  u8,
	mmap_addr: *mut libc::c_void,
	mmap_len:  usize,
}

impl Value {
	pub fn mmap (address: &Address) -> Result<Self> {
		assert!(address.offset <= 28); // Not yet supported.

		use nix::unistd::{sysconf, SysconfVar};
		use nix::sys::mman::{mmap, ProtFlags, MapFlags};
		use std::os::unix::io::AsRawFd;

		let handle = std::fs::OpenOptions::new().read(true).write(true).open("/dev/mem")
		             .map_err(|error| anyhow!("can't open /dev/mem: {error}"))?;

		let prot_flags  = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE;
		let page_size   = sysconf(SysconfVar::PAGE_SIZE)?.unwrap_or(4096) as usize;
		let length      = std::num::NonZeroUsize::new(page_size).unwrap();
		let page_base   = (address.base & !(page_size - 1)) as libc::off_t;
		let page_offset = address.base & (page_size - 1);
		let offset      = address.offset;

		let address = unsafe {
			mmap(None, length, prot_flags, MapFlags::MAP_SHARED, handle.as_raw_fd(), page_base)
			.map_err(|error| anyhow!("can't mmap page {page_base:0x}: {error}"))?
		};

		log::debug!("mmaped address = {address:?}");

		Ok(Value {
			address: unsafe { address.add(page_offset) } as *mut u32,
			offset,
			mmap_addr: address,
			mmap_len:  usize::from(length),
		})
	}

	pub fn get_as_ns (&self) -> Result<f32> {
		match self.get()? {
			0          => Ok(0.0),
			1          => Ok(0.3),
			x @ 2..=12 => Ok(0.25 * x as f32),
			13..=16    => Ok(3.25),
			x          => bail!("invalid RGMII clock/data delay: {x:0x}")
		}
	}

	pub fn get (&self) -> Result<u32> {
		let value = unsafe { *self.address };
		Ok((value >> self.offset) & 0xF)
	}

	pub fn set (&mut self, clock_delay: f32) -> Result<()> {
		let bits  = convert_to_bits(clock_delay)?;
		let value = unsafe { *self.address };
		let value = (value & !(0xF << self.offset)) | (bits << self.offset);

		unsafe { *self.address = value }

		Ok(())
	}
}

impl Drop for Value {
	fn drop (&mut self) {
		unsafe { nix::sys::mman::munmap(self.mmap_addr, self.mmap_len).unwrap() }
	}
}

pub(crate) fn convert_to_bits(ns: f32) -> Result<u32> {
	// floating point literals not allowed anymore in patterns:
	// https://github.com/rust-lang/rust/issues/41620b
	if ns == 0.3 {
		Ok(1)
	} else if ns >= 0.0
	       && ns != 0.25
	       && ns <= 3.25
               && ns % 0.25 == 0.0 {
		Ok((ns * 4.0) as u32)
	} else {
		bail!("invalid RGMII clock/data delay: {ns} (in nanoseconds)")
	}
}

#[test]
fn test_convert_bits () {
	assert_eq!(convert_to_bits(0.0).unwrap(),   0);
	assert_eq!(convert_to_bits(0.3).unwrap(),   1);
	assert_eq!(convert_to_bits(0.5).unwrap(),   2);
	assert_eq!(convert_to_bits(0.75).unwrap(),  3);
	assert_eq!(convert_to_bits(1.0).unwrap(),   4);
	assert_eq!(convert_to_bits(1.25).unwrap(),  5);
	assert_eq!(convert_to_bits(1.5).unwrap(),   6);
	assert_eq!(convert_to_bits(1.75).unwrap(),  7);
	assert_eq!(convert_to_bits(2.0).unwrap(),   8);
	assert_eq!(convert_to_bits(2.25).unwrap(),  9);
	assert_eq!(convert_to_bits(2.5).unwrap(),  10);
	assert_eq!(convert_to_bits(2.75).unwrap(), 11);
	assert_eq!(convert_to_bits(3.0).unwrap(),  12);
	assert_eq!(convert_to_bits(3.25).unwrap(), 13);
	assert!(convert_to_bits(1.2).is_err());
	assert!(convert_to_bits(0.25).is_err());
}

lazy_static! {
	pub(crate) static ref VALID_VALUES: Vec<f32> = {
		let mut valid_values = vec![0 as f32, 0.3];
		valid_values.append(&mut (2..=13).map(|x| x as f32 * 0.25).collect());
		valid_values
	};
}

pub(crate) fn parser (value: &str) -> Result<f32> {
	match value.parse::<f32>() {
		Err(error) => bail!("not a floating point value ({error})"),
		Ok(value)  => {
			if VALID_VALUES.contains(&value) {
				Ok(value)
			} else {
				bail!("must be one of {:?}", *VALID_VALUES)
			}
		}
	}
}

#[derive(Debug)]
pub(crate) struct Gpio {
	pub bank: char,
	pub line: u8,
	pub pinctrl: String,
}

#[derive(Debug)]
struct Address {
	base:   usize,
	offset: u8,
}

impl std::fmt::Display for Gpio {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(formatter, "{}{} ({})", self.bank, self.line, self.pinctrl)
	}
}

impl std::fmt::Display for Address {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(formatter, "{:#x} (bits {}-{})", self.base, self.offset, self.offset + 4)
	}
}
