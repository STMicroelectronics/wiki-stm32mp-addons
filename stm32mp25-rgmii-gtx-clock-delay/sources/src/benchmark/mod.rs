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

mod ethtool;

use crate::clock_delay;
use crate::device_tree;

use byte_unit::Byte;
use std::time::{Instant, Duration};
use std::ops::Range;
use anyhow::Result;

pub(crate) fn perform(device: &str, url: &str, speed_low_limit: Byte, timeout: u64) -> Result<()> {
	let reversed_valid_values = clock_delay::VALID_VALUES.iter().cloned().rev().collect::<Vec<_>>();

	println!("Using URL {url}");

	println!("Pass 1/2");
	let results1 = perform_single_pass(device, url, speed_low_limit, timeout, &clock_delay::VALID_VALUES)?;

	println!("Pass 2/2");
	let results2 = perform_single_pass(device, url, speed_low_limit, timeout, &reversed_valid_values)?;

	let results = std::iter::zip(results1, results2.iter().rev()).map(|(a, b)| a + b).collect::<Vec<_>>();

	let mut best_results = Vec::new();

	for strike in find_strikes(&results) {
		let middle = (strike.start as f32 + strike.end as f32) / 2.0;
		let index1 = middle.floor() as usize;
		let index2 = middle.ceil() as usize;
		let best   = if results[index1] < results[index2] { index1 } else { index2 };
		best_results.push(best);
	}

	best_results.sort_by(|a, b| results[*b].partial_cmp(&results[*a]).unwrap());

	match best_results.pop() {
		None        => println!("No reliable RGMII GTX clock delay found"),
		Some(index) => {
			let best_value = clock_delay::VALID_VALUES[index];

			println!("Best RGMII GTX clock delay is {:.2} ns", best_value);

			let best_value = clock_delay::convert_to_bits(best_value).unwrap();
			let dt_name    = device_tree::get_name(device)?;
			let gpio       = clock_delay::get_gpio(&dt_name)?;
			let nodes      = device_tree::find_nodes(&gpio);

			if nodes.is_empty() {
				log::error!("Can't find any device-tree node that uses GPIO {gpio}");
			} else {
				println!("To permanently use this RGMII GTX clock delay, add \"st,io-delay = <{best_value:#02x}>;\" into following device-tree node(s):");
				for node in &nodes {
					println!("\t{node}");
				}
			}
		}
	}

	Ok(())
}

fn perform_single_pass(device: &str, url: &str, speed_low_limit: Byte, timeout: u64, delays: &[f32]) -> Result<Vec<f32>> {
	let mut results = Vec::new();

	for clock_delay in delays.iter() {
		use std::io::Write;

		let clock_delay = *clock_delay;

		clock_delay::access(device, Some(clock_delay), false)?;

		let message = format!("Benchmarking RGMII GTX clock delay = {clock_delay:.2} nanoseconds... ");
		let _ = std::io::stdout().write(message.as_bytes());
		let _ = std::io::stdout().flush();

		let start = get_info(device)?;

		let status = download(url, speed_low_limit, timeout);
		if let Err(error) = &status {
			if error.is_operation_timedout() {
				println!("{error}");
				results.push(f32::NAN);
				continue;
			}
		}
		status?;

		let end = get_info(device)?;

		let mmc_rx_crc_error = end.mmc_rx_crc_error - start.mmc_rx_crc_error;
		let rx_pkt_n         = end.rx_pkt_n         - start.rx_pkt_n;
		let percent          = (100 * mmc_rx_crc_error) as f32 / rx_pkt_n as f32;
		let duration         = end.instant - start.instant;

		println!("Done in {:.2}s; CRC error rate was {percent:.2}% ({mmc_rx_crc_error}/{rx_pkt_n})", duration.as_secs_f32());

		results.push(percent);
	}

	assert_eq!(results.len(), delays.len());

	Ok(results)
}

fn download(url: &str, speed_low_limit: Byte, timeout: u64) -> Result<(), curl::Error> {
	use curl::easy as curl;

	let mut handle = curl::Easy::new();

	handle.url(url)?;
	handle.fail_on_error(true)?;

	// Abort if transfer speed is < speed_low_limit/second during timeout seconds.
	let timeout = Duration::from_secs(timeout);

	handle.low_speed_limit(speed_low_limit.get_bytes() as u32)?;
	handle.low_speed_time(timeout)?;
	handle.connect_timeout(timeout)?;

	let curl_result = {
		let mut transfer = handle.transfer();

		transfer.write_function(|data| {
			Ok(data.len())
		})?;

		transfer.perform()
	};

	curl_result?;

	Ok(())
}

fn get_info(device: &str) -> Result<Info> {
	let nic_stats = ethtool::get_nic_stats(device)?;

	let get = |key| nic_stats.get(key)
	                .ok_or(anyhow!("can't find NIC statistic named \"{key}\" for device {device}"));

	Ok(Info {
		mmc_rx_crc_error: *get("mmc_rx_crc_error")?,
		rx_pkt_n:         *get("rx_pkt_n")?,
		instant:          Instant::now(),
	})
}

struct Info {
	mmc_rx_crc_error: u64,
	rx_pkt_n:         u64,
	instant:          Instant,
}

fn find_strikes (array: &[f32]) -> Vec<Range<usize>> {
	let mut strikes = Vec::new();
	let mut start   = None;

	for (index, value) in array.iter().enumerate() {
		if value.is_nan() {
			if let Some(index_start) = start {
				strikes.push(Range { start: index_start, end: index - 1 });
				start = None;
			}
		} else if start.is_none() {
			start = Some(index);
		}
	}

	if let Some(index_start) = start {
		strikes.push(Range { start: index_start, end: array.len() - 1 });
	}

	strikes
}

#[test]
fn test_find_strikes () {
	let array = [f32::NAN, 1.89, 1.78, 1.88, 1.87, 1.99, 1.91, f32::NAN];
	assert_eq!(find_strikes(&array), vec![(1 .. 6)]);

	let array = [f32::NAN, 1.89, 1.78, 1.88, f32::NAN, 1.87, 1.99, 1.91, f32::NAN];
	assert_eq!(find_strikes(&array), vec![(1 .. 3), (5 .. 7)]);

	let array = [1.89, 1.78, 1.88, f32::NAN, 1.87, 1.99, 1.91];
	assert_eq!(find_strikes(&array), vec![(0 .. 2), (4 .. 6)]);

	let array = [f32::NAN, 1.87, 1.99, 1.91];
	assert_eq!(find_strikes(&array), vec![(1 .. 3)]);

	let array = [1.89, 1.78, 1.88, f32::NAN];
	assert_eq!(find_strikes(&array), vec![(0 .. 2)]);

	let array = [f32::NAN, 1.89, 1.78, f32::NAN, 1.88, 1.87, f32::NAN, 1.99, 1.91, f32::NAN];
	assert_eq!(find_strikes(&array), vec![(1 .. 2), (4 .. 5), (7 .. 8)]);

	let array = [f32::NAN];
	assert_eq!(find_strikes(&array), Vec::new());
}
