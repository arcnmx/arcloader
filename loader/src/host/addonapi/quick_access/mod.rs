use crate::{
	host::addonapi::NexusHost,
	util::ffi::cstr_opt,
};
use nexus::gui::RawGuiRender;
use std::{collections::HashMap, ffi::{c_char, CStr, CString}, ptr, sync::{Arc, LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak}, time::Duration};

pub mod ui;

#[derive(Debug, Clone, Default)]
pub struct QuickAccessMenu {
	pub items: HashMap<Arc<CStr>, Arc<QuickAccessItem>>,
	pub context_items: HashMap<Arc<CStr>, Arc<QuickAccessContextItem>>,
	pub notifications: HashMap<Arc<CStr>, u32>,
}

pub static QUICK_ACCESS: LazyLock<RwLock<QuickAccessMenu>> = LazyLock::new(Default::default);

impl QuickAccessMenu {
	pub const QA_MENU: &'static CStr = cstr!("0_QA_MENU");

	pub fn init() {
		// TODO: lazy init? also register render callback for UI

		let item = Self::default_item();
		{
			let mut menu = Self::lock_write();
			if let Some(item) = item {
				menu.items.insert(item.id.clone(), Arc::new(item));
			}
		}
	}

	pub fn default_item() -> Option<QuickAccessItem> {
		let keybind_id = {
			debug!("TODO: {:?} keybind", Self::QA_MENU);
			None
		};
		let texture = {
			debug!("TODO: {:?} texture", Self::QA_MENU);
			Some(super::texture::TextureCache::TEXTURE_FALLBACK_ID.into())
		};
		Some(QuickAccessItem {
			id: Arc::from(Self::QA_MENU),
			tooltip: Some(cstr!("arcloader").into()),
			keybind_id,
			texture,
			// TODO
			texture_hover: None,
		})
	}

	#[cfg(todo)]
	pub fn context_items_for_target_id<'a: 'i, 'i>(&'a self, target: &'i CStr) -> impl Iterator<Item = &'a Arc<QuickAccessContextItem>> + 'a + 'i where
		// come on .-.
		'i: 'a,
	{
		self.context_items.values()
			.filter(move |&citem| *citem.id == *target)
	}

	pub fn context_items_for_target(&self, target: *const QuickAccessItem) -> impl Iterator<Item = &Arc<QuickAccessContextItem>> {
		self.context_items.values()
			.filter(move |citem| Weak::as_ptr(&citem.target) == target)
	}

	/// TODO
	pub fn settings() -> QuickAccessSettings {
		QuickAccessSettings::default()
	}

	#[cfg(todo)]
	pub fn lock_read() -> RwLockReadGuard<'static, Self> {
		QUICK_ACCESS.read()
			.unwrap_or_else(|e| e.into_inner())
	}

	pub fn lock_write() -> RwLockWriteGuard<'static, Self> {
		QUICK_ACCESS.write()
			.unwrap_or_else(|e| e.into_inner())
	}
}

#[derive(Debug, Clone)]
pub struct QuickAccessItem {
	pub id: Arc<CStr>,
	pub texture: Option<CString>,
	pub texture_hover: Option<CString>,
	pub keybind_id: Option<CString>,
	pub tooltip: Option<CString>,
}

impl QuickAccessItem {
	pub fn try_clear_notifications(&self) -> Result<bool, ()> {
		let mut menu = match QUICK_ACCESS.try_write() {
			Ok(m) => m,
			_ => return Err(()),
		};
		let prev = menu.notifications.remove(&self.id);
		Ok(prev.is_some())
	}

	pub fn select(&self, released: bool) -> Result<bool, ()> {
		let keybind = match &self.keybind_id {
			Some(id) => id,
			None => return Ok(false),
		};
		unsafe {
			NexusHost::addonapi_input_binds_invoke(keybind.as_ptr(), released);
		}
		Ok(true)
	}
}

#[derive(Debug, Clone)]
pub struct QuickAccessContextItem {
	pub id: Arc<CStr>,
	pub target: Weak<QuickAccessItem>,
	pub render: Option<RawGuiRender>,
}

impl QuickAccessContextItem {
}

#[derive(Debug, Clone, Copy)]
pub enum QuickAccessItemAction {
	Selected {
		pressed: bool,
	},
	ContextMenu {
		pressed: Option<bool>,
	},
	Hovered,
}

#[derive(Debug, Clone)]
pub struct QuickAccessSettings {
	#[cfg(todo)]
	pub x_offset: f32,
	#[cfg(todo)]
	pub y_row: f32,
	pub tooltip_delay: Duration,
	#[cfg(todo)]
	pub alignment: (),
	#[cfg(todo)]
	pub hidden: HashSet<CString>,
}

impl Default for QuickAccessSettings {
	fn default() -> Self {
		Self {
			tooltip_delay: Duration::from_millis(650),
		}
	}
}

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_quick_access_add(identifier: *const c_char, texture_identifier: *const c_char, texture_hover_identifier: *const c_char, keybind_identifier: *const c_char, tooltip_text: *const c_char) {
		let id = cstr_opt(&identifier);
		let texture_id = cstr_opt(&texture_identifier);
		let texture_hover_id = cstr_opt(&texture_hover_identifier);
		let keybind_id = cstr_opt(&keybind_identifier);
		let tooltip = cstr_opt(&tooltip_text);

		addonapi_stub!(quick_access::add("{:?}, {:?}, {:?}, {:?}, {:?}", id, texture_id, texture_hover_id, keybind_id, tooltip));

		let id: Arc<CStr> = match id {
			Some(id) => id.into(),
			None => {
				error!("quick access identifier required");
				return
			},
		};

		let item = QuickAccessItem {
			id: id.clone(),
			texture: texture_id.map(ToOwned::to_owned),
			texture_hover: texture_hover_id.map(ToOwned::to_owned),
			keybind_id: keybind_id.map(ToOwned::to_owned),
			tooltip: tooltip.map(ToOwned::to_owned),
		};
		let prev = {
			let mut menu = QuickAccessMenu::lock_write();
			menu.items.insert(id, Arc::new(item))
		};
		ui::QuickAccessMenuUi::mark_dirty();

		if let Some(_prev) = prev {
			warn!("quick access item {:?} replaced", _prev);
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_remove(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(quick_access::remove("{:?}", id));
		let id = match id {
			Some(id) => id,
			None => {
				error!("quick access identifier required");
				return
			},
		};

		let prev = {
			let mut menu = QuickAccessMenu::lock_write();
			menu.items.remove(id)
		};
		match prev {
			Some(prev) => {
				ui::QuickAccessMenuUi::mark_dirty();
				debug!("quick access item {:?} removed", prev);
			},
			None => {
				warn!("cannot find quick access item {:?} to remove", id);
			},
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_notify(identifier: *const c_char) {
		let id = cstr_opt(&identifier);
		let id = match id {
			Some(id) => id,
			None => {
				error!("quick access identifier required");
				return
			},
		};

		let prev = {
			let mut menu = QuickAccessMenu::lock_write();
			let key = match menu.items.get_key_value(id) {
				None => None,
				Some((k, _)) => Some(k.clone()),
			};
			key.map(|key| {
				let e = menu.notifications.entry(key);
				let notifications = e.or_insert(0);
				let prev = *notifications;
				*notifications = prev.saturating_add(1);
				prev
			})
		};
		match prev {
			Some(prev) => {
				// TODO: this makes more sense as an atomic, probably even just on the arc item?
				ui::QuickAccessMenuUi::mark_dirty();
				debug!("quick access item {id:?} notified {}+1 times", prev);
			},
			None => {
				warn!("cannot find quick access item {id:?} to notify");
			},
		}

		addonapi_stub!(quick_access::notify("{:?}", id) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_add_context_menu(identifier: *const c_char, target_identifier: *const c_char, shortcut_render_callback: RawGuiRender)  {
		let id = cstr_opt(&identifier);
		let target_id = cstr_opt(&target_identifier);
		addonapi_stub!(quick_access::add_context_menu("{:?}, {:?}, {:?}", id, target_id, shortcut_render_callback));

		let id = match id {
			Some(id) => id,
			None => {
				error!("quick access context identifier required");
				return
			},
		};

		let target_id = target_id.unwrap_or(QuickAccessMenu::QA_MENU);
		let prev = {
			let mut menu = QuickAccessMenu::lock_write();

			let target = match menu.items.get(target_id) {
				Some(target) => Arc::downgrade(&target),
				None => {
					warn!("quick access item {:?} not found", target_id);
					// TODO: return instead?
					Weak::new()
				},
			};
			let id: Arc<CStr> = id.into();
			let item = QuickAccessContextItem {
				id: id.clone(),
				target,
				render: Some(shortcut_render_callback),
			};
			menu.context_items.insert(id, Arc::new(item))
		};
		ui::QuickAccessMenuUi::mark_dirty();

		if let Some(_prev) = prev {
			warn!("quick access context item {:?} replaced", _prev);
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_add_context_menu_v2(identifier: *const c_char, shortcut_render_callback: RawGuiRender)  {
		Self::addonapi_quick_access_add_context_menu(identifier, ptr::null_mut(), shortcut_render_callback)
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_remove_context_menu(identifier: *const c_char) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(quick_access::remove_context_menu("{:?}", id));

		let id = match id {
			Some(id) => id,
			None => {
				error!("quick access context identifier required");
				return
			},
		};

		let prev = {
			let mut menu = QuickAccessMenu::lock_write();
			menu.context_items.remove(id)
		};
		match prev {
			Some(prev) => {
				ui::QuickAccessMenuUi::mark_dirty();
				debug!("quick access context item {:?} replaced", prev);
			},
			None => {
				warn!("cannot find quick access context item {:?} to remove", id);
			},
		}
	}
}
