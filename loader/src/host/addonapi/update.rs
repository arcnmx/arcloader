use crate::{
	host::addonapi::NexusHost,
	util::ffi::cstr_opt,
};
use std::ffi::c_char;

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_request_update(signature: i32, update_url: *const c_char) {
		let update_url = cstr_opt(&update_url);

		addonapi_stub!(update::request("{:?}, {:?}", signature, update_url) => ())
	}
}
