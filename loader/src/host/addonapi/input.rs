use nexus::keybind::{Keybind as NexusKeybind, RawKeybindHandler, RawKeybindHandlerOld};
use windows::Win32::Foundation::ERROR_KEY_DOES_NOT_EXIST;
use crate::{
	host::addonapi::NexusHost,
	util::{ffi::cstr_opt, nexus::{InputCode, Keybind, NexusId}, win::{WinError, WinResult}},
};
use std::{collections::BTreeMap, ffi::{c_char, CStr, CString}, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}};

pub struct InputBinds {
	/// TODO: a plain set plus cache rebuilt whenever binds changes would be better tbh...
	pub binds: BTreeMap<Option<InputCode>, Vec<InputRegistration>>,
}

pub static INPUT_BINDS: RwLock<InputBinds> = RwLock::new(InputBinds::new());

impl InputBinds {
	pub const fn new() -> Self {
		Self {
			binds: BTreeMap::new(),
		}
	}

	pub fn register<I>(&mut self, id: I, callback: RawKeybindHandler, bind: Option<Keybind>) -> WinResult<()> where
		I: Into<CString>,
	{
		let (bind, code) = match bind {
			None => (Keybind::default(), None),
			Some(bind) => {
				let code = bind.code()
					.map_err(|()| WinError::new(ERROR_KEY_DOES_NOT_EXIST.to_hresult(), format!("unrecognized keybind {bind:?}")))?;
				(bind, code)
			},
		};
		let registration = InputRegistration {
			id: id.into(),
			callback,
			bind,
		};
		let binds = self.binds.entry(code)
			.or_insert(Default::default());
		binds.push(registration);

		Ok(())
	}

	pub fn lock_read() -> RwLockReadGuard<'static, Self> {
		INPUT_BINDS.read()
			.unwrap_or_else(|e| e.into_inner())
	}

	pub fn lock_write() -> RwLockWriteGuard<'static, Self> {
		INPUT_BINDS.write()
			.unwrap_or_else(|e| e.into_inner())
	}

	pub fn binds_for_addon<'i>(&'i self, host: &'i NexusHost, sig: NexusId) -> impl Iterator<Item = &'i InputRegistration> + 'i {
		self.binds.values()
			.flatten()
			.filter(move |reg| host.addon_for_ptr(reg.callback as *const _)
				.map(|addon| addon.signature == sig)
				.unwrap_or(false)
			)
	}

	pub fn find_id<'i>(&'i self, id: &CStr) -> Option<&'i InputRegistration> {
		self.binds.values()
			.flatten()
			.find(move |reg| reg.id.as_c_str() == id)
	}
}

#[derive(Debug, Clone)]
pub struct InputRegistration {
	pub id: CString,
	pub bind: Keybind,
	pub callback: RawKeybindHandler,
}

#[cfg(todo)]
impl<'r> PartialOrd<InputRegistration<'r>> for InputRegistration<'_> {
	fn partial_cmp(&self, other: &InputRegistration<'r>) -> Option<std::cmp::Ordering> {
		self.id.partial_cmp(&other.id)
	}
}

#[cfg(todo)]
impl<'l> Ord for InputRegistration<'l> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.id.cmp(&other.id)
	}
}

#[cfg(todo)]
impl<'r> PartialEq<InputRegistration<'r>> for InputRegistration<'_> {
	fn eq(&self, other: &InputRegistration<'r>) -> bool {
		self.id.eq(&other.id)
	}
}

#[cfg(todo)]
impl Borrow<CStr> for InputRegistration {
	fn borrow(&self) -> &CStr {
		&self.id
	}
}

unsafe impl Sync for InputRegistration {}
unsafe impl Send for InputRegistration {}

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
			let bind = match kb.is_empty() {
				true => Ok(None),
				false => Keybind::try_from(kb).map(Some),
			};
			let res = bind.and_then(|bind| InputBinds::lock_write()
				.register(id, keybind_handler, bind)
			);
			if let Err(_e) = res {
				error!("keybind registration failed for {kb:?}: {_e}");
			}
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_struct(identifier: *const c_char, keybind_handler: RawKeybindHandler, keybind: NexusKeybind) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(input_binds::register_with_struct("{:?}, {:?}, {:?}", id, keybind_handler, keybind));

		let id = match id {
			Some(id) => id,
			None => {
				error!("ID required for keybind {keybind:?} handler {keybind_handler:?}");
				return
			},
		};

		let keybind = Keybind::from(keybind);
		let res = InputBinds::lock_write()
			.register(id, keybind_handler, Some(keybind));
		if let Err(_e) = res {
			error!("keybind registration failed for {keybind:?}: {_e}");
		}
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
			regs.retain(|reg| reg.id.as_c_str() != id);
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
				if let Some(reg) = regs.iter().find(|reg| reg.id.as_c_str() == id) {
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

	pub unsafe extern "C-unwind" fn addonapi_input_binds_register_with_struct_v2(identifier: *const c_char, keybind_handler: RawKeybindHandlerOld, keybind: NexusKeybind) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(input_binds::register_with_struct_v2("{:?}, {:?}, {:?}", id, keybind_handler, keybind) => ())
	}
}
