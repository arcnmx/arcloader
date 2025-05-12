use crate::{
	host::addonapi::{
		imgui::{self, MouseButton, StyleVar, Ui}, input::InputBinds, quick_access::{QuickAccessContextItem, QuickAccessItem, QuickAccessItemAction, QuickAccessMenu, QuickAccessSettings, QUICK_ACCESS}, NexusHost
	},
	ui::imgui_id_cstr,
	util::{ffi::nonnull_const, nexus::Keybind},
	RenderThread,
};
use nexus::{imgui::WindowFlags, texture::Texture as NexusTexture};
use std::{borrow::Cow, cell::RefCell, collections::BTreeSet, hash::{DefaultHasher, Hash, Hasher}, ptr, str, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};

#[derive(Debug, Clone, Default)]
pub struct QuickAccessMenuUi {
	pub draw_items: Vec<QuickAccessItemUi>,
}

static QA_UI_DIRTY: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct QuickAccessItemUi {
	desc: Arc<QuickAccessItem>,
	context: Vec<Arc<QuickAccessContextItem>>,
	texture: Result<Option<NexusTexture>, ()>,
	texture_hover: Result<Option<NexusTexture>, ()>,
	hover_start: Option<f64>,
	state: Option<QuickAccessItemAction>,
	tooltip: Option<Result<(), String>>,
	notifications: Option<u32>,
}

impl QuickAccessItemUi {
	pub fn with_item(desc: Arc<QuickAccessItem>) -> Self {
		Self {
			desc,
			context: Vec::new(),
			texture: Ok(None),
			texture_hover: Ok(None),
			hover_start: None,
			state: None,
			tooltip: None,
			notifications: None,
		}
	}

	pub fn texture_with_hover(&mut self, hovering: bool) -> Option<imgui::TextureId> {
		let (texture_storage, id) = match hovering {
			true if self.desc.texture_hover.is_some() => (&mut self.texture_hover, &self.desc.texture_hover),
			_ => (&mut self.texture, &self.desc.texture),
		};

		let texture = match (texture_storage, id) {
			(Ok(storage @ None), Some(id)) => unsafe {
				use crate::host::addonapi::texture::{TextureEntry, TextureCache};
				let texture = match NexusHost::texture_lookup_with(Some(id), |cache, entry, _id| Some(match entry {
					TextureEntry::Loaded(t) => &**t,
					TextureEntry::Failed => cache.fallback().map(TextureCache::addonapi_ptr_nn)?,
					TextureEntry::Upload { .. } => ptr::null(),
				})) {
					Err(..) => TextureCache::addonapi_fallback(),
					Ok(res) => res,
				};
				let texture =
					nonnull_const(texture)
					.map(|t| &*t.as_ptr());
				match texture {
					Some(t) => Some(&*storage.insert(t.clone())),
					None => None,
				}
			},
			(Ok(t), _) => t.as_ref(),
			(Err(()), _) => None,
		};

		match texture {
			None => None,
			Some(t) if t.resource.is_none() => None,
			Some(t) => Some(t.id()),
		}
	}

	pub fn draw(&mut self, ui: &Ui, settings: &QuickAccessSettings) {
		let mut action = self.state;
		let mut item_pos = None;
		match action {
			Some(QuickAccessItemAction::ContextMenu { pressed }) => match pressed {
				Some(true) if ui.is_mouse_released(MouseButton::Right) => {
					action = Some(QuickAccessItemAction::ContextMenu { pressed: Some(false) });
					if !self.context.is_empty() {
						ui.open_popup("quick_access_context");
					}
				},
				#[cfg(todo)]
				false if clicked_elsewhere => self.state = None,
				_ => (),
			},
			_ => {
				item_pos = Some(ui.cursor_pos());
				action = self.desc.predraw(ui, settings);
			},
		};

		let hovered_visual = action
			.map(|a| a.is_visual_hover())
			.unwrap_or(false);
		let texture = self.texture_with_hover(hovered_visual);

		if let Some(texture) = texture {
			if let Some(item_pos) = item_pos {
				ui.set_item_allow_overlap();
				ui.set_cursor_pos(item_pos);
			}
			self.desc.draw(ui, settings, texture);
		}

		if let Some(notifications) = self.notifications {
			debug!("TODO: draw notifications");
		}

		self.state = action;

		self.draw_popup(ui, settings);
	}

	pub fn draw_popup(&mut self, ui: &Ui, settings: &QuickAccessSettings) {
		match self.state {
			Some(QuickAccessItemAction::Hovered) => {
				self.draw_tooltip(ui, settings);
			},
			Some(QuickAccessItemAction::ContextMenu { .. }) => {
				self.draw_context(ui, settings);
			},
			_ => (),
		}
	}

	pub fn draw_tooltip(&mut self, ui: &Ui, settings: &QuickAccessSettings) -> bool {
		if self.desc.keybind_id.is_none() && self.desc.tooltip.is_none() {
			return false
		}

		ui.tooltip(|| {
			let tooltip_drawn = match self.tooltip() {
				Some(tooltip) => {
					ui.text(tooltip);
					true
				},
				_ => false,
			};

			if self.desc.keybind_id.is_some() {
				if tooltip_drawn {
					ui.separator()
				}

				ui.text("Hotkey: ");
				ui.same_line();
				match self.keybind() {
					Some(keybind) =>
						ui.text(keybind.to_string()),
					None =>
						ui.text_disabled("unassigned"),
				}
			}
		});

		true
	}

	pub fn draw_context(&mut self, ui: &Ui, settings: &QuickAccessSettings) {
		if self.context.is_empty() {
			return
		}
		let mut drawn = false;
		ui.popup("quick_access_context", || {
			drawn = true;
			if !ui.is_window_hovered() && !ui.is_any_item_hovered() { drawn = false; }
			// if !!ui.is_any_item_hovered()
			let mut is_first = true;
			for citem in &self.context {
				let _id = ui.push_id(imgui_id_cstr(&citem.id));
				if let Some(render) = citem.render {
					if !is_first {
						ui.separator()
					}
					render();
					is_first = false;
				}
			}
			if !drawn {
				ui.close_current_popup();
			}
		});

		match self.state {
			Some(QuickAccessItemAction::ContextMenu { pressed: None }) if !drawn => {
				self.state = None;
			},
			_ => (),
		}
	}

	pub fn tooltip(&mut self) -> Option<&str> {
		let tooltip = self.desc.tooltip.as_ref()?;
		let tooltip_valid = self.tooltip
			.get_or_insert_with(|| match tooltip.to_string_lossy() {
				Cow::Owned(s) => Err(s),
				Cow::Borrowed(..) => Ok(()),
			});
		Some(match tooltip_valid {
			Ok(()) => unsafe {
				str::from_utf8_unchecked(tooltip.as_bytes())
			},
			Err(s) => s,
		})
	}

	pub fn hover_time_since(&mut self, ui_time: f64) -> Option<Duration> {
		let start = *match self.state {
			Some(QuickAccessItemAction::Hovered) => self.hover_start.get_or_insert(ui_time),
			_ => {
				self.hover_start = None;
				return None
			},
		};
		Duration::try_from_secs_f64(ui_time - start).ok()
	}

	pub fn keybind(&mut self) -> Option<Keybind> {
		let keybind = self.desc.keybind_id.as_ref()?;

		let bind = InputBinds::lock_read().find_id(keybind)?.bind;

		Some(bind)
	}
}

impl QuickAccessItem {
	pub fn predraw(&self, ui: &Ui, settings: &QuickAccessSettings) -> Option<QuickAccessItemAction> {
		let mut action = None::<QuickAccessItemAction>;

		let clicked = ui.invisible_button("", settings.ui_size(ui));
		let clicked_context = ui.is_item_clicked_with_button(MouseButton::Right);
		let hovered = ui.is_item_hovered();

		if clicked_context {
			action.get_or_insert(QuickAccessItemAction::ContextMenu { pressed: Some(true) });
		}

		if clicked {
			action.get_or_insert(QuickAccessItemAction::Selected { pressed: false });
		} else if ui.is_item_clicked() {
			action.get_or_insert(QuickAccessItemAction::Selected { pressed: true });
		}

		if hovered {
			action.get_or_insert(QuickAccessItemAction::Hovered);
		}

		action
	}

	pub fn draw(&self, ui: &Ui, settings: &QuickAccessSettings, texture_id: imgui::TextureId) {
		imgui::Image::new(texture_id, settings.ui_size(ui))
			.build(ui);
	}
}

// TODO: proper state transitions instead of haphazard garbage! also one per item may make little sense if you can't interact with more than one at once?
impl QuickAccessItemAction {
	pub fn is_visual_hover(self) -> bool {
		match self {
			Self::Selected { .. } | Self::Hovered | Self::ContextMenu { pressed: Some(false) | None } => true,
			// have some visual feedback while waiting for the menu
			Self::ContextMenu { pressed: Some(true) } => false,
		}
	}
}

impl QuickAccessSettings {
	pub fn ui_size(&self, _ui: &Ui) -> [f32; 2] {
		//let [w, h] = ui.io().display_framebuffer_scale;
		let [w, h] = [1.0f32, 1.0f32];
		[w * 31.5, h * 31.5]
	}

	pub fn ui_spacing(&self) -> [f32; 2] {
		[0.5, 0.0]
	}

	pub fn ui_padding(&self) -> [f32; 2] {
		[0.5, 0.0]
	}

	pub fn ui_padding_hover(&self) -> [f32; 2] {
		[-4.0, -4.0]
	}

	pub fn ui_alpha(&self, hover: bool) -> f32 {
		match hover {
			true => 0.85,
			_ => 0.575,
		}
	}

	pub fn ui_start_pos(&self, ui: &Ui) -> [f32; 2] {
		let [w, _] = self.ui_size(ui);
		let [spacex, _] = self.ui_spacing();
		let x = (w + spacex) * 18.0;
		[x, 0.0]
	}
}

impl QuickAccessMenuUi {
	pub fn mark_dirty() {
		QA_UI_DIRTY.store(true, Ordering::Relaxed);
	}

	pub fn clear_dirty() {
		QA_UI_DIRTY.store(false, Ordering::Relaxed);
	}

	pub fn is_probably_dirty() -> bool {
		QA_UI_DIRTY.load(Ordering::Relaxed)
	}

	pub fn render() {
		thread_local! {
			static QA_UI: RefCell<QuickAccessMenuUi> = RefCell::default();
		}

		QA_UI.with_borrow_mut(|qa| {
			if Self::is_probably_dirty() {
				if let Ok(menu) = QUICK_ACCESS.try_read() {
					qa.rebuild(&menu);
					Self::clear_dirty();
				}
			}

			let settings = QuickAccessMenu::settings();
			RenderThread::with_ui(|ui| {
				qa.draw(ui, &settings);
			});
		});
	}

	pub fn is_empty(&self) -> bool {
		self.draw_items.is_empty()
	}

	pub fn draw(&mut self, ui: &Ui, settings: &QuickAccessSettings) {
		if self.is_empty() {
			return
		}

		let res = imgui::Window::new("arcloader_quickaccess")
			.flags(WindowFlags::NO_MOVE | WindowFlags::NO_NAV_INPUTS | WindowFlags::NO_NAV | WindowFlags::NO_DECORATION | WindowFlags::NO_FOCUS_ON_APPEARING | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | WindowFlags::NO_SCROLL_WITH_MOUSE | WindowFlags::ALWAYS_AUTO_RESIZE | WindowFlags::NO_BACKGROUND)
			.position_pivot([0.0, 0.0])
			.position(settings.ui_start_pos(ui), imgui::Condition::Always/*FirstUseEver*/)
			.build(ui, || {
				ui.set_cursor_pos([0.0, 0.0]);
				self.draw_items(ui, settings);
			});

		match res {
			None => {
				warn!("QA not drawn?");
			},
			Some(()) => (),
		}
	}

	pub fn draw_items(&mut self, ui: &Ui, settings: &QuickAccessSettings) {
		//let prev_screen_pos = ui.cursor_screen_pos();

		let mut context_pos = None;
		let mut hover_pos = None;
		let [bounds_x, _] = ui.window_size();

		for (i, item) in self.draw_items.iter_mut().enumerate() {
			let _id = ui.push_id(imgui_id_cstr(&item.desc.id));
			{
				let hover = item.state.map(|s| s.is_visual_hover()).unwrap_or(false);
				let _alpha = match item.notifications {
					None | Some(0) => Some(ui.push_style_var(StyleVar::Alpha(settings.ui_alpha(hover)))),
					Some(..) => None,
				};
				let padding = match hover {
					true => settings.ui_padding_hover(),
					false => settings.ui_padding(),
				};
				let _padding = ui.push_style_var(StyleVar::FramePadding(padding));
				let _spacing = ui.push_style_var(StyleVar::ItemSpacing(settings.ui_spacing()));
				#[cfg(todo)]
				if ui.cursor_screen_pos()[0] + settings.ui_size(ui)[0] + settings.ui_spacing()[0] >= bounds_x {
					ui.new_line();
				}
				item.draw(ui, settings);
				ui.same_line();
			}

			if let Some(QuickAccessItemAction::Selected { .. } | QuickAccessItemAction::ContextMenu { .. }) = item.state {
				item.clear_notifications();
			}

			match item.state {
				Some(QuickAccessItemAction::Hovered) => {
					let hover_time_since = item.hover_time_since(ui.time());
					if hover_time_since.map(|d| d >= settings.tooltip_delay).unwrap_or(false) {
						hover_pos.get_or_insert((i, ui.cursor_screen_pos()));
					}
				},
				Some(QuickAccessItemAction::Selected { pressed }) if !pressed || ui.is_mouse_clicked(MouseButton::Left) => {
					item.select();
				},
				Some(QuickAccessItemAction::ContextMenu { pressed: Some(false) | None }) => {
					context_pos.get_or_insert((i, ui.cursor_screen_pos()));
				},
				_ => (),
			}
		}

		let popup = context_pos.or(hover_pos);
		if let Some((i, pos)) = popup {
			let item = &mut self.draw_items[i];
			#[cfg(todo)] {
				ui.set_cursor_screen_pos(pos);
				item.draw_popup(ui, settings);
			}
			#[cfg(deleteme)]
			if context_pos.is_some() {
				debug!("TODO: context menu garbage");
				item.draw_context(ui, settings);
			} else {
				item.draw_tooltip(ui, settings);
			}

			match item.state {
				Some(QuickAccessItemAction::ContextMenu { pressed: Some(false) }) => {
					item.state = Some(QuickAccessItemAction::ContextMenu { pressed: None });
				},
				#[cfg(deleteme)]
				Some(QuickAccessItemAction::ContextMenu { pressed: None }) if !ui.is_any_item_focused() =>
					item.state = None,
				_ => (),
			}
		}

		//ui.set_cursor_screen_pos(prev_screen_pos);
	}

	pub fn rebuild(&mut self, menu: &QuickAccessMenu) {
		let mut menu_items: BTreeSet<*const QuickAccessItem> = menu.items.values()
			.map(|item| Arc::as_ptr(item))
			.collect();

		self.draw_items.retain(|uitem| {
			let key = Arc::as_ptr(&uitem.desc);
			menu_items.remove(&key)
		});

		for uitem in &mut self.draw_items {
			uitem.update_notifications(menu);
			if uitem.is_context_dirty(menu) {
				uitem.rebuild_context(menu);
			}
		}

		let new_items = menu.items.values()
			.filter(|item| menu_items.contains(&Arc::as_ptr(item)));

		for item in new_items {
			let idx = self.draw_items.partition_point(|uitem| uitem.desc.id <= item.id);
			let mut uitem = QuickAccessItemUi::with_item(item.clone());
			uitem.rebuild_context(menu);
			self.draw_items.insert(idx, uitem);
		}
	}

	fn hash_context_items<'a, I>(items: I) -> u64 where
		I: IntoIterator<Item = &'a Arc<QuickAccessContextItem>>,
	{
		let mut signature = 0usize;
		let mut len = 0usize;
		for i in items {
			let value = Arc::as_ptr(i) as usize;
			signature = signature.wrapping_add(value);
			len = len.wrapping_add(1);
		}
		let mut hasher = DefaultHasher::new();
		len.hash(&mut hasher);
		signature.hash(&mut hasher);
		hasher.finish()
	}
}

impl QuickAccessItemUi {
	pub fn is_context_dirty(&self, menu: &QuickAccessMenu) -> bool {
		let menu_items = menu.context_items_for_target(Arc::as_ptr(&self.desc));
		let hash_self = QuickAccessMenuUi::hash_context_items(self.context.iter());
		let hash_menu = QuickAccessMenuUi::hash_context_items(menu_items);
		hash_self != hash_menu
	}

	pub fn rebuild_context(&mut self, menu: &QuickAccessMenu) {
		self.context.clear();

		let menu_items = menu.context_items_for_target(Arc::as_ptr(&self.desc));
		for citem in menu_items {
			let idx = self.context.partition_point(|uicitem| uicitem.id <= citem.id);
			let uicitem = citem.clone();
			self.context.insert(idx, uicitem);
		}
	}

	pub fn update_notifications(&mut self, menu: &QuickAccessMenu) {
		self.notifications = menu.notifications.get(&self.desc.id).copied();
	}

	pub fn clear_notifications(&mut self) {
		if self.notifications.is_none() {
			return
		}

		if self.desc.try_clear_notifications().is_err() {
			return
		}
		self.notifications = None;
	}

	pub fn select(&mut self) {
		let pressed = match self.state {
			Some(QuickAccessItemAction::Selected { pressed }) => {
				if !pressed {
					self.state = None;
				}
				pressed
			},
			_ => {
				debug!("that's weird... {self:?}");
				true
			},
		};

		self.desc.select(pressed);
	}
}
