use gw2_mumble::{Identity, MumbleLink};
use nexus::{data_link::NexusLink, event::MumbleIdentityUpdate};

use crate::{
	extensions::nexus::NexusHost,
	util::ffi::cstr_opt,
};
use std::{ptr, ffi::{c_char, c_void, CStr}, sync::{Mutex, OnceLock}};

pub fn mumble_identity(id: &Identity) -> MumbleIdentityUpdate {
	let mut name = [0u8; 20];
	let name_len = id.name.len().min(name.len());
	name[..name_len].copy_from_slice(id.name[..].as_bytes());
	MumbleIdentityUpdate {
		name,
		profession: id.profession as u32,
		specialization: id.spec,
		race: id.race as u32,
		map_id: id.map_id,
		world_id: id.world_id,
		team_color_id: id.team_color_id,
		is_commander: id.commander,
		fov: id.fov,
		ui_size: id.ui_scale as u32,
	}
}

pub static MUMBLE_LINK: OnceLock<Option<MumbleLink>> = OnceLock::new();
pub static MUMBLE_LINK_IDENTITY: Mutex<Option<MumbleIdentityUpdate>> = Mutex::new(None);

impl NexusHost {
	pub const DATA_LINK_NEXUS: &'static CStr = cstr!("DL_NEXUS_LINK");
	pub const DATA_LINK_MUMBLE: &'static CStr = cstr!("DL_MUMBLE_LINK");
	pub const DATA_LINK_MUMBLE_IDENTITY: &'static CStr = cstr!("DL_MUMBLE_LINK_IDENTITY");

	pub unsafe extern "C-unwind" fn addonapi_data_link_get(identifier: *const c_char) -> *const c_void {
		let id = cstr_opt(&identifier);

		match id {
			Some(id) if id == Self::DATA_LINK_MUMBLE => return MUMBLE_LINK.get()
				.and_then(|ml| match ml {
					&Some(ref ml) => Some(ml),
					None => None,
				}).map(|ml| ml.as_ptr())
				.unwrap_or(ptr::null()) as *const c_void,
			Some(id) if id == Self::DATA_LINK_MUMBLE_IDENTITY => return MUMBLE_LINK_IDENTITY.lock().unwrap()
				.as_ref()
				.map(|ml| ml as *const MumbleIdentityUpdate)
				.unwrap_or(ptr::null()) as *const c_void,
			Some(id) if id == Self::DATA_LINK_NEXUS => {
				let host = Self::lock_read();
				let nl = host.nexus_link.lock().unwrap();
				let nl: &NexusLink = &nl;
				return nl as *const NexusLink as *const c_void
			},
			_ => (),
		}
		addonapi_stub!(data_link::get("{:?}", id));

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
