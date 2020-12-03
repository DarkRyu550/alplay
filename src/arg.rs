use clap::ArgMatches;
use crate::error::Error;
use cpal::traits::DeviceTrait;
use cpal::{StreamConfig, SupportedStreamConfig};

/** A sample can be in multiple different endians. */
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Endianness {
	Little,
	Big,
	Native
}

pub struct Arguments {
	/** The audio host we are going to be using. */
	host: cpal::Host,
	/** The name given by the user to pick this host, if any. If there is no
	 * such value, it should be assumed that the audio host was picked using
	 * [`cpal::default_host()`]. */
	host_pick: Option<(usize, String)>,

	/** The audio output device we are going to be using. */
	device: cpal::Device,
	/** The name given by the user to pick this device, if any. */
	device_pick: Option<(usize, String)>,

	/** Number of requested output channels. */
	channels: Option<u16>,
	/** Requested output sample rate. */
	sample_rate: Option<u32>,
	/** Requested output sample format. */
	sample_format: Option<(cpal::SampleFormat, Endianness)>
}
impl Arguments {
	/** Creates a new instance of the arguments structure from the parsed
	 * argument strings provided by `clap`. */
	pub fn new(matches: &ArgMatches) -> Result<Self, Error> {
		/* Pick the host and its name. */
		let (host, host_pick) = match matches.value_of(crate::ARG_HOST) {
			Some(host) => {
				let name = host.to_owned();

				let index = usize::from_str_radix(host, 10)
					.map_err(|what| Error::MalformedHost {
						what,
						value: host.to_owned()
					})?;
				let host_id = cpal::available_hosts().get(index)
					.cloned()
					.ok_or(Error::NoSuchHost {
						name: index
					})?;
				let host = cpal::host_from_id(host_id)
					.map_err(|what| Error::HostUnavailable {
						what,
						name: index,
						id: host_id
					})?;

				(host, Some((index, name)))
			},
			None =>
				/* Just pick the default audio host. */
				(cpal::default_host(), None)
		};

		/* Pick the device and specify its name. */
		use cpal::traits::HostTrait;
		let (device, device_pick) = match matches.value_of(crate::ARG_DEVICE) {
			Some(device) => {
				unimplemented!()
			},
			None =>
				/* Just pick the default audio output. */
				(
					host.default_output_device()
						.ok_or(Error::NoOutputDevice {
							host_pick: host_pick.clone()
						})?,
					None
				)
		};

		/* Get the values for the channels and sample rate. */
		let channels = matches.value_of(crate::ARG_CHANNELS)
			.map(|channels| u16::from_str_radix(channels, 10))
			.transpose()
			.map_err(|what| Error::MalformedChannels(what))?;
		let sample_rate = matches.value_of(crate::ARG_SAMPLE_RATE)
			.map(|sample_rate| u32::from_str_radix(sample_rate, 10))
			.transpose()
			.map_err(|what| Error::MalformedSampleRate(what))?;

		/* Get the endianness and format specification for the sample. */
		let sample_format = matches.value_of(crate::ARG_SAMPLE_FORMAT)
			.map(|sample_format| match sample_format.to_ascii_lowercase().as_str() {
				"f32le" => Ok((cpal::SampleFormat::F32, Endianness::Little)),
				"s16le" => Ok((cpal::SampleFormat::I16, Endianness::Little)),
				"u16le" => Ok((cpal::SampleFormat::U16, Endianness::Little)),
				"f32be" => Ok((cpal::SampleFormat::F32, Endianness::Big)),
				"s16be" => Ok((cpal::SampleFormat::I16, Endianness::Big)),
				"u16be" => Ok((cpal::SampleFormat::U16, Endianness::Big)),
				"f32"   => Ok((cpal::SampleFormat::F32, Endianness::Native)),
				"s16"   => Ok((cpal::SampleFormat::I16, Endianness::Native)),
				"u16"   => Ok((cpal::SampleFormat::U16, Endianness::Native)),
				n @ _ => Err(Error::MalformedSampleFormat {
					expected: &[
						"f32le",
						"s16le",
						"u16le",
						"f32be",
						"s16be",
						"u16be",
						"f32",
						"s16",
						"u16"
					],
					got: n.to_string()
				})
			})
			.transpose()?;

		Ok(Self {
			host,
			host_pick,
			device,
			device_pick,
			channels,
			sample_rate,
			sample_format
		})
	}

	/** Pick an audio host that matches the given settings. */
	pub fn host(&self) -> &cpal::Host {
		&self.host
	}

	/** The selection parameters used to pick the current host. If no selection
	 * was made, it should be assumed that the default host was picked. */
	pub fn host_pick(&self) -> Option<(usize, &str)> {
		self.host_pick
			.as_ref()
			.map(|(a, b)| (*a, b.as_str()))
	}

	/** Pick an audio output device that matches the given settings. */
	pub fn device(&self) -> &cpal::Device {
		&self.device
	}

	/** The selection parameters used to pick the current device. If no
	 * selection was made, it should be assumed that the default host was
	 * picked. */
	pub fn device_pick(&self) -> Option<(usize, &str)> {
		self.device_pick
			.as_ref()
			.map(|(a, b)| (*a, b.as_str()))
	}

	/** Format for interpret the input data as. */
	pub fn endianness(&self) -> Option<Endianness> {
		self.sample_format.map(|(_, a)| a)
	}

	/** Find the best suited output stream configuration, if any is possible. */
	pub fn config(
		&self,
		preferred_sample_rate: u32,
		preferred_channels: u16,
		preferred_sample_format: cpal::SampleFormat)
		-> Result<SupportedStreamConfig, Error> {

		let mut best = None;
		for output in self.device.supported_output_configs()? {
			let channels = if let Some(channels) = self.channels {
				if output.channels() != channels { continue }
				channels
			} else {
				preferred_channels
			};

			let format = if let Some((format, _)) = self.sample_format {
				if output.sample_format() != format { continue }
				format
			} else {
				preferred_sample_format
			};

			let sample_rate = if let Some(sample_rate) = self.sample_rate {
				if sample_rate < output.min_sample_rate().0 { continue }
				if sample_rate > output.max_sample_rate().0 { continue }

				sample_rate
			} else {
				preferred_sample_rate
			};

			let config = if output.min_sample_rate().0 > sample_rate {
				let max = output.min_sample_rate();
				output.with_sample_rate(max)
			} else if output.max_sample_rate().0 < sample_rate {
				let min = output.min_sample_rate();
				output.with_sample_rate(min)
			} else {
				output.with_sample_rate(cpal::SampleRate(sample_rate))
			};

			let better = |best: &SupportedStreamConfig, candidate: &SupportedStreamConfig| {
				let a =
					  u32::max(best.sample_rate().0, sample_rate)
					- u32::min(best.sample_rate().0, sample_rate);
				let b =
					  u16::max(best.channels(), channels)
					- u16::min(best.channels(), channels);
				let c = if best.sample_format() != format { 1 } else { 0 };
				let best = a + u32::from(b) + c;

				let a =
					  u32::max(candidate.sample_rate().0, sample_rate)
					- u32::min(candidate.sample_rate().0, sample_rate);
				let b =
					  u16::max(candidate.channels(), channels)
					- u16::min(candidate.channels(), channels);
				let c = if candidate.sample_format() != format { 1 } else { 0 };
				let candidate = a + u32::from(b) + c;

				candidate < best
			};
			best = Some(match best {
				Some(best) =>
					if better(&best, &config) {
						config
					} else {
						best
					}
				None => config
			});
		}

		best.ok_or(Error::NoSuitableStreamConfig {
			required_format: self.sample_format.map(|(a, _)| a),
			required_sample_rate: self.sample_rate,
			required_channels: self.channels
		})
	}
}
