use std::{collections::{HashMap, HashSet}, ffi::{c_char, CString}, fmt, ops::Deref, sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use nexus::{event::RawEventConsumeUnknown, gui::{RawGuiRender, RenderType}};
use windows::{core::{Error as WinError, Owned}, Win32::Foundation::{ERROR_CALL_NOT_IMPLEMENTED, HMODULE}};
use crate::host::addonapi::{NexusHost, AddonApiV};
use crate::util::{nexus::{get_addon_def, AddonDesc}, win::WinResult};

pub struct NexusAddon {
	module: Owned<HMODULE>,
	desc: *const AddonDesc,
	loaded: AtomicBool,
	pub api: AddonApiV,
	pub cache: Arc<RwLock<NexusAddonCache>>,
}

impl NexusAddon {
	pub fn with_module(host: &NexusHost, module: Owned<HMODULE>) -> WinResult<Self> {
		let (desc, api_version) = get_addon_def(&module)?;

		let api = host.api_with_version(api_version)?;
		// TODO: nope!

		let addon = Self {
			desc,
			module,
			loaded: AtomicBool::new(false),
			api,
			cache: Default::default(),
		};

		Ok(addon)
	}

	pub fn load(&self) -> WinResult<()> {
		if self.is_loaded() {
			return Ok(())
		}

		let () = unsafe {
			(self.def().load)(self.api.as_ptr())
		};
		// TODO: how is failure signalled?
		self.loaded.store(true, Ordering::SeqCst);
		Ok(())
	}

	pub fn unload(&self) -> WinResult<()> {
		if !self.is_loaded() {
			return Ok(())
		}

		match self.def().unload {
			_ if !self.desc().can_hotload() =>
				Err(WinError::new(ERROR_CALL_NOT_IMPLEMENTED.to_hresult(), "hotloading disabled")),
			None =>
				Err(WinError::new(ERROR_CALL_NOT_IMPLEMENTED.to_hresult(), "unload required")),
			Some(unload) => unsafe {
				self.loaded.store(false, Ordering::SeqCst);
				unload();
				Ok(())
			},
		}
	}

	pub fn is_loaded(&self) -> bool {
		self.loaded.load(Ordering::SeqCst)
	}

	pub fn module(&self) -> HMODULE {
		*self.module
	}

	#[inline]
	pub fn desc(&self) -> &AddonDesc {
		unsafe { &*self.desc }
	}
}

impl Deref for NexusAddon {
	type Target = AddonDesc;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.desc()
	}
}

impl fmt::Display for NexusAddon {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = self.name().to_string_lossy();
		let version = self.version();
		write!(f, "{name} {version}")?;
		
		#[cfg(todo)]
		if let Some(author) = self.author() {
			let author = self.name().to_string_lossy();
			write!(f, " by {author}")?;
		}

		Ok(())
	}
}

impl fmt::Debug for NexusAddon {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut f = f.debug_struct("NexusAddon");
		f
			.field("addon", &format_args!("{}", self))
			.field("desc", &self.desc())
			.field("module", &self.module)
			.field("api", &format_args!("{}", self.api))
			.finish()
	}
}

#[derive(Debug, Default)]
pub struct NexusAddonCache {
	pub cstrings: HashMap<Arc<CString>, *const c_char>,
	pub event_handlers: HashMap<CString, HashSet<RawEventConsumeUnknown>>,
	pub renderers: HashMap<RenderType, HashSet<RawGuiRender>>,
	pub shared_data: HashMap<CString, Box<[u8]>>,
}

impl NexusAddonCache {
	pub fn lock_read(rw: &RwLock<Self>) -> RwLockReadGuard<Self> {
		rw.read().unwrap_or_else(|e| e.into_inner())
	}

	pub fn lock_write(rw: &RwLock<Self>) -> RwLockWriteGuard<Self> {
		rw.write().unwrap_or_else(|e| e.into_inner())
	}
}

unsafe impl Sync for NexusAddonCache {
}

unsafe impl Send for NexusAddonCache {
}
