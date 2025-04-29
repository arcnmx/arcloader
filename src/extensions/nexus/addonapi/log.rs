use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
};
use nexus::log::LogLevel;
use std::ffi::c_char;

impl NexusHost {
	#[cfg(feature = "log")]
	pub unsafe extern "C-unwind" fn addonapi_log(level: LogLevel, channel: *const c_char, message: *const c_char) {
		use log::Level;

		if let LogLevel::Off = level {
			#[cfg(all(feature = "log", debug_assertions))] {
				let channel = cstr_opt(&channel);
				let message = cstr_opt(&message);
				log::debug!("discarded addon log: [{}] {}",
					channel.unwrap_or_default().to_str().unwrap_or_default(),
					message.unwrap_or_default().to_str().unwrap_or_default(),
				);
			}

			return
		}

		let level = match level {
			LogLevel::Trace => Some(Level::Trace),
			LogLevel::Debug => Some(Level::Debug),
			LogLevel::Info => Some(Level::Info),
			LogLevel::Warning => Some(Level::Warn),
			LogLevel::Critical => Some(Level::Error),
			/*LogLevel::Off | LogLevel::All |*/ _ => None,
		};

		match level {
			Some(level) if !log::log_enabled!(level) =>
				return,
			_ => (),
		}

		let channel = cstr_opt(&channel)
			.map(|c| c.to_string_lossy());
		let message = cstr_opt(&message);

		let mut metadata = log::MetadataBuilder::new();
		if let Some(channel) = &channel {
			metadata.target(channel);
		}
		if let Some(level) = level {
			metadata.level(level);
		}
		log::logger().log(&log::RecordBuilder::new()
			.metadata(metadata.build())
			.args(format_args!("{}", message.unwrap_or_default().to_string_lossy()))
			.build()
		);
	}

	#[cfg(not(feature = "log"))]
	pub unsafe extern "C-unwind" fn addonapi_log(level: LogLevel, channel: *const c_char, message: *const c_char) {
		let (log_file, log_window) = match level {
			#[cfg(debug_assertions)]
			LogLevel::Trace => (false, true),
			#[cfg(not(debug_assertions))]
			LogLevel::Trace => (false, false),
			LogLevel::Debug => (true, false),
			LogLevel::Info | LogLevel::Warning | LogLevel::Critical =>
				(true, true),
		};
		if log_file {
			arcdps::exports::raw::e3_log_file(message);
		}
		if log_window {
			arcdps::exports::raw::e8_log_window(message);
		}
	}
}
