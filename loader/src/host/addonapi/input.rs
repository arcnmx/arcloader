use nexus::keybind::{Keybind, RawKeybindHandler, RawKeybindHandlerOld};
use crate::{
	host::addonapi::{NexusAddonCache, NexusHost},
	util::ffi::cstr_opt,
};
use std::{borrow::{Borrow, Cow}, collections::{BTreeMap, BTreeSet, HashMap}, ffi::{c_char, CStr, CString}, sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard}};

pub struct InputBinds {
	pub binds: BTreeMap<CString, BTreeSet<InputRegistration<'static>>>,
}

pub static INPUT_BINDS: RwLock<InputBinds> = RwLock::new(InputBinds::new());

impl InputBinds {
	pub const fn new() -> Self {
		Self {
			binds: BTreeMap::new(),
		}
	}

	pub fn register<I, K>(&mut self, id: I, callback: RawKeybindHandler, keybind: K) where
		I: Into<CString>,
		K: Into<CString>,
	{
		let registration = InputRegistration {
			id: Cow::Owned(id.into()),
			callback,
		};
		let binds = self.binds.entry(keybind.into())
			.or_insert(Default::default());
		binds.insert(registration);
	}

	pub fn lock_read() -> RwLockReadGuard<'static, Self> {
		INPUT_BINDS.read()
			.unwrap_or_else(|e| e.into_inner())
	}

	pub fn lock_write() -> RwLockWriteGuard<'static, Self> {
		INPUT_BINDS.write()
			.unwrap_or_else(|e| e.into_inner())
	}
}

#[derive(Debug, Clone)]
pub struct InputRegistration<'i> {
	pub id: Cow<'i, CStr>,
	pub callback: RawKeybindHandler,
}

impl<'r> PartialOrd<InputRegistration<'r>> for InputRegistration<'_> {
	fn partial_cmp(&self, other: &InputRegistration<'r>) -> Option<std::cmp::Ordering> {
		self.id.partial_cmp(&other.id)
	}
}

impl<'l> Ord for InputRegistration<'l> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.id.cmp(&other.id)
	}
}

impl<'r> PartialEq<InputRegistration<'r>> for InputRegistration<'_> {
	fn eq(&self, other: &InputRegistration<'r>) -> bool {
		self.id.eq(&other.id)
	}
}

impl Eq for InputRegistration<'_> {}

impl<'i> Borrow<CStr> for InputRegistration<'i> {
	fn borrow(&self) -> &CStr {
		&self.id
	}
}

unsafe impl Sync for InputRegistration<'_> {}
unsafe impl Send for InputRegistration<'_> {}

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_string(identifier: *const c_char, keybind_handler: RawKeybindHandler, keybind: *const c_char) {
		let id = cstr_opt(&identifier);
		let keybind = cstr_opt(&keybind);

		addonapi_stub!(input_binds::register_with_string("{:?}, {:?}, {:?}", id, keybind_handler, keybind));

		let id = match id {
			Some(id) => id,
			None => {
				error!("ID required for keybind {keybind:?} handler {keybind_handler:?}");
				return
			},
		};

		if let Some(kb) = keybind {
			InputBinds::lock_write()
				.register(id, keybind_handler, kb);
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_struct(identifier: *const c_char, keybind_handler: RawKeybindHandler, keybind: Keybind) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(input_binds::register_with_struct("{:?}, {:?}, {:?}", id, keybind_handler, keybind) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_deregister(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(input_binds::deregister("{:?}", id));

		let id = match id {
			Some(id) => id,
			None => {
				error!("ID required to unregister keybind");
				return
			},
		};

		let mut input_binds = InputBinds::lock_write();
		for (_, regs) in &mut input_binds.binds {
			regs.remove(id);
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_invoke(identifier: *const c_char, is_release: bool) {
		let id = cstr_opt(&identifier);

		let id = match id {
			Some(id) => id,
			None => {
				error!("ID required to invoke keybind");
				return
			},
		};

		addonapi_stub!(input_binds::invoke("{:?}, {:?}", id, is_release));

		let callback = {
			let mut callback = None;
			let input_binds = InputBinds::lock_read();
			for (_, regs) in &input_binds.binds {
				if let Some(reg) = regs.get(id) {
					callback = Some(reg.callback);
					break
				}
			}
			callback
		};
		match callback {
			Some(cb) => cb(id.as_ptr(), is_release),
			None => {
				warn!("cannot find keybind {id:?} to invoke");
			},
		}
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
