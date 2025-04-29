use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
};
use std::ffi::c_char;

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_localization_translate(identifier: *const c_char) -> *const c_char {
		let id = cstr_opt(&identifier);
		addonapi_stub!(localization::translate("{:?}", id) => identifier)
	}

	pub unsafe extern "C-unwind" fn addonapi_localization_translate_to(identifier: *const c_char, language_identifier: *const c_char) -> *const c_char {
		let id = cstr_opt(&identifier);
		let lang = cstr_opt(&language_identifier);
		addonapi_stub!(localization::translate_to("{:?}, {:?}", id, lang) => identifier)
	}

	pub unsafe extern "C-unwind" fn addonapi_localization_set(identifier: *const c_char, language_identifier: *const c_char, string: *const c_char) {
		let id = cstr_opt(&identifier);
		let lang = cstr_opt(&language_identifier);
		let string = cstr_opt(&string);
		addonapi_stub!(localization::set("{:?}, {:?}, {:?}", id, lang, string) => ())
	}
}
