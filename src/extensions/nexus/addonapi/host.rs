use std::{collections::BTreeMap, ffi::{CStr, c_void}, mem::{transmute, MaybeUninit}, sync::{Arc, LazyLock, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError}};

use nexus::{data_link::NexusLink, event::MumbleIdentityUpdate, gui::RenderType, imgui::{self, Ui}};
use windows::{core::Owned, Win32::Foundation::{ERROR_NOT_FOUND, HMODULE}};

use crate::{
	extensions::nexus::{
		addonapi::data_link::{MumbleIdentity, MUMBLE_LINK},
		NexusAddon, NexusAddonCache
	},
	util::{nexus::NexusId, win::{get_module_from_ptr, WinResult, WinError}},
};

pub static NEXUS_HOST: RwLock<NexusHost> = RwLock::new(NexusHost::empty());

pub struct NexusHost {
	pub addons: BTreeMap<NexusId, Arc<NexusAddon>>,
	pub fallback_cache: LazyLock<Arc<RwLock<NexusAddonCache>>>,
	pub nexus_link: Mutex<NexusLink>,
	pub mumble_identity: Option<MumbleIdentity>,
}

impl NexusHost {
	pub const fn empty() -> Self {
		Self {
			addons: BTreeMap::new(),
			fallback_cache: LazyLock::new(|| Default::default()),
			nexus_link: Mutex::new(unsafe {
				MaybeUninit::zeroed().assume_init()
			}),
			mumble_identity: None,
		}
	}

	pub fn init() {
		#[cfg(todo)] {
		Self::cache_write_with(ptr::null(), |mut cache| {
			cache.shared_data.insert(Self::DATA_LINK_MUMBLE, vec![0u8; size_of::<nexus::data_link::MumbleLink>()].into_boxed_slice());
			cache.shared_data.insert(Self::DATA_LINK_NEXUS, vec![0u8; size_of::<NexusLink>()].into_boxed_slice());
		})
		}

		let mut host = Self::lock_write();
		{
			let nl = host.nexus_link.get_mut()
				.unwrap_or_else(|e| e.into_inner());
			unsafe {
				let ui = arcdps::__macro::ui();
				if let Some(&font) = ui.fonts().fonts().first() {
					let font: imgui::FontId = font;
					nl.font = transmute(font);
					nl.font_ui = transmute(font);
					nl.font_big = transmute(font);
				}
			}
		}

		let ml = gw2_mumble::MumbleLink::new();
		#[cfg(feature = "log")]
		if let Err(e) = &ml {
			error!("failed to open MumbleLink: {e}");
		}
		let ml = ml.ok();
		if let Some(ml) = &ml {
			let id = host.mumble_identity.insert(MumbleIdentity::new());
			id.update_identity_str(&ml.as_mumble_ptr());
			id.update_identity_from_str();
		}
		MUMBLE_LINK.set(ml);
	}

	pub fn unload() {
		let mut host = Self::lock_write();
		host.shutdown();
	}

	pub fn lock_read() -> RwLockReadGuard<'static, Self> {
		NEXUS_HOST.read().unwrap_or_else(|e| e.into_inner())
	}

	pub fn lock_write() -> RwLockWriteGuard<'static, Self> {
		match NEXUS_HOST.write() {
			Ok(lock) => lock,
			Err(e) => {
				// TODO...
				e.into_inner()
			},
		}
	}

	pub fn imgui_present(ui: &Ui, not_charsel_or_loading: bool) {
		#[cfg(todo)]
		if not_charsel_or_loading {
			// tracking for AddonFlag::OnlyLoadDuringGameLaunchSequence
			self.load_missed = true;
		}

		let mli_update = if let Ok(mut host) = NEXUS_HOST.try_write() {
			#[cfg(todo)]
			let nl: *const Mutex<NexusLink> = &*host.nexus_link;
			#[cfg(todo)]
			let nl = unsafe { (*(nl as *mut Mutex<NexusLink>)).get_mut() }
				.unwrap_or_else(|e| e.into_inner());
			{
				let mut nl = host.nexus_link.lock()
				.unwrap_or_else(|e| e.into_inner());
				nl.is_gameplay = not_charsel_or_loading;
				let [w, h] = ui.io().display_size;
				nl.width = w as u32;
				nl.height = h as u32;
				let [x, y] = ui.io().display_framebuffer_scale;
				nl.scaling = 1.0;
				//nl.is_moving = ?;
				//nl.is_camera_moving = ?;
				//nl.scaling = x / y;
				let font: imgui::FontId = ui.current_font().id();
				unsafe {
					nl.font = transmute(font);
					nl.font_ui = transmute(font);
					nl.font_big = transmute(font);
				}
			}

			host.update_mumble_link_identity()
		} else { None };
		if let Some(mli) = mli_update {
			Self::event_broadcast(Self::EV_MUMBLE_IDENTITY_UPDATED, mli.as_ptr() as *const MumbleIdentityUpdate as *const _);
		}
		Self::render(RenderType::PreRender);
		Self::render(RenderType::Render);
		Self::render(RenderType::PostRender);
	}

	pub fn render(ty: RenderType) {
		let callbacks: Vec<_> = {
			let host = Self::lock_read();
			host.addons.values()
				.filter(|a| a.is_loaded())
				.flat_map(|addon| {
					NexusAddonCache::lock_read(&addon.cache).renderers.get(&ty).cloned().into_iter().flatten()
				})
				.collect()
		};

		for cb in callbacks {
			cb()
		}
	}

	pub fn imgui_options_end(_ui: &Ui) {
		Self::render(RenderType::OptionsRender);
	}

	pub fn shutdown(&mut self) {
		for addon in self.addons.values_mut() {
			if addon.can_hotload() {
				let _res = addon.unload();
				#[cfg(feature = "log")]
				if let Err(e) = _res {
					error!("{addon} failed to unload at shutdown: {e}");
				}
			}
		}

		// TODO: keep non-hotpluggable ones alive?
		self.addons.clear();
	}

	pub fn enumerate_addon(module: Owned<HMODULE>) -> WinResult<NexusId> {
		let mut host = Self::lock_write();
		let addon = Arc::new(NexusAddon::with_module(&host, module)?);
		let sig = addon.signature;
		host.addons.insert(sig, addon.clone());
		Ok(sig)
	}

	pub fn load_addon(sig: i32) -> WinResult<()> {
		let addon = {
			let host = Self::lock_read();
			host.addons.get(&sig).cloned()
		}.ok_or_else(|| WinError::new(ERROR_NOT_FOUND.to_hresult(), "addon not enumerated"))?;

		let res = addon.load();

		if let Err(_e) = &res {
			#[cfg(feature = "log")] {
				error!("{addon} failed to load: {_e}");
			}

			Self::lock_write().addons.remove(&addon.signature);
		} else {
			Self::event_broadcast(Self::EV_ADDON_LOADED, &sig as *const _ as *const _);
			#[cfg(unnecessary)]
			match &*MUMBLE_LINK_IDENTITY.lock().unwrap() {
				Some(mli) =>
					Self::event_broadcast(Self::EV_MUMBLE_IDENTITY_UPDATED, mli as *const _ as *const _),
				_ => (),
			}
		}

		res
	}

	pub fn event_broadcast(key: &CStr, data: *const c_void) {
		let interest: Vec<_> = {
			let host = Self::lock_read();
			let x = NexusAddonCache::lock_read(&host.fallback_cache).event_handlers.get(key).cloned()
				.into_iter().flatten()
				.chain(host.addons.values().flat_map(|a| NexusAddonCache::lock_read(&a.cache)
					.event_handlers.get(key).cloned().into_iter().flatten()
				)).collect();
			x
		};
		for cb in interest {
			cb(data);
		}
	}

	pub fn unload_addon(sig: NexusId) -> WinResult<()> {
		let addon = {
			let host = Self::lock_read();
			host.addons.get(&sig).cloned()
		}.ok_or_else(|| WinError::new(ERROR_NOT_FOUND.to_hresult(), "addon not enumerated"))?;

		let res = addon.unload();

		if let Err(_e) = &res {
			#[cfg(feature = "log")] {
				error!("{addon} failed to unload: {_e}");
			}
		}

		Self::lock_write().addons.remove(&addon.signature);

		Ok(())
	}

	pub fn addon_for_ptr(&self, p: *const ()) -> Option<&Arc<NexusAddon>> {
		get_module_from_ptr(p as *const _)
			.ok().flatten()
			.and_then(|module| self.addons.values().find(|a| *a.module == module))
	}

	pub fn cache_for(&self, p: *const ()) -> &Arc<RwLock<NexusAddonCache>> {
		match self.addon_for_ptr(p) {
			Some(addon) => &addon.cache,
			None => &self.fallback_cache,
		}
	}

	#[cfg(todo)]
	pub fn cache_fallback(read: RwLockReadGuard<Self>) -> MappedRwLockReadGuard<NexusAddonCache> {
		read.map(|host| host.cache)
	}

	pub fn cache_rw_for(p: *const ()) -> Arc<RwLock<NexusAddonCache>> {
		Self::lock_read()
			.cache_for(p)
			.clone()
	}

	pub fn cache_read_with<R, F: FnOnce(RwLockReadGuard<NexusAddonCache>) -> R>(p: *const (), f: F) -> R {
		let cache = {
			let host = Self::lock_read();
			let cache = host.cache_for(p);

			let read_lock = match cache.try_read() {
				Ok(r) => Some(r),
				Err(TryLockError::Poisoned(r)) => Some(r.into_inner()),
				Err(TryLockError::WouldBlock) => None,
			};

			match read_lock {
				Some(r) => return f(r),
				None => cache.clone(),
			}
		};
		f(NexusAddonCache::lock_read(&cache))
	}

	pub fn cache_write_with<R, F: FnOnce(RwLockWriteGuard<NexusAddonCache>) -> R>(p: *const (), f: F) -> R {
		let cache = {
			let host = Self::lock_write();
			let cache = host.cache_for(p);

			let write_lock = match cache.try_write() {
				Ok(r) => Some(r),
				Err(TryLockError::Poisoned(r)) => Some(r.into_inner()),
				Err(TryLockError::WouldBlock) => None,
			};

			match write_lock {
				Some(r) => return f(r),
				None => cache.clone(),
			}
		};
		f(NexusAddonCache::lock_write(&cache))
	}
}
