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

use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag};
use anyhow::Result;

pub(crate) fn get_nic_stats(device: &str) -> Result<std::collections::HashMap<String, u64>> {
	log::debug!("getting NIC statistics");

	let handle = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), None)
	             .map_err(|error| anyhow!("can't create socket: {error}"))?;

	let mut sset_info = self::sset_info {
		cmd:       self::GSSET_INFO,
		reserved:  0,
		sset_mask: 1 << self::SS_STATS,
		data:      [0],
	};

	self::send_ioctl(handle, device, std::ptr::addr_of_mut!(sset_info) as *mut libc::c_void)?;

	let nb_stats = sset_info.data[0];

	const NB_STATS_MAX: usize = 512;
	const NAME_SIZE:    usize = self::GSTRING_LEN * std::mem::size_of::<u8>();

	assert!(nb_stats <= NB_STATS_MAX);

	let mut names = self::gstrings {
		cmd:        self::GSTRINGS,
		string_set: self::SS_STATS,
		len:        nb_stats as u32,
		data:       [0u8; NB_STATS_MAX * NAME_SIZE],
	};

	let mut values = self::stats {
		cmd:     self::GSTATS,
		n_stats: nb_stats as u32,
		data:    [0u64; NB_STATS_MAX],
	};

	self::send_ioctl(handle, device, std::ptr::addr_of_mut!(names)  as *mut libc::c_void)?;
	self::send_ioctl(handle, device, std::ptr::addr_of_mut!(values) as *mut libc::c_void)?;

	Ok(std::iter::zip(names.data.chunks(NAME_SIZE), values.data)
	   .take(names.len as usize)
	   .map(|(name, value)| (String::from_utf8_lossy(name).trim_end_matches('\0').into(), value))
	   .collect())
}

#[repr(C)]
#[derive(Debug)]
struct sset_info<T: ?Sized> {
	pub cmd:       u32,
	pub reserved:  u32,
	pub sset_mask: u64,
	pub data:      T,
}

#[repr(C)]
#[derive(Debug)]
struct gstrings<T: ?Sized> {
	pub cmd:        u32,
	pub string_set: u32,
	pub len:        u32,
	pub data:       T,
}

#[repr(C)]
#[derive(Debug)]
struct stats<T: ?Sized> {
	pub cmd:     u32,
	pub n_stats: u32,
	pub data:    T,
}

nix::ioctl_write_ptr_bad!(ioctl_write, libc::SIOCETHTOOL, libc::ifreq);

fn send_ioctl(handle: libc::c_int, device: &str, data: *mut libc::c_void) -> Result<i32> {
	let ifr = libc::ifreq {
		ifr_name: convert_to_ifr_name(device)?,
		ifr_ifru: libc::__c_anonymous_ifr_ifru {
			ifru_data: data as *mut libc::c_char,
		},
	};

	unsafe {
		self::ioctl_write(handle, std::ptr::addr_of!(ifr))
		.map_err(|error| anyhow!("can't perform ioctl write: {error}"))
	}
}

fn convert_to_ifr_name (name: &str) -> Result<[libc::c_char; 16]> {
	if name.len() >= 16 {
		bail!("IFR name \"{name}\" is too long, it should be <= 16 characters");
	}

	let mut result = [0; 16];

	for (index, char) in name.chars().enumerate() {
		result[index] = char as libc::c_char;
	}

	Ok(result)
}

const GSSET_INFO:  u32   = 0x00000037;
const SS_STATS:    u32   = 1;
const GSTRING_LEN: usize = 32;
const GSTRINGS:    u32   = 0x0000001b;
const GSTATS:      u32   = 0x0000001d;
