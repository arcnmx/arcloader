use nexus::event::RawEventConsumeUnknown;

use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
	cstr,
};
use std::ffi::{c_char, c_void, CStr};

impl NexusHost {
	pub const EV_ADDON_LOADED: &'static CStr = cstr!("EV_ADDON_LOADED");
	pub const EV_MUMBLE_IDENTITY_UPDATED: &'static CStr = cstr!("EV_MUMBLE_IDENTITY_UPDATED");

	pub unsafe extern "C-unwind" fn addonapi_event_subscribe(identifier: *const c_char, consume_callback: RawEventConsumeUnknown) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(event::subscribe("{:?}, {:?}", id, consume_callback));

		let id = match id {
			Some(id) => id,
			None => {
				warn!("expected event id");
				return
			},
		};

		Self::cache_write_with(consume_callback as *const _, move |mut cache| {
			let handlers = cache.event_handlers.entry(id.to_owned())
				.or_default();
			handlers.insert(consume_callback);
		});
	}

	pub unsafe extern "C-unwind" fn addonapi_event_unsubscribe(identifier: *const c_char, consume_callback: RawEventConsumeUnknown) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(event::unsubscribe("{:?}, {:?}", id, consume_callback));

		let id = match id {
			Some(id) => id,
			None => {
				warn!("expected event id");
				return
			},
		};

		let removed = Self::cache_write_with(consume_callback as *const _, move |mut cache| {
			cache.event_handlers.get_mut(id)
				.map(|h| h.remove(&consume_callback))
				.unwrap_or(false)
		});
		if !removed {
			warn!("subscriber not found");
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise(identifier: *const c_char, event_data: *const c_void) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise("{:?}, {:?}", id, event_data) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise_targeted(signature: i32, identifier: *const c_char, event_data: *const c_void) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise_targeted("{:?}, {:?}, {:?}", signature, id, event_data) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise_notification(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise_notification("{:?}", id) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_event_raise_notification_targeted(signature: i32, identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(event::raise_notification_targeted("{:?}, {:?}", signature, id) => ())
	}
}
