use crate::{extensions::{Loader, LoaderCommand}, supervisor::{Supervisor, SupervisorCommand, SUPERVISOR}, util::{arc::game_dir, win::get_module_from_name}};
use std::{cell::RefCell, collections::{BTreeSet, HashSet}, ffi::OsString, num::NonZeroU32, path::{Path, MAIN_SEPARATOR_STR}, sync::Arc};
use arcdps::{
	 exports::{self, CoreColor}, imgui::{Id, StyleColor, TableColumnSetup, TableFlags, Ui}
};
#[cfg(feature = "extras")]
use arcdps::extras::{ExtrasAddonInfo, UserInfoIter};
use windows::core::Owned;
use windows_strings::HSTRING;

use crate::util::win::get_module_path;

thread_local! {
	static OPTIONS: RefCell<Option<Options>> = RefCell::new(None);
}

#[derive(Clone, Debug)]
struct ExtCache {
	pub sig: Option<NonZeroU32>,
	pub name: String,
	pub file_name: OsString,
}

#[derive(Clone, Debug)]
struct ExtSettings {
	pub blacklist: BTreeSet<u32>,
	pub cache: BTreeSet<ExtCache>,
}

#[derive(Debug)]
enum UserCommand {
	Loader(LoaderCommand),
	Supervisor(SupervisorCommand),
}

pub struct Options {
}

impl Options {
	pub fn new() -> Self {
		Self {
		}
	}

	pub fn init() {
	}

	pub fn unload() {
	}

	pub fn imgui_options_end(ui: &Ui) {
		OPTIONS.with_borrow_mut(|opts| match opts {
			Some(opts) => opts.imgui_render(ui),
			None => ui.text_disabled("unsupported"),
		})
	}

	pub fn imgui_present(_ui: &Ui) {
		let needs_init = OPTIONS.with_borrow(|opt| opt.is_none());
		if needs_init {
			OPTIONS.set(Some(Self::new()));
		}
	}

	pub fn imgui_render(&mut self, ui: &Ui) {
		ui.spacing();

		if ui.button("Refresh") {
			let _ = Supervisor::send_command(SupervisorCommand::RefreshArcdps);
			let _ = Supervisor::send_command(SupervisorCommand::RefreshExternal);
		}

		self.imgui_options_table(ui)
	}

	pub fn imgui_options_table(&mut self, ui: &Ui) {
		if !exports::has_list_extension() {
			ui.text_disabled("unsupported");
			return
		}

		let mut seen = HashSet::new();

		let ext_cmd = self.imgui_extensions_table(ui, &mut seen);

		#[cfg(feature = "log")]
		if let Some(cmd) = &ext_cmd {
			debug!("arcloader command: {cmd:?}...");
		}

		#[cfg(feature = "nexus-host")] {
			self.extensions_options_nexus(ui);
		}

		if let Some(cmd) = ext_cmd {
			// TODO: this may be async, need a way to get result back later!
			let res = Loader::send_command(cmd);

			if let Err(e) = res {
				#[cfg(feature = "log")] {
					error!("loader failed: {e}");
				}
			}
		}
	}

	fn colours() -> Option<exports::Colors> {
		exports::has_e5_colors().then(|| exports::colors())
	}

	pub fn imgui_extensions_table(&mut self, ui: &Ui, seen: &mut HashSet<OsString>) -> Option<LoaderCommand> {
		let mut cmd = None;

		let sv = match SUPERVISOR.try_read() {
			Ok(sv) => sv,
			Err(_e) => {
				// TODO: fall back to cached state or placeholder idk
				ui.text_disabled("refreshing...");
				return None
			},
		};

		let exts = ui.begin_table_header_with_flags("exts", [
			TableColumnSetup::new("extensions"),
			TableColumnSetup::default(),
			TableColumnSetup::default(),
			TableColumnSetup::default(),
		], TableFlags::ROW_BG | TableFlags::BORDERS_H | TableFlags::NO_SAVED_SETTINGS).unwrap();
		ui.table_next_column();

		if let Some(c) = self.extensions_table_arcdps(ui, &sv, seen) {
			cmd = Some(c)
		}

		#[cfg(feature = "nexus-host")]
		if let Some(()) = self.extensions_table_nexus(ui, &sv, seen) {
		}

		if let Some(c) = self.extensions_table_external(ui, &sv, seen) {
			cmd = Some(c)
		}

		exts.end();

		ui.separator();

		cmd
	}

	pub fn extensions_table_arcdps(&mut self, ui: &Ui, sv: &Supervisor, seen: &mut HashSet<OsString>) -> Option<LoaderCommand> {
		let mut cmd = None;

		let colours = Self::colours();
		let button_width = ui.current_font().fallback_advance_x * 10.0;

		for ext in sv.arcdps.values() {
			let ext_token = ui.push_id(Id::Int(ext.sig.get() as i32));

			if let Some(path) = &ext.path {
				if let Some(fname) = path.file_name() {
					seen.insert(fname.to_owned());
				}
				seen.insert(path.as_os_str().to_owned());
			}

			let is_self = ext.sig == crate::export::arcdps::SIG;
			if !is_self && exports::has_remove_extension() {
				let width = ui.current_column_width()
					.max(button_width);
				let remove = ui.button_with_size("unload", [width, 0.0]);
				if remove {
					#[cfg(feature = "log")] {
						info!("removing {}...", ext.name);
					}
					cmd = Some(LoaderCommand::Unload { sig: ext.sig });
				}
			}
			ui.table_next_column();

			ui.text(&ext.name);
			ui.same_line();
			ui.text_disabled(format!("{:08x}", ext.sig));
			let colour = colours.as_ref()
				.and_then(|c| c.core(CoreColor::LightTeal))
				.unwrap_or(ui.style_color(StyleColor::NavHighlight));
			let c = ui.push_style_color(StyleColor::Text, colour);
			ui.text_wrapped(&ext.build);
			c.end();

			ui.same_line();
			ui.text(" (arcdps)");

			let fname = ext.path.as_ref()
				.and_then(|p| p.file_name())
				.and_then(|p| p.to_str());
			if let Some(fname) = fname {
				let colour = colours.as_ref()
					.and_then(|c| c.core(CoreColor::LightGrey))
					.unwrap_or(ui.style_color(StyleColor::Separator));
				let c = ui.push_style_color(StyleColor::Text, colour);
				ui.text(fname);
				c.end();
			}
			ui.table_next_column();

			ext_token.end();
		}

		cmd
	}

	pub fn extensions_table_external(&mut self, ui: &Ui, sv: &Supervisor, seen: &mut HashSet<OsString>) -> Option<LoaderCommand> {
		let mut cmd = None;

		let colours = Self::colours();
		let button_width = ui.current_font().fallback_advance_x * 10.0;

		for ext in &sv.external {
			if seen.contains(ext.path.as_os_str()) {
				continue
			}
			if let Some(fname) = ext.path.file_name() {
				if seen.contains(fname) {
					continue
				}
			}

			let ext_token = ui.push_id(Id::Ptr(Arc::as_ptr(ext) as *const _));
			let width = ui.current_column_width()
					.max(button_width);
			if !exports::has_add_extension() {
				ui.text_disabled("unavailable");
			} else if ui.button_with_size("load", [width, 0.0]) {
				cmd = Some(LoaderCommand::LoadPath { path: ext.path.as_path().into() });
			}
			if ui.button_with_size("free", [width, 0.0]) {
				// TODO: move this into a command!
				let handle = get_module_from_name(&HSTRING::from(ext.path.as_os_str()));
				let handle = match handle {
					Ok(None) => match ext.path.file_name() {
						Some(name) => get_module_from_name(&HSTRING::from(name)),
						None => Ok(None),
					},
					res => res,
				};
				match handle {
					Ok(Some(module)) => unsafe {
						#[cfg(feature = "log")] {
							info!("freeing {module:?} forcibly");
						}

						let module = Owned::new(module);
						drop(module)
					},
					Err(_e) => {
						#[cfg(feature = "log")] {
							warn!("{} not found: {_e}", ext.path.display());
						}
					},
					Ok(None) => {
						#[cfg(feature = "log")] {
							info!("{} not found", ext.path.display());
						}
					},
				}
			}
			ui.table_next_column();

			let fname = ext.path.file_name().and_then(|f| f.to_str());
			let name = match () {
				_ => fname,
			};
			let sig = None::<NonZeroU32>;

			if let Some(name) = name {
				ui.text(name);
			} else {
				ui.text_disabled("external");
			}
			if let Some(sig) = sig {
				ui.same_line();
				ui.text_disabled(format!("{:08x}", sig));
			}
			let (path, path_is_file) = match (ext.path.parent(), fname) {
				(Some(parent), fname@Some(..)) if name == fname =>
					(parent, false),
				_ => (ext.path.as_path(), true),
			};
			let display_path = match game_dir().and_then(|game_root| path.strip_prefix(game_root).ok()) {
				Some(p) if p.as_os_str().is_empty() => None,
				Some(p) => Some(p),
				None => Some(path),
			};
			if let Some(path) = display_path {
				let colour = colours.as_ref()
					.and_then(|c| c.core(CoreColor::LightGrey))
					.unwrap_or(ui.style_color(StyleColor::Separator));
				let c = ui.push_style_color(StyleColor::Text, colour);
				if path.is_relative() {
					let suffix = match path_is_file {
						true => "",
						false => MAIN_SEPARATOR_STR,
					};
					ui.text_wrapped(format!("{}{}", path.display(), suffix));
				} else {
					ui.text_wrapped(path.to_string_lossy());
				}
				c.end();
			}
			ui.table_next_column();

			ext_token.end();
		}

		cmd
	}

	#[cfg(feature = "nexus-host")]
	pub fn extensions_table_nexus(&mut self, ui: &Ui, _sv: &Supervisor, seen: &mut HashSet<OsString>) -> Option<()> {
		use nexus::AddonFlags;
		use crate::extensions::nexus::{NexusHost, NEXUS_HOST};

		let colours = Self::colours();
		let button_width = ui.current_font().fallback_advance_x * 10.0;

		ui.separator();

		let host = match NEXUS_HOST.try_read() {
			Ok(h) => h,
			_ => return None,
		};

		for addon in host.addons.values() {
			let ext_token = ui.push_id(Id::Ptr(Arc::as_ptr(addon) as *const _));

			if let Ok(path) = get_module_path(Some(addon.module())) {
				if let Some(fname) = Path::new(&path).file_name() {
					seen.insert(fname.to_owned());
				}
				seen.insert(path);
			}

			let width = ui.current_column_width()
					.max(button_width);
			let unload = match addon.def().flags.contains(AddonFlags::DisableHotloading) && addon.is_loaded() {
				false => ui.button_with_size("unload", [width, 0.0]),
				true => {
					ui.text_disabled("unload");
					false
				},
			};
			if unload {
				#[cfg(feature = "log")] {
					info!("unloading {addon}...");
				}
				let sig = addon.def().signature;
				drop(host);
				let res = NexusHost::unload_addon(sig);
				return None
			}
			if !addon.is_loaded() {
				let load = ui.button_with_size("load", [width, 0.0]);
				if load {
					#[cfg(feature = "log")] {
						info!("loading {addon}...");
					}
					let sig = addon.def().signature;
					drop(host);
					let res = NexusHost::load_addon(sig);
					return None
				}
			}
			if ui.button("whee") {
				for (c, cb) in &NexusHost::lock_read().fallback_cache.write().unwrap().key_binds {
					cb(c.as_ptr(), false);
				}
			}
			ui.table_next_column();

			ui.text(addon.name().to_string_lossy());
			if let Some(author) = addon.author() {
				let c = match addon.is_raidcore() {
					true => colours.as_ref()
						.and_then(|c| c.core(CoreColor::LightRed)),
					false => None,
				};
				let c = c.map(|c| ui.push_style_color(StyleColor::Text, c));

				ui.same_line();
				ui.text(" by ");
				ui.same_line();
				ui.text(author.to_string_lossy());

				drop(c);
			}
			ui.same_line();
			ui.text_disabled(format!("{:08x}", addon.def().signature));
			let colour = colours.as_ref()
				.and_then(|c| c.core(CoreColor::LightTeal))
				.unwrap_or(ui.style_color(StyleColor::NavHighlight));
			let c = ui.push_style_color(StyleColor::Text, colour);
			ui.text(addon.version().to_string());
			c.end();

			ui.same_line();
			ui.text(format!(" ({})", addon.api));

			if let Some(desc) = addon.description() {
				ui.text_wrapped(desc.to_string_lossy());
			}
			/*
			let fname = addon.path.as_ref()
				.and_then(|p| p.file_name())
				.and_then(|p| p.to_str());
			if let Some(fname) = fname {
				let colour = colours.as_ref()
					.and_then(|c| c.core(CoreColor::LightGrey))
					.unwrap_or(ui.style_color(StyleColor::Separator));
				let c = ui.push_style_color(StyleColor::Text, colour);
				ui.text(fname);
				c.end();
			}*/
			ui.table_next_column();

			ext_token.end();
		}

		None
	}

	#[cfg(feature = "nexus-host")]
	pub fn extensions_options_nexus(&mut self, ui: &Ui) {
		use nexus::gui::RenderType;
		use crate::extensions::nexus::{NexusAddonCache, NEXUS_HOST};

		ui.separator();

		let host = match NEXUS_HOST.try_read() {
			Ok(h) => h,
			_ => return,
		};

		let header = ui.tab_bar("addon_options");
		for addon in host.addons.values() {
			let ext_token = ui.push_id(Id::Ptr(Arc::as_ptr(addon) as *const _));

			let cache = NexusAddonCache::lock_read(&addon.cache);
			let renderers = match cache.renderers.get(&RenderType::OptionsRender) {
				Some(render) if !render.is_empty() => render,
				_ => continue,
			};

			//or collapsing_header_with_close_button?
			let tab = match ui.tab_item(addon.name().to_string_lossy()) {
				Some(tab) => tab,
				None => continue,
			};

			for cb in renderers {
				cb();
			}

			// TODO: quick access interactions, show keybinds, and more?

			tab.end();

			ext_token.end();
		}
		drop(header);
	}
}
