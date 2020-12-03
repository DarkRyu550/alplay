use std::num::ParseIntError;
use cpal::SampleFormat;

#[derive(Debug)]
pub enum Error {
	MalformedHost {
		what: ParseIntError,
		value: String,
	},
	NoSuchHost {
		name: usize,
	},
	HostUnavailable {
		what: cpal::HostUnavailable,
		name: usize,
		id: cpal::HostId,
	},
	NoOutputDevice {
		host_pick: Option<(usize, String)>
	},
	DevicesError(cpal::DevicesError),
	MalformedChannels(ParseIntError),
	MalformedSampleRate(ParseIntError),
	MalformedSampleFormat {
		expected: &'static [&'static str],
		got: String
	},
	SupportedStreamConfigsError(cpal::SupportedStreamConfigsError),
	NoSuitableStreamConfig {
		required_format: Option<cpal::SampleFormat>,
		required_sample_rate: Option<u32>,
		required_channels: Option<u16>,
	},
}
impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::MalformedHost { what, value: name } =>
				write!(f, "the given host name \"{}\" is malformed: {}", name, what),
			Self::HostUnavailable { what, name, id } =>
				write!(f, "failed to use host '{}' ({:?}): {}", name, id, what),
			Self::NoSuchHost { name } =>
				write!(f, "no such host {}", name),
			Self::NoOutputDevice { host_pick } => match host_pick {
				Some((index, name)) =>
					write!(f, "host {} ({}) has no audio output devices", index, name),
				None =>
					write!(f, "the default host has no audio output devices")
			},
			Self::DevicesError(what) =>
				write!(f, "{}", what),
			Self::MalformedChannels(what) =>
				write!(f, "the given channel count is malformed: {}", what),
			Self::MalformedSampleRate(what) =>
				write!(f, "the given sample rate is malformed: {}", what),
			Self::MalformedSampleFormat { expected, got } => {
				write!(f, "the given sample format \"{}\" is malformed. expected one of {{", got)?;
				for (i, expect) in expected.iter().enumerate() {
					write!(f, "\"{}\"", expect)?;
					if i + 1 < expected.len() {
						write!(f, ", ")?;
					}
				}
				write!(f, "}}")
			},
			Self::SupportedStreamConfigsError(what) =>
				write!(f, "{}", what),
			Self::NoSuitableStreamConfig {
				required_format,
				required_sample_rate,
				required_channels } => {

				let count =
					  if required_channels.is_some()    { 1 } else { 0 }
					+ if required_format.is_some()      { 1 } else { 0 }
					+ if required_sample_rate.is_some() { 1 } else { 0 };

				if count == 0 {
					write!(f, "the device supports no output formats")
				} else {
					write!(f, "failed to find a suitable output format with ")?;

					let mut written = 0;
					if let Some(format) = required_format.as_ref() {
						write!(f, "{} format",
							match format {
								SampleFormat::I16 => "an S16",
								SampleFormat::U16 => "a U16",
								SampleFormat::F32 => "an F32"
							})?;
						written += 1;
					}

					if let Some(sample_rate) = required_sample_rate.as_ref() {
						if written > 0 {
							if count == 2 {
								write!(f, ", ")?;
							} else if count == 3 {
								write!(f, " and ")?;
							}

							write!(f, "a sample rate of {}Hz", sample_rate)?;
							written += 1;
						}
					}

					if let Some(channels) = required_channels.as_ref() {
						if written > 0 {
							write!(f, " and ")?;
						}
						write!(f, "{} channels", channels)?;
					}

					Ok(())
				}
			}
		}
	}
}
impl std::error::Error for Error {}
impl From<cpal::DevicesError> for Error {
	fn from(what: cpal::DevicesError) -> Self {
		Self::DevicesError(what)
	}
}
impl From<cpal::SupportedStreamConfigsError> for Error {
	fn from(what: cpal::SupportedStreamConfigsError) -> Self {
		Self::SupportedStreamConfigsError(what)
	}
}