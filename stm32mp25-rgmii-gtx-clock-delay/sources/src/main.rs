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

mod clock_delay;
mod benchmark;
mod device_tree;

use clap::{Parser, Subcommand};
use byte_unit::Byte;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

use anyhow::{Context, Result};

fn main () -> Result<()> {
	let options = Options::parse();

	let _ = stderrlog::new()
	        .verbosity(options.verbose as usize)
	        .color(stderrlog::ColorChoice::Never)
	        .init();

	main2(options)
}

fn main2 (options: Options) -> Result<()> {
	match options.command {
		Command::Benchmark {device, url, speed_low_limit, timeout } => {
			benchmark::perform(&device, &url, speed_low_limit, timeout)
			.context("can't benchmark all possible RGMII GTX clock delays")?
		}

		Command::Set { device, clock_delay } => {
			clock_delay::access(&device, Some(clock_delay), true)
			.context("can't set RGMII GTX clock delay")?
		}

		Command::Get { device } => {
			clock_delay::access(&device, None, true)
			.context("can't get RGMII GTX clock delay")?
		}

		Command::License { } => {
			println!("\n\
				Copyright 2023 STMicroelectronics\n\
				\n\
				Redistribution and use in source and binary forms, with or without\n\
				modification, are permitted provided that the following conditions are\n\
				met:\n\
				\n\
				1. Redistributions of source code must retain the above copyright\n\
				   notice, this list of conditions and the following disclaimer.\n\
				\n\
				2. Redistributions in binary form must reproduce the above copyright\n\
				   notice, this list of conditions and the following disclaimer in the\n\
				   documentation and/or other materials provided with the\n\
				   distribution.\n\
				\n\
				3. Neither the name of the copyright holder nor the names of its\n\
				   contributors may be used to endorse or promote products derived\n\
				   from this software without specific prior written permission.\n\
				\n\
				THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS\n\
				'AS IS' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT\n\
				LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR\n\
				A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT\n\
				HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,\n\
				SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT\n\
				LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,\n\
				DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY\n\
				THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT\n\
				(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE\n\
				OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.\n\
			");
		}
	}

	Ok(())
}

#[derive(Parser)]
struct Options {
	/// Increase verbosity level (once = debug, twice = trace)
	#[clap(short, long, action = clap::ArgAction::Count)]
	verbose: u8,

	#[clap(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
#[clap(author, version, about = "Handle STM32MP25 RGMII GTX clock delay")]
enum Command {
	/// Benchmark all possible RGMII GTX clock delays
	Benchmark {
		/// Device name
		#[clap(short, long)]
		device: String,

		/// Benchmark by fetching data from this URL (recommended size > 100 MiB)
		#[clap(short, long, default_value = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.4.3.tar.xz")]
		url: String,

		/// Skip if transfer rate is below SPEED_LOW_LIMIT/second during more than TIMEOUT seconds
		#[clap(short, long, default_value = "100 kiB", value_parser = speed_low_limit_parser)]
		speed_low_limit: Byte,

		/// Timemout for SPEED_LOW_LIMIT and for the connection phase.
		#[clap(short, long, default_value = "5")]
		timeout: u64,
	},

	/// Set RGMII GTX clock delay
	Set {
		/// Device name
		#[clap(short, long)]
		device: String,

		/// RGMII GTX clock delay (in ns)
		#[clap(short, long, value_parser = clock_delay::parser)]
		clock_delay: f32,
	},

	/// Get RGMII GTX clock delay
	Get {
		/// Device name
		#[clap(short, long)]
		device: String,
	},

	/// Print license & copyright for this software
	License { }
}

fn speed_low_limit_parser (value: &str) -> Result<Byte> {
	Byte::from_str(value).map_err(|error| anyhow!("not a valid size in bytes ({error})"))
}
