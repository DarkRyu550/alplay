use crate::arg::Arguments;
use cpal::traits::{HostTrait, DeviceTrait};
use crate::error::Error;
use cpal::SampleFormat;

/** Lists all of the hosts available in this device. */
pub fn list_hosts() {
	eprintln!("**** Lists of AVAILABLE audio hosts ****");

	let hosts = cpal::available_hosts();
	let default = cpal::default_host();

	for (i, host) in hosts.iter().enumerate() {
		print!("host {}: {:?}", i, host);
		if default.id() == *host {
			println!(" [default]")
		} else {
			println!()
		}
	}
}

/** List all of the output devices for a given host. */
pub fn list_devices(arg: &Arguments) -> Result<(), Error> {
	eprintln!("**** List of audio output devices for {} ({:?}) ****",
		match arg.host_pick() {
			Some((index, _)) =>
				format!("host {}", index),
			None =>
				"the default host".to_owned()
		},
		arg.host().id());

	let default = arg.host().default_output_device();
	for (i, device) in arg.host().output_devices()?.enumerate() {
		print!("device {}: ", i);
		match device.name() {
			Ok(name) => {
				print!("{}", name);

				/* We have no real way of checking whether two devices are the
				 * same so, as a workaround, we just check whether their names
				 * are the same, and call that good enough. */
				if let Some(dev) = default.as_ref() {
					if let Ok(d) = dev.name() {
						if d == name { print!(" [default]"); }
					}
				}
			},
			Err(what) => {
				println!();
				print!("    ! error while retrieving device name: {}", what)
			}
		}
		println!();

		let outputs = match device.supported_output_configs() {
			Ok(outputs) => outputs,
			Err(what) => {
				println!("    ! error while retrieving supported configurations: {}", what);
				continue
			}
		};
		for (i, output) in outputs.enumerate() {
			println!("    output {}: ", i);
			println!("        channels: {}", output.channels());
			println!("        format:   {} ({} bytes)",
				match output.sample_format() {
					SampleFormat::I16 => "S16",
					SampleFormat::U16 => "U16",
					SampleFormat::F32 => "F32"
				},
				output.sample_format().sample_size());
			println!("        min rate: {}Hz", output.min_sample_rate().0);
			println!("        max rate: {}Hz", output.max_sample_rate().0);
		}
	}

	Ok(())
}
