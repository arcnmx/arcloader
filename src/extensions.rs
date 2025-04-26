use std::{cell::Cell, collections::{BTreeMap, BTreeSet}, ffi::{c_char, CStr, OsString}, mem::transmute, os::windows::ffi::OsStringExt, path::{Path, PathBuf}, ptr::NonNull, rc::Rc};
use arcdps::{
	evtc::{Agent, Event}, exports::{self, AddExtensionResult}, imgui::{TableColumnSetup, Ui}
};
#[cfg(feature = "extras")]
use arcdps::extras::{ExtrasAddonInfo, UserInfoIter};
#[cfg(feature = "log")]
use log::{debug, info, warn, error};
use windows::{core::{Error as WinError, PCSTR}, Win32::{Foundation::{FreeLibrary, GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, HMODULE, MAX_PATH, WIN32_ERROR}, System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleExA, GetProcAddress, LoadLibraryW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT}}};
use windows_strings::HSTRING;

#[derive(Clone, Debug)]
pub struct ExtCache {
	pub sig: u32,
	pub name: String,
	pub file_name: OsString,
}

#[derive(Clone, Debug)]
pub struct ExtSettings {
	pub blacklist: BTreeSet<u32>,
	pub cache: BTreeSet<ExtCache>,
}

#[derive(Clone, Debug)]
pub enum ExtState {
	Loaded {
		build: Rc<str>,
		can_release: bool,
	},
	Unloaded {
		path: Rc<Path>,
	},
	Removed {
		handle: HMODULE,
	},
}

#[derive(Clone, Debug)]
pub struct Ext {
	sig: u32,
	name: Rc<str>,
	state: ExtState,
}

pub type WinResult<T> = Result<T, WinError>;

pub fn get_module_path(handle: Option<HMODULE>) -> WinResult<OsString> {
	let mut file_name_buf = [0u16; 128];
	let res = unsafe {
		let sz = GetModuleFileNameW(handle, &mut file_name_buf);
		GetLastError()
			.ok().map(|()| sz as usize)
	};
	match res {
		Err(e) if e.code().0 == ERROR_INSUFFICIENT_BUFFER.0 as i32 => (),
		Err(e) => return Err(e),
		Ok(len @ 0..=128) => return Ok(OsString::from_wide(&file_name_buf[..len])),
		Ok(_) => {
			#[cfg(feature = "log")] {
				debug!("weird, I didn't ask for that");
			}
		},
	}
	
	let mut buf = vec![0u16; MAX_PATH as usize];
	let res = unsafe {
		let sz = GetModuleFileNameW(handle, &mut buf[..]);
		GetLastError()
			.ok().map(|()| sz as usize)
	};
	res.map(move |len| {
		buf.truncate(len);
		OsString::from_wide(&buf)
	})
}

impl Ext {
	pub fn loaded(export: &arcdps_exports) -> Self {
		Self {
			sig: export.sig,
			name: Rc::from(export.name().to_string_lossy().into_owned().into_boxed_str()),
			state: ExtState::Loaded {
				build: Rc::from(export.build().to_string_lossy().into_owned().into_boxed_str()),
				// TODO:
				can_release: Self::export_release_fn(export).is_some(),
			},
		}
	}

	pub fn get_module_name(handle: HMODULE) -> Rc<str> {
		get_module_path(Some(handle)).ok()
			.and_then(|path| Path::new(&path).file_name().map(|s| s.to_os_string()))
			.map(|s| s.to_string_lossy().into_owned())
			.unwrap_or_else(|| format!("{:?}", handle))
			.into()
	}

	pub fn export_release_fn(export: &arcdps_exports) -> Option<unsafe extern "system" fn()> {
		let handle = Self::handle_by_export(export)?;
		unsafe {
			GetProcAddress(handle, windows_strings::s!("get_release_addr"))
				.map(|f| transmute(f))
		}
	}

	pub fn handle_by_export(export: &arcdps_exports) -> Option<HMODULE> {
		let mut any_fn_ptr = [
			export.wnd_filter,
			export.combat,
			export.imgui,
			export.options_tab,
			export.combat_local,
			export.wnd_filter,
			export.options_windows,
			//export.out_name,
		].into_iter().filter_map(|p| NonNull::new(p as *mut ()));

		let handle = any_fn_ptr.next()
			.and_then(|p| Self::handle_by_ptr(p.as_ptr()).transpose())
			.transpose();
		match handle {
			#[cfg(todo)]
			Ok(None) => match Self::handle_by_ptr(export.out_name) {
				Ok(Some(h)) if !Self::handle_is_arcdps(h) => Some(h),
				_ => None,
			},
			Ok(h) => h,
			Err(e) => {
				#[cfg(feature = "log")] {
					warn!("failed to determine handle for {:?}", export.name());
				}
				None
			},
		}
	}

	pub fn handle_by_ptr(p: *const ()) -> WinResult<Option<HMODULE>> {
		// TODO: owned module???
		let mut handle = HMODULE::default();
		let res = unsafe {
			let flags = GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS;
			GetModuleHandleExA(flags, PCSTR(p as *const _), &mut handle)
		};
		res.map(|()| if handle.is_invalid() && handle != HMODULE::default() { None } else { Some(handle) })
	}

	pub fn handle_by_path(path: &Path) -> Option<HMODULE> {
		// TODO: GetModuleHandleExW()
		None
	}

	#[cfg(todo)]
	pub fn unloaded(sig: u32, name: String, handle: HMODULE) -> WinResult<Self> {
		let path = get_module_path(Some(handle))?;
		Ok(Self {
			sig,
			name: Rc::new(name),
			state: ExtState::Unloaded {
				path: Rc::new(path.into()),
			},
		})
	}

	#[cfg(todo)]
	pub fn with_handle(sig: u32, name: String, handle: HMODULE) -> WinResult<Self> {
		let path = get_module_file_name(Some(handle))?;
		Ok(Self {
			sig,
			name: Rc::new(name),
			state: ExtState::Unloaded {
				path: Rc::new(path.into()),
			},
		})
	}
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct arcdps_exports {
	size: u64,
	sig: u32,
	imguivers: u32,
	out_name: *const c_char,
	out_build: *const c_char,
	wnd_nofilter: *const (),
	combat: *const (),
	imgui: *const (),
	options_tab: *const (),
	combat_local: *const (),
	wnd_filter: *const (),
	options_windows: *const (),
}

impl arcdps_exports {
	pub fn name(&self) -> &CStr {
		unsafe {
			CStr::from_ptr(self.out_name)
		}
	}

	pub fn build(&self) -> &CStr {
		unsafe {
			CStr::from_ptr(self.out_build)
		}
	}
}

#[derive(Debug)]
enum ExtCommand {
	Remove {
		sig: u32,
	},
	Unload {
		sig: u32,
		handle: HMODULE,
	},
	Reload {
		handle: HMODULE,
	},
}

pub struct Exts {
	pub extensions: BTreeMap<u32, Ext>,
}

impl Exts {
	pub fn new() -> Self {
		Self {
			extensions: Default::default()
		}
	}

	pub fn imgui_options_table(&mut self, ui: &Ui) -> Result<(), &'static str> {
		let exts = ui.begin_table_header("exts", [
			TableColumnSetup::new("extensions"),
			TableColumnSetup::default(),
			TableColumnSetup::new("arcload"),
			TableColumnSetup::default(),
		]).unwrap();
		ui.table_next_column();

		if !exports::has_list_extension() {
			return Err("unsupported");
		}

		thread_local! {
			static EXT_REF: Cell<Option<*mut dyn FnMut(&arcdps_exports)>> = Cell::new(None);
		}

		fn ext_callback(ext: &arcdps_exports) {
			match EXT_REF.get() {
				Some(cb) => unsafe { (*cb)(ext) },
				None => {
					let _ = exports::log_to_window("list_extension failure");
				},
			}
		}

		let mut extensions_unloaded: BTreeSet<u32> = self.extensions.values()
			.filter_map(|ext| match ext.state {
				ExtState::Loaded { .. } => Some(ext.sig),
				ExtState::Removed { .. } | ExtState::Unloaded { .. } => None,
			}).collect();

		{
			let extensions = &mut self.extensions;
			let mut ext = |export: &arcdps_exports| {
				extensions_unloaded.remove(&export.sig);
				let ext = extensions.entry(export.sig).or_insert_with(|| Ext::loaded(export));
				if let ExtState::Unloaded { .. } | ExtState::Removed { .. } = ext.state {
					ext.name = Rc::from(export.name().to_string_lossy().into_owned().into_boxed_str());
					ext.state = ExtState::Loaded {
						build: Rc::from(export.build().to_string_lossy().into_owned().into_boxed_str()),
						can_release: Ext::export_release_fn(export).is_some(),
					};
				}
			};


			EXT_REF.set(Some(unsafe { transmute(&mut ext as &mut dyn FnMut(&arcdps_exports)) }));
			unsafe {
				arcdps::exports::raw::list_extension(ext_callback as *const _);
			}
			EXT_REF.set(None);
		}

		for sig in extensions_unloaded {
			if let Some(_ext) = self.extensions.remove(&sig) {
				#[cfg(feature = "log")] {
					info!("lost track of extension: {}", _ext.name);
				}
			} else {
				#[cfg(feature = "log")] {
					warn!("what happened to extension {:08x}?", sig);
				}
			}
		}

		let mut ext_cmd = None;
		for ext in self.extensions.values() {
			ui.label_text(&ext.name[..], format!("{:08x}", ext.sig));
			ui.table_next_column();

			match ext.state {
				ExtState::Loaded { ref build, .. } => ui.text_disabled(build),
				ExtState::Removed { handle } if exports::has_add_extension() => if ui.button("reload") {
					#[cfg(feature = "log")] {
						info!("reloading {}", ext.name);
					}
					ext_cmd = Some(ExtCommand::Reload { handle });
				},
				ExtState::Unloaded { ref path } if exports::has_add_extension() => {
					if ui.button("load") {
						#[cfg(feature = "log")] {
							info!("reloading {}", ext.name);
						}
						let res = unsafe { LoadLibraryW(&HSTRING::from(path.as_os_str())) };
						match res {
							Ok(handle) => ext_cmd = Some(ExtCommand::Reload { handle }),
							Err(_e) => {
								#[cfg(feature = "log")] {
									error!("failed to load extension {} from {:?}: {:?}", ext.name, path, _e);
								}
							}
						}
					}
					ui.text_disabled(path.to_string_lossy());
				},
				_ => ui.text_disabled("unavailable"),
			}
			ui.table_next_column();

			let is_self = ext.sig == crate::exports::SIG;
			let remove = match ext.state {
				_ if !exports::has_free_extension() => false,
				ExtState::Loaded { can_release: true, .. } if !is_self => ui.button("remove"),
				_ => {
					ui.text_disabled("remove");
					false
				},
			};
			if remove {
				//exports::log_to_window(format!("unloading {:?}", ext.name()));
				#[cfg(feature = "log")] {
					info!("removing {}...", ext.name);
				}
				ext_cmd = Some(ExtCommand::Remove { sig: ext.sig });
			}
			ui.table_next_column();
			if !is_self && exports::has_free_extension() {
				let unload = if let ExtState::Removed { handle } = ext.state {
					ui.button("unload").then_some(handle)
				} else {
					ui.text_disabled("unload");
					None
				};
				if let Some(handle) = unload {
					#[cfg(feature = "log")] {
						info!("unloading {}...", ext.name);
					}
					ext_cmd = Some(ExtCommand::Unload { sig: ext.sig, handle });
				}
			}
			ui.table_next_column();
		}

		#[cfg(feature = "log")]
		if let Some(cmd) = &ext_cmd {
			debug!("arcloader command: {:?}...", cmd);
		}

		match ext_cmd {
			Some(ExtCommand::Remove { sig }) => {
				match exports::free_extension(sig) {
					None => {
						#[cfg(feature = "log")] {
							info!("could not remove something that doesn't exist?");
						}
					},
					Some(handle) => {
						let ext = self.extensions.entry(sig)
							.or_insert_with(|| Ext {
								sig,
								name: Ext::get_module_name(handle),
								state: ExtState::Removed { handle },
							});
						#[cfg(feature = "log")] {
							info!("extension removed: {}", ext.name);
						}
						ext.state = ExtState::Removed { handle };
					},
				}
			},
			Some(ExtCommand::Unload { sig, handle }) => {
				let path = get_module_path(Some(handle));
				let unloaded = unsafe {
					FreeLibrary(handle)
				};
				match unloaded {
					Err(e) => {
						#[cfg(feature = "log")] {
							error!("failed to unload {:08x}: {}", sig, e);
						}
					},
					Ok(()) => {
						#[cfg(feature = "log")] {
							info!("removed!");
						}
						match path {
							Ok(path) => if let Some(ext) = self.extensions.get_mut(&sig) {
								ext.state = ExtState::Unloaded {
									path: Rc::from(PathBuf::from(path)),
								};
							},
							Err(_e) => {
								#[cfg(feature = "log")] {
									warn!("no path known for {:08x}: {}", sig, _e);
								}
								self.extensions.remove(&sig);
							},
						}
					},
				}
			},
			Some(ExtCommand::Reload { handle }) => {
				match exports::add_extension(handle) {
					AddExtensionResult::Ok => {
						#[cfg(feature = "log")] {
							info!("loaded!");
						}
					},
					_res => {
						#[cfg(feature = "log")] {
							error!("extension failed to load: {:?}", _res);
						}
					},
				}
			},
			None => (),
		}

		exts.end();

		Ok(())
	}
}
