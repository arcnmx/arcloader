use crate::{
	host::addonapi::NexusHost,
	util::ffi::cstr_opt,
};
use std::{ffi::{c_char, c_void, CStr}, pin::Pin, ptr::{self, NonNull}};

mod mumble;
mod nexus;

pub use self::{
	mumble::{MumbleIdentity, MumbleLinkProvider},
	nexus::NexusLinkProvider,
};

#[cfg(todo2)]
use super::host::RegistrationSource;

pub unsafe trait DataLinkShare {
	fn get_data_share(this: Pin<&mut Self>) -> NonNull<[u8]> where
		Self: Sized
	{
		unsafe {
			this.get_unchecked_mut().get_data_share_pinned()
		}
	}

	unsafe fn get_data_share_pinned(&mut self) -> NonNull<[u8]>;
}

#[cfg(todo2)]
pub struct DataLinkRegistration {
	pub source: RegistrationSource,
	pub share: Box<dyn DataLinkShare>,
}

impl NexusHost {
	pub const DATA_LINK_MUMBLE: &'static CStr = cstr!("DL_MUMBLE_LINK");
	pub const DATA_LINK_MUMBLE_IDENTITY: &'static CStr = cstr!("DL_MUMBLE_LINK_IDENTITY");
	pub const DATA_LINK_NEXUS: &'static CStr = cstr!("DL_NEXUS_LINK");

	pub unsafe extern "C-unwind" fn addonapi_data_link_get(identifier: *const c_char) -> *const c_void {
		let id = cstr_opt(&identifier);
		addonapi_stub!(data_link::get("{:?}", id));

		match id {
			Some(id) if id == Self::DATA_LINK_MUMBLE => return MumbleLinkProvider::get_ptr()
				.map(|ml| ml.as_ptr())
				.unwrap_or(ptr::null()) as *const c_void,
			Some(id) if id == Self::DATA_LINK_MUMBLE_IDENTITY => return NexusHost::lock_read().mumble_link_identity_ptr()
				.map(|ml| ml.as_ptr() as *const _)
				.unwrap_or(ptr::null()) as *const c_void,
			Some(id) if id == Self::DATA_LINK_NEXUS => return NexusLinkProvider::get_ptr()
				.map(|ml| ml.as_ptr() as *const _)
				.unwrap_or(ptr::null()) as *const c_void,
			_ => (),
		}

		id.and_then(|id| Self::cache_read_with(ptr::null(), |cache|
			cache.shared_data.get(id).map(|d| d.as_ptr())
		)).unwrap_or(ptr::null()) as *const c_void
	}

	pub unsafe extern "C-unwind" fn addonapi_data_link_share(identifier: *const c_char, resource_size: usize) -> *mut c_void {
		let id = cstr_opt(&identifier);
		addonapi_stub!(data_link::share("{:?}, {:?}", id, resource_size));

		let id = match id {
			Some(id) => id,
			None => {
				warn!("expected data link id");
				return ptr::null_mut()
			},
		};

		let mut data = vec![0u8; resource_size].into_boxed_slice();
		let ptr = data.as_mut_ptr();
		Self::cache_write_with(ptr::null(), |mut cache|
			cache.shared_data.insert(id.to_owned(), data)
		);
		// TODO: use caches to clean up when owner dies!

		ptr as *mut c_void
	}
}
