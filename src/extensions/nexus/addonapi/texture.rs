use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
};
use nexus::texture::{RawTextureReceiveCallback, Texture};
use windows::Win32::Foundation::HMODULE;
use std::{ffi::{c_char, c_void}, ptr};

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_texture_get(identifier: *const c_char) -> *const Texture {
		let id = cstr_opt(&identifier);

		addonapi_stub!(texture::get("{:?}", id) => ptr::null())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_file(identifier: *const c_char, filename: *const c_char, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);
		let filename = cstr_opt(&filename);

		addonapi_stub!(texture::load_from_file("{:?}, {:?}, {:?}", id, filename, callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_url(identifier: *const c_char, remote: *const c_char, endpoint: *const c_char, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);
		let remote = cstr_opt(&remote);
		let endpoint = cstr_opt(&endpoint);

		addonapi_stub!(texture::load_from_url("{:?}, {:?}, {:?}, {:?}", id, remote, endpoint, callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_memory(identifier: *const c_char, data: *const c_void, size: usize, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(texture::load_from_memory("{:?}, {:?}, {:?}, {:?}", id, data, size, callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_resource(identifier: *const c_char, resource_id: u32, module: HMODULE, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(texture::load_from_resource("{:?}, {:?}, {:?}, {:?}", id, resource_id, module, callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_file(identifier: *const c_char, filename: *const c_char) -> *const Texture {
		let id = cstr_opt(&identifier);
		let filename = cstr_opt(&filename);

		addonapi_stub!(texture::get_or_create_from_file("{:?}, {:?}", id, filename) => ptr::null())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_resource(identifier: *const c_char, resource_id: u32, module: HMODULE) -> *const Texture {
		let id = cstr_opt(&identifier);

		addonapi_stub!(texture::get_or_create_from_resource("{:?}, {:?}, {:?}", id, resource_id, module) => ptr::null())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_url(identifier: *const c_char, remote: *const c_char, endpoint: *const c_char) -> *const Texture {
		let id = cstr_opt(&identifier);
		let remote = cstr_opt(&remote);
		let endpoint = cstr_opt(&endpoint);

		addonapi_stub!(texture::get_or_create_from_url("{:?}, {:?}, {:?}", id, remote, endpoint) => ptr::null())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_memory(identifier: *const c_char, data: *const c_void, size: usize) -> *const Texture {
		let id = cstr_opt(&identifier);

		addonapi_stub!(texture::get_or_create_from_memory("{:?}, {:?}, {:?}", id, data, size) => ptr::null())
	}
}
