use crate::arg::{Endianness, Arguments};
use crate::error::Error;
use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, StreamTrait};
use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/** When no sample rate is specified, the playback will try to select the value
 * that gets the closest to this number and that is still supported. */
pub const PREFERRED_SAMPLE_RATE: u32 = 48000;

/** When no channel count is specified, the playback will try to select the
 * value that gets the closest to this number and that is still supported. */
pub const PREFERRED_CHANNELS: u16 = 2;

/** When no sample format is specified, the playback will try to select the
 * value that gets the closest to this number and that is still supported. */
pub const PREFERRED_SAMPLE_FORMAT: cpal::SampleFormat = cpal::SampleFormat::I16;

/** When no sample endian is specified, the playback will try to select the
 * value that gets the closest to this number and that is still supported. */
pub const PREFERRED_SAMPLE_ENDIAN: Endianness = Endianness::Little;

/** Plays audio from a given source. */
pub fn play<R>(args: &Arguments, mut source: R)
	where R: Read + Send + 'static {

	eprint!("playing <file> ");
	if let Some((index, name)) = args.device_pick() {
		eprint!("to device {} ({}) ", index, name);
	} else {
		eprint!("to the default device ");
	}
	if let Some((index, name)) = args.host_pick() {
		eprintln!("within host {} ({})", index, name);
	} else {
		eprintln!("within the default host");
	}

	let format = args.config(
		PREFERRED_SAMPLE_RATE,
		PREFERRED_CHANNELS,
		PREFERRED_SAMPLE_FORMAT);
	let format = match format {
		Ok(format) => format,
		Err(what) => {
			eprintln!("error: {}", what);
			std::process::exit(1)
		}
	};

	let endian = args.endianness().unwrap_or(PREFERRED_SAMPLE_ENDIAN);
	eprint!("playing as: {:?}{}, ",
		format.sample_format(),
		match endian {
			Endianness::Little => "LE",
			Endianness::Big    => "BE",
			Endianness::Native => "",
		});
	eprint!("{} channels, ", format.channels());
	eprintln!("{}Hz", format.sample_rate().0);

	/* Create the output stream. */
	let end0 = Arc::new(AtomicBool::new(false));
	let end1 = end0.clone();

	let device = args.device();
	let output = device.build_output_stream_raw(
		&format.config(),
		format.sample_format(),
		move |data, info| {
			let result = source.read_exact(data.bytes_mut());
			match result {
				/*Ok(result) =>
					eprintln!("{:?}: fed {} bytes with {} bytes",
						info.timestamp().playback,
						data.bytes().len(),
						result),*/
				Err(what) => if what.kind() == std::io::ErrorKind::UnexpectedEof {
					eprintln!("e o f");
					end1.store(true, Ordering::Relaxed);
				} else {
					eprintln!("error: data read failed: {}", what);
					std::process::exit(1);
				},
				_ => {}
			}

			let samples = data.bytes().len() / data.sample_format().sample_size();
			let per_sec = format.sample_rate().0 * format.channels() as u32;
			let projected = samples as f64 / per_sec as f64;
		},
		|what| {
			eprintln!("error: output stream failed: {}", what);
			std::process::exit(1);
		});
	let output = match output {
		Ok(output) => output,
		Err(what) => {
			eprintln!("error: could not initialize output stream: {}", what);
			std::process::exit(1);
		}
	};

	output.play();
	while !end0.load(Ordering::Relaxed) { }
	output.pause();
}
