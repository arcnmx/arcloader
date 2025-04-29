use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
};
use nexus::{font::RawFontReceive, imgui::sys::ImFontConfig};
use windows::Win32::Foundation::HMODULE;
use std::{ffi::{c_char, c_void}, mem::transmute};

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_font_get(identifier: *const c_char, callback: RawFontReceive) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(font::get("{:?}, {:?}", id, callback));

		let font_id = None;
		let ui = arcdps::__macro::ui();
		let fonts = ui.fonts();
		let font = font_id.and_then(|id| fonts.get_font(id))
			.map(|f| f.id())
			.or_else(|| fonts.fonts().first().cloned());
		if let Some(font) = font {
			callback(identifier, transmute(font));
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_font_release(identifier: *const c_char, callback: RawFontReceive) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(font::release("{:?}, {:?}", id, callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_font_add_from_file(identifier: *const c_char, font_size: f32, filename: *const c_char, callback: RawFontReceive, config: *mut ImFontConfig) {
		let id = cstr_opt(&identifier);
		let filename = cstr_opt(&filename);
		addonapi_stub!(font::add_from_file("{:?}, {:?}, {:?}, {:?}, {:?}", id, font_size, filename, callback, config) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_font_add_from_resource(identifier: *const c_char, font_size: f32, resource_id: u32, module: HMODULE, callback: RawFontReceive, config: *mut ImFontConfig) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(font::add_from_resource("{:?}, {:?}, {:?}, {:?}, {:?}, {:?}", id, font_size, resource_id, module, callback, config) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_font_add_from_memory(identifier: *const c_char, font_size: f32, data: *const c_void, size: usize, callback: RawFontReceive, config: *mut ImFontConfig) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(font::add_from_memory("{:?}, {:?}, {:?}, {:?}, {:?}, {:?}", id, font_size, data, size, callback, config) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_font_resize(identifier: *const c_char, font_size: f32) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(font::resize("{:?}, {:?}", id, font_size) => ())
	}
}
