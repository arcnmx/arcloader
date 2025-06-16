use std::{
	ffi::OsString,
	hash::{DefaultHasher, Hash, Hasher},
	os::windows::ffi::OsStringExt,
	ptr::{self, NonNull},
};
use gw2_mumble::{Identity, MumblePtr};
use arcffi::{nonnull_ref_unchecked, wide::WideUtf8Reader, cstr::CStrPtr16};

#[cfg(feature = "nexus")]
pub use nexus::event::MumbleIdentityUpdate as ImpIdentity;

#[cfg(feature = "nexus")]
pub use self::ImpIdentity as NexusIdentityUpdate;
#[cfg(not(feature = "nexus"))]
pub type NexusIdentityUpdate = compile_error!("TODO: struct NexusIdentityUpdate");

#[derive(Clone)]
pub struct MumbleIdentity {
	pub hash: u64,
	len: usize,
	pub identity_data: [u16; Self::ID_DATA_LEN + 1],
	pub identity: Box<NexusIdentityUpdate>,
}

impl MumbleIdentity {
	pub const ID_DATA_LEN: usize = 256;
	#[cfg(feature = "nexus")]
	pub const ID_EMPTY: NexusIdentityUpdate = unsafe {
		::core::mem::MaybeUninit::zeroed().assume_init()
	};

	pub fn new() -> Self {
		Self {
			hash: 0,
			len: 0,
			identity_data: [0u16; Self::ID_DATA_LEN + 1],
			identity: Box::new(Self::ID_EMPTY),
		}
	}

	pub fn identity_update_ptr(this: *const Self) -> NonNull<NexusIdentityUpdate> {
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
		let identity = unsafe {
			CStrPtr16::new(nonnull_ref_unchecked(identity as *const u16))
		};
		let identity = identity.as_data();
		if identity.len() >= Self::ID_DATA_LEN {
			#[cfg(feature = "log")] {
				log::error!("identity string unterminated");
			}
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
					#[cfg(all(feature = "log", debug_assertions))] {
						log::debug!("mumble identity: {mli:#?}");
					}
					Some(mli)
				},
				Err(e) => {
					if e.is_io() || e.is_eof() {
						// TODO: mark string encoding invalid?
					}
					#[cfg(feature = "log")] {
						log::error!("Failed to parse mumble link identity: {e}");
						log::debug!("unsupported data: {:?}", self.identity_string());
					}
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

	pub fn update_from_parsed(id: &Identity) -> NexusIdentityUpdate {
		let mut name = [0u8; 20];
		let name_len = id.name.len().min(name.len());
		name[..name_len].copy_from_slice(id.name[..].as_bytes());
		NexusIdentityUpdate {
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

	pub fn update_is_empty(id: &NexusIdentityUpdate) -> bool {
		id.name[0] == 0 && id.world_id == 0 && id.fov == 0.0
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
}
