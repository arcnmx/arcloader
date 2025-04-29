use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
};
use std::ffi::c_char;

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_ui_send_alert(message: *const c_char) {
		let message = cstr_opt(&message);

		addonapi_stub!(ui::send_alert("{:?}", message) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_ui_register_close_on_escape(window_name: *const c_char, is_visible: *mut bool) {
		let window_name = cstr_opt(&window_name);

		addonapi_stub!(ui::register_close_on_escape("{:?}, {:?}", window_name, is_visible) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_ui_deregister_close_on_escape(window_name: *const c_char) {
		let window_name = cstr_opt(&window_name);

		addonapi_stub!(ui::deregister_close_on_escape("{:?}", window_name) => ())
	}
}
