use gw2_mumble::{Identity, MumbleLink, MumblePtr};
use nexus::event::MumbleIdentityUpdate;
use crate::host::addonapi::{NexusHost, NEXUS_HOST};
use arcffi::{
	wide::WideUtf8Reader,
	nonnull_bytes, nonnull_const, nonnull_ref_unchecked,
	CStrPtr16,
};
use std::{ffi::OsString, hash::{DefaultHasher, Hash, Hasher}, mem::{transmute, MaybeUninit}, num::NonZeroI32, os::windows::ffi::OsStringExt, ptr::{self, NonNull}, sync::LazyLock};

use super::DataLinkShare;

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
		let identity = unsafe { CStrPtr16::new(nonnull_ref_unchecked(identity as *const u16)) };
		let identity = identity.as_data();
		if identity.len() >= Self::ID_DATA_LEN {
			error!("identity string unterminated");
			return &[]
		}

		unsafe { &*(identity as *const [u16]) }
	}

	pub fn update_identity_str(&mut self, ml: &MumblePtr) -> bool {
		let identity = Self::borrow_identity(ml);
		let mut hasher = DefaultHasher::new();
		identity.hash(&mut hasher);
		let hash = hasher.finish();
		if self.hash == hash {
			return false
		}
		unsafe {
			ptr::copy_nonoverlapping(identity.as_ptr(), self.identity_data.as_mut_ptr(), identity.len());
		}
		self.hash = hash;
		self.len = identity.len();

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

	pub fn init() {
		let ml = match MumbleLinkProvider::get_ptr() {
			None => {
				debug!("MumbleIdentity requires MumbleLink");
				return
			},
			Some(ml) => ml,
		};
		let mut mli = Self::new();
		let _ = mli.update(&ml);
		#[cfg(todo)] {
			let mli = Self::get_ptr();
			NexusHost::register_data_link(NexusHost::DATA_LINK_MUMBLE_IDENTITY, mli);
		}
		NexusHost::lock_write().mumble_identity = Some(mli);

		unsafe {
			NexusHost::addonapi_event_subscribe(NexusHost::EV_ADDON_LOADED.as_ptr(), transmute(Self::ev_addon_loaded as unsafe extern "C-unwind" fn(_)));
		}
	}

	pub fn update(&mut self, ml: &MumblePtr) -> bool {
		if !self.update_identity_str(&ml) {
			return false
		}
		match self.update_identity_from_str() {
			true => true,
			_ => false,
		}
	}

	pub fn try_update() {
		let ml = match MumbleLinkProvider::get_ptr() {
			Some(ml) => ml,
			None => return,
		};
		let mli_update = {
			let mut host = NEXUS_HOST.try_write().ok();
			let mli = host.as_mut()
				.and_then(|host| host.mumble_identity.as_mut());
			let mli = match mli {
				Some(mli) => mli,
				_ => return,
			};
			match mli.update(&ml) {
				#[cfg(todo)]
				true if MumbleIdentity::update_is_empty(id) => return,
				true => Self::identity_update_ptr(mli),
				_ => return,
			}
		};
		NexusHost::event_broadcast(NexusHost::EV_MUMBLE_IDENTITY_UPDATED, mli_update.as_ptr() as *const MumbleIdentityUpdate as *const _);
	}

	pub unsafe extern "C-unwind" fn ev_addon_loaded(sig: *const i32) {
		let sig = nonnull_const(sig)
			.map(|sig| *sig.as_ptr());
		let sig = match sig.and_then(NonZeroI32::new) {
			Some(sig) => sig.get(),
			None => return,
		};
		let mli = match NEXUS_HOST.try_read().ok().and_then(|host| host.mumble_link_identity_ptr()) {
			Some(mli) => mli,
			None => return,
		};
		NexusHost::addonapi_event_raise_targeted(sig, NexusHost::EV_MUMBLE_IDENTITY_UPDATED.as_ptr(), mli.as_ptr() as *const MumbleIdentityUpdate as *const _);
	}
}

unsafe impl DataLinkShare for MumbleIdentity {
	unsafe fn get_data_share_pinned(&mut self) -> NonNull<[u8]> {
		let p = Self::identity_update_ptr(self);
		nonnull_bytes(p)
	}
}

pub static MUMBLE_LINK: LazyLock<MumbleLinkProvider> = LazyLock::new(MumbleLinkProvider::new);

pub struct MumbleLinkProvider {
	ml: Option<MumbleLink>,
}

impl MumbleLinkProvider {
	pub fn new() -> Self {
		let ml = MumbleLink::new();

		if let Err(_e) = &ml {
			error!("failed to open mumble link: {_e:?}");
		}
		let ml = ml.ok();

		Self {
			ml,
		}
	}

	pub fn mumble_ptr(&self) -> Option<MumblePtr> {
		self.ml.as_ref()
			.map(|ml| ml.as_mumble_ptr())
	}

	pub fn get_ptr() -> Option<MumblePtr> {
		MUMBLE_LINK.mumble_ptr()
	}

	pub fn init() {
		#[cfg(todo)] {
			let ml = Self::get_ptr();
			NexusHost::register_data_link(NexusHost::DATA_LINK_MUMBLE, ml);
		}
	}
}

unsafe impl DataLinkShare for MumbleLinkProvider {
	unsafe fn get_data_share_pinned(&mut self) -> NonNull<[u8]> {
		let p = self.mumble_ptr()
			.expect("ML data share registration must be valid");
		nonnull_bytes(p.as_non_null())
	}
}

impl NexusHost {
	pub fn mumble_link_identity_ptr(&self) -> Option<NonNull<MumbleIdentityUpdate>> {
		let id = self.mumble_identity.as_ref()?;
		#[cfg(todo)]
		if MumbleIdentity::update_is_empty(id) {
			return None
		}
		Some(MumbleIdentity::identity_update_ptr(id))
	}
}
