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

use std::path::{Path, PathBuf};
use std::io::Read;

use crate::clock_delay::Gpio;
use anyhow::Result;

pub(crate) fn get_name (device: &str) -> Result<String> {
	use std::io::BufRead;

	let path   = format!("/sys/class/net/{device}/device/uevent");
	let handle = std::fs::File::open(&path)
	             .map_err(|error| anyhow!("can't open {path}: {error}"))?;

	let reader = std::io::BufReader::new(handle);
	let error  = anyhow!("can't find OF_NAME entry in {path}");

	for line in reader.lines().flatten() {
		let mut tokens = line.split('=');

		if tokens.next() == Some("OF_NAME") {
			return match tokens.next() {
				Some(token) => Ok(String::from(token)),
				None        => Err(error),
			}
		}
	}

	bail!(error)
}

pub(crate) fn find_nodes(gpio: &Gpio) -> Vec<String> {
	let mut paths = Vec::new();

	let base = "/sys/firmware/devicetree/base";

	find_paths(&format!("{base}/soc/{}", gpio.pinctrl), gpio, &mut paths);
	find_paths(&format!("{base}/soc@0/{}", gpio.pinctrl), gpio, &mut paths);

	paths.iter().map(|path| format!("/{}", path.strip_prefix(base).unwrap().display())).collect()
}

fn find_paths<P: AsRef<Path>>(current_dir: P, gpio: &Gpio, result: &mut Vec<PathBuf>) {
	let current_dir  = current_dir.as_ref();
	let current_dir_ = format!("{}", current_dir.display());

	if ! current_dir.is_dir() {
		log::warn!("{current_dir_} is not a directory or does not exist");
		return;
	}

	let read_dir = match std::fs::read_dir(current_dir) {
		Err(error)   => { log::warn!("{error} while opening {current_dir_}"); return }
		Ok(read_dir) => { read_dir }
	};

	for entry in read_dir {
		let entry = match entry {
			Err(error) => { log::warn!("{error} while reading {current_dir_}"); continue }
			Ok(entry)  => { entry }
		};

		let file_type = match entry.file_type() {
			Err(error)    => { log::warn!("{error} while reading {current_dir_}"); continue }
			Ok(file_type) => { file_type }
		};

		if file_type.is_dir() {
			find_paths(&entry.path(), gpio, result);
		} else if file_type.is_file() {
			if entry.file_name() != "pinmux" {
				continue;
			}

			let handle = match std::fs::File::open(entry.path()) {
				Err(error) => { log::warn!("{error} while opening {:?}", entry.path()); continue }
				Ok(handle) => { handle }
			};

			let mut reader = std::io::BufReader::new(handle);
			let mut buffer = [0u8; 4];

			loop {
				match reader.read(&mut buffer) {
					Err(error) => { log::warn!("{error} while reading {:?}", entry.path()); continue }
					Ok(4)      => {
						let value  = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
						let pinmux = PinMux::from(value);

						if pinmux.bank == gpio.bank as u8 - b'A' && pinmux.line == gpio.line {
							let mut path = entry.path();
							path.pop();
							result.push(path);
						}
					}
					Ok(0) => { break }
					Ok(_) => { log::warn!("Unexpected end-of-file while reading {:?}", entry.path()); continue }
				}
			}
		}
	}
}

#[derive(Debug)]
struct PinMux {
	bank:  u8,
	line:  u8,
	_mode: u8,
}

impl From<u32> for PinMux {
	fn from(value: u32) -> Self {
		PinMux {
			_mode: (value & 0xFF)           as u8,
			line:  ((value & 0xF00) >> 8)   as u8,
			bank:  ((value & 0xF000) >> 12) as u8,
		}
	}
}

#[test]
fn from_u32_for_pinmux () {
	let pinmux = PinMux::from(0x0000580b);

	assert_eq!(pinmux.bank,  5);
	assert_eq!(pinmux.line,  8);
	assert_eq!(pinmux._mode, 0xb);
}
