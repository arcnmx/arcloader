use nexus::keybind::{Keybind, RawKeybindHandler, RawKeybindHandlerOld};
use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
};
use std::ffi::c_char;

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_string(identifier: *const c_char, keybind_handler: RawKeybindHandler, keybind: *const c_char) {
		let id = cstr_opt(&identifier);
		let keybind = cstr_opt(&keybind);

		addonapi_stub!(input_binds::register_with_string("{:?}, {:?}, {:?}", id, keybind_handler, keybind) => ());

		/*Self::cache_write_with(keybind_handler as *const _, move |mut cache| {
			let handlers = cache.key_binds.entry(render_type)
				.or_default();
			handlers.insert(keybind_handler);
		});*/
		if let Some(kb) = keybind {
			if kb.to_bytes().len() > 1 {
				NexusHost::lock_read().fallback_cache.write().unwrap().key_binds.insert(kb.to_owned(), keybind_handler);
			}
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_struct(identifier: *const c_char, keybind_handler: RawKeybindHandler, keybind: Keybind) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(input_binds::register_with_struct("{:?}, {:?}, {:?}", id, keybind_handler, keybind) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_deregister(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(input_binds::deregister("{:?}", id) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_invoke(identifier: *const c_char, is_release: bool) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(input_binds::invoke("{:?}, {:?}", id, is_release) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_string_v2(identifier: *const c_char, keybind_handler: RawKeybindHandlerOld, keybind: *const c_char) {
		let id = cstr_opt(&identifier);
		let keybind = cstr_opt(&keybind);

		addonapi_stub!(input_binds::register_with_string_v2("{:?}, {:?}, {:?}", id, keybind_handler, keybind) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_struct_v2(identifier: *const c_char, keybind_handler: RawKeybindHandlerOld, keybind: nexus::keybind::Keybind) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(input_binds::register_with_struct_v2("{:?}, {:?}, {:?}", id, keybind_handler, keybind) => ())
	}
}
