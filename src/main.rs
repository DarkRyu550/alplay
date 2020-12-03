use clap::{App, Arg, SubCommand};
use crate::arg::Arguments;

/** Playback functionality. */
mod play;

/** Diagnostics functionality. */
mod diag;

/** Runtime argument processor. */
mod arg;

/** Error type. */
mod error;

/** Audio source types. */
mod src;

/** Argument ID for host specification. */
const ARG_HOST: &'static str = "HOST";
/** Argument ID for device specification */
const ARG_DEVICE: &'static str = "DEVICE";
/** Argument ID for channel count specification */
const ARG_CHANNELS: &'static str = "CHANNELS";
/** Argument ID for channel count specification */
const ARG_EXTERNAL_SYNC: &'static str = "EXTERNAL_SYNC";
/** Argument ID for sample rate specification */
const ARG_SAMPLE_RATE: &'static str = "SAMPLE_RATE";
/** Argument ID for sample format specification */
const ARG_SAMPLE_FORMAT: &'static str = "SAMPLE_FORMAT";
/** Subcommand ID for device listing. */
const ARG_LIST_DEVICES: &'static str = "DEVICES";
/** Subcommand ID for host listing. */
const ARG_LIST_HOSTS: &'static str = "HOSTS";

fn main() {
	let matches = App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about("")
		.args(&[
			Arg::with_name(ARG_HOST)
				.short("s")
				.long("host")
				.takes_value(true)
				.help("specify the name of the audio host to be used"),
			Arg::with_name(ARG_DEVICE)
				.short("d")
				.long("device")
				.takes_value(true)
				.help("specify the name of the audio device to be used"),
			Arg::with_name(ARG_LIST_HOSTS)
				.short("l")
				.long("list-hosts")
				.takes_value(false)
				.help("list all available audio hosts"),
			Arg::with_name(ARG_LIST_DEVICES)
				.short("L")
				.long("list-devices")
				.takes_value(false)
				.help("list all audio output devices in a given host"),
			Arg::with_name(ARG_CHANNELS)
				.short("c")
				.long("channels")
				.takes_value(true)
				.help("specify the number of channels for audio playback"),
			Arg::with_name(ARG_SAMPLE_RATE)
				.short("r")
				.long("rate")
				.takes_value(true)
				.help("specify the sample rate for audio playback"),
			Arg::with_name(ARG_SAMPLE_FORMAT)
				.short("f")
				.long("format")
				.takes_value(true)
				.help("specify the sample format for audio playback"),
			Arg::with_name(ARG_EXTERNAL_SYNC)
				.short("e")
				.long("external-sync")
				.takes_value(false)
				.help("sync playback to external source")
		])
		.get_matches();

	let args = match Arguments::new(&matches) {
		Ok(args) => args,
		Err(what) => {
			eprintln!("error: {}", what);
			std::process::exit(1);
		}
	};

	if matches.is_present(ARG_LIST_HOSTS) {
		diag::list_hosts();
	} else if matches.is_present(ARG_LIST_DEVICES) {
		diag::list_devices(&args);
	} else {
		let stdin = std::io::stdin();
		if matches.is_present(ARG_EXTERNAL_SYNC) {
			let source = src::Skipper::new_with_capacity(stdin, 16 * 1024 * 1024);
			play::play(&args, source);
		} else {
			play::play(&args, stdin);
		}
	}
}
