use gw2_mumble::{Identity, MumbleLink, MumblePtr};
use nexus::{data_link::NexusLink, event::MumbleIdentityUpdate};
use windows_strings::PCWSTR;

use crate::{
	host::addonapi::NexusHost,
	util::{ffi::{cstr_opt, nonnull_ref_unchecked}, win::WideUtf8Reader},
};
use std::{ffi::{c_char, c_void, CStr, OsString}, hash::{DefaultHasher, Hash, Hasher}, mem::MaybeUninit, os::windows::ffi::OsStringExt, ptr::{self, NonNull}, sync::OnceLock};

#[derive(Clone)]
pub struct MumbleIdentity {
	pub hash: u64,
	len: usize,
	pub identity_data: [u16; Self::ID_DATA_LEN + 1],
	pub identity: Box<MumbleIdentityUpdate>,
}

impl MumbleIdentity {
	pub const ID_DATA_LEN: usize = 256;
	pub const ID_EMPTY: MumbleIdentityUpdate = unsafe {
		MaybeUninit::zeroed().assume_init()
	};

	pub fn new() -> Self {
		Self {
			hash: 0,
			len: 0,
			identity_data: [0u16; Self::ID_DATA_LEN + 1],
			identity: Box::new(Self::ID_EMPTY),
		}
	}

	pub fn identity_update_ptr(this: *const Self) -> NonNull<MumbleIdentityUpdate> {
		unsafe {
			let boxp = ptr::addr_of!((*this).identity);
			nonnull_ref_unchecked(ptr::addr_of!(**boxp))
		}
	}

	pub fn identity_data(&self) -> &[u16] {
		unsafe {
			self.identity_data.get_unchecked(..self.len)
		}
	}

	pub fn identity_str_iter(&self) -> WideUtf8Reader {
		WideUtf8Reader::new(self.identity_data())
	}

	pub fn identity_string(&self) -> OsString {
		OsString::from_wide(self.identity_data())
	}

	pub fn borrow_identity(ml: &MumblePtr) -> &[u16] {
		let lm = ml.as_ptr();
		let identity = unsafe { ptr::addr_of!((*lm).identity) };
		let identity = PCWSTR::from_raw(identity as *const u16);
		let identity = unsafe { identity.as_wide() };
		if identity.len() >= Self::ID_DATA_LEN {
			error!("identity string unterminated");
			return &[]
		}

		unsafe { &*(identity as *const [u16]) }
	}

	pub fn update_identity_str(&mut self, ml: &MumblePtr) -> bool {
		let identity = Self::borrow_identity(ml);
		let len = identity.len();
		let mut hasher = DefaultHasher::new();
		identity.hash(&mut hasher);
		let hash = hasher.finish();
		if self.hash == hash {
			return false
		}
		unsafe {
			ptr::copy_nonoverlapping(identity.as_ptr(), self.identity_data.as_mut_ptr(), len);
		}
		self.hash = hash;
		self.len = len;

		true
	}

	pub fn update_identity_from_str(&mut self) -> bool {
		let identity = match self.len {
			0 => None,
			_ => match serde_json::from_reader(self.identity_str_iter()) {
				Ok(mli) => {
					debug!("mumble identity: {mli:#?}");
					Some(mli)
				},
				Err(e) => {
					if e.is_io() || e.is_eof() {
						// TODO: mark string encoding invalid?
					}
					error!("Failed to parse mumble link identity: {e}");
					debug!("unsupported data: {:?}", self.identity_string());
					return false
				},
			},
		};
		let identity = identity.as_ref().map(Self::update_from_parsed)
			.unwrap_or(Self::ID_EMPTY);
		unsafe {
			ptr::write_volatile(&mut *self.identity, identity);
		}

		true
	}

	pub fn update_from_parsed(id: &Identity) -> MumbleIdentityUpdate {
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

	pub fn update_is_empty(id: &MumbleIdentityUpdate) -> bool {
		id.name[0] == 0 && id.world_id == 0 && id.fov == 0.0
	}
}

pub static MUMBLE_LINK: OnceLock<Option<MumbleLink>> = OnceLock::new();

impl NexusHost {
	pub const DATA_LINK_NEXUS: &'static CStr = cstr!("DL_NEXUS_LINK");
	pub const DATA_LINK_MUMBLE: &'static CStr = cstr!("DL_MUMBLE_LINK");
	pub const DATA_LINK_MUMBLE_IDENTITY: &'static CStr = cstr!("DL_MUMBLE_LINK_IDENTITY");

	pub fn update_mumble_link_identity(&mut self) -> Option<NonNull<MumbleIdentityUpdate>> {
		let ml = MUMBLE_LINK.get()?.as_ref()?;
		let id = self.mumble_identity.as_mut()?;
		if !id.update_identity_str(&ml.as_mumble_ptr()) {
			return None
		}
		match id.update_identity_from_str() {
			true => Some(MumbleIdentity::identity_update_ptr(id)),
			_ => None,
		}
	}

	pub fn mumble_link_identity_ptr(&self) -> Option<NonNull<MumbleIdentityUpdate>> {
		let id = self.mumble_identity.as_ref()?;
		#[cfg(todo)]
		if MumbleIdentity::update_is_empty(id) {
			return None
		}
		Some(MumbleIdentity::identity_update_ptr(id))
	}

	#[cfg(todo)]
	pub fn nexus_link_update_with<R, F: FnOnce(&mut NexusLink) -> R>(&self, f: F) -> R {
	}

	pub unsafe extern "C-unwind" fn addonapi_data_link_get(identifier: *const c_char) -> *const c_void {
		let id = cstr_opt(&identifier);

		match id {
			Some(id) if id == Self::DATA_LINK_MUMBLE => return MUMBLE_LINK.get()
				.and_then(|ml| match ml {
					&Some(ref ml) => Some(ml),
					None => None,
				}).map(|ml| ml.as_ptr())
				.unwrap_or(ptr::null()) as *const c_void,
			Some(id) if id == Self::DATA_LINK_MUMBLE_IDENTITY => return NexusHost::lock_read().mumble_link_identity_ptr()
				.as_ref()
				.map(|ml| ml.as_ptr() as *const MumbleIdentityUpdate)
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
