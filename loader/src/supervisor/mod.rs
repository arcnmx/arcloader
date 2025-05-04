use crate::util::{arc::{config_dir, game_dir}, win::WinResult};
use std::{collections::{BTreeMap, BTreeSet}, ffi::OsStr, fs, io, iter, num::NonZeroU32, ops::Deref, os::windows::fs::FileTypeExt, path::{Path, PathBuf}, sync::{Arc, RwLock}};
use arcdps::exports;
#[cfg(feature = "arcdps-extras")]
use arcdps::extras::{ExtrasAddonInfo, UserInfoIter};
use windows::{core::Error as WinError, Win32::Foundation::{ERROR_INDEX_OUT_OF_BOUNDS, ERROR_INVALID_HANDLE, ERROR_THREAD_WAS_SUSPENDED, HMODULE}};

use crate::util::{arc::ArcDpsExtensionRef, win::get_module_path};

pub static SUPERVISOR: RwLock<Supervisor> = RwLock::new(Supervisor::empty());

#[derive(Clone, Debug)]
pub enum SupervisorCommand {
	RefreshArcdps,
	RefreshExternal,
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ExtDir {
	pub path: PathBuf,
}

impl ExtDir {
	pub const INFIX_ARCDPS: &'static str = "arcdps_";
	#[cfg(feature = "host-addonapi")]
	pub const INFIX_NEXUS: &'static str = "nexus_";
	pub const EXTENSION: &'static str = "dll";

	pub fn new<P: Into<PathBuf>>(path: P) -> Self {
		let path = path.into();

		Self {
			path,
		}
	}

	pub fn enumerate_extensions(&self) -> io::Result<Vec<ExtDisk>> {
		let mut files = Vec::new();

		for f in fs::read_dir(&self.path)? {
			let f = match f {
				Ok(f) if !f.file_type().map(|ft| ft.is_file() || ft.is_symlink_file()).unwrap_or(false) =>
					continue,
				Ok(f) => f,
				Err(_e) => {
					warn!("failed to enumerate {}: {}", self.path.display(), _e);
					continue
				},
			};
			let fname = f.file_name();

			if Path::new(&fname).extension() != Some(OsStr::new(Self::EXTENSION)) {
				continue
			}

			match f.file_name().as_os_str().to_str() {
				Some(s) if s.contains(Self::INFIX_ARCDPS) => (),
				#[cfg(feature = "host-addonapi")]
				Some(s) if s.contains(Self::INFIX_NEXUS) => (),
				_ => continue,
			}
			files.push(ExtDisk {
				path: self.path.join(fname)
			});
		}

		Ok(files)
	}
}

#[derive(Clone, Debug)]
pub struct ExtArc {
	pub desc: Arc<ExtArcDesc>,
	pub module: HMODULE,
}

impl ExtArc {
	pub fn with_extension(ext: ArcDpsExtensionRef) -> Self {
		Self {
			module: ext.module(),
			desc: Arc::new(ExtArcDesc::with_extension(ext)),
		}
	}
}

unsafe impl Send for ExtArc {}
unsafe impl Sync for ExtArc {}

impl Deref for ExtArc {
	type Target = ExtArcDesc;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.desc
	}
}

#[derive(Clone, Debug)]
pub struct ExtArcDesc {
	pub sig: NonZeroU32,
	pub name: String,
	pub build: String,
	pub path: Option<PathBuf>,
}

impl ExtArcDesc {
	pub fn with_extension(ext: ArcDpsExtensionRef) -> Self {
		Self {
			sig: ext.sig(),
			name: ext.name().to_string_lossy().into_owned(),
			build: ext.build().to_string_lossy().into_owned(),
			path: match get_module_path(Some(ext.module())) {
				Ok(p) => Some(PathBuf::from(p)),
				Err(_e) => {
					warn!("no path found for arcdps extension {}: {}", ext, _e);
					None
				},
			},
		}
	}
}

#[derive(Clone, Debug)]
pub struct ExtDisk {
	pub path: PathBuf,
}

#[derive(Debug)]
pub struct Supervisor {
	pub arcdps: BTreeMap<NonZeroU32, ExtArc>,
	pub external: Vec<Arc<ExtDisk>>,
	pub arcdps_dirty: bool,
}

/// TODO: this into settings!
pub fn external_dirs() -> BTreeSet<ExtDir> {
	let mut dirs = BTreeSet::new();

	if let Some(arcdps) = config_dir() {
		let arcloader_arc = arcdps.join("arcloader");

		let addons_dir = arcdps.parent();
		let arcloader_addons = addons_dir.map(|a| a.join("arcloader"));

		for arcloader in arcloader_addons.into_iter().chain(iter::once(arcloader_arc)) {
			if let Some(true) = arcloader.try_exists().ok() {
				dirs.insert(ExtDir::new(arcloader));
			}
		}

		dirs.insert(ExtDir::new(arcdps));
	}

	if let Some(gw2) = game_dir() {
		dirs.insert(ExtDir::new(gw2));
	}

	dirs
}

impl Supervisor {
	pub const fn empty() -> Self {
		Self {
			arcdps: BTreeMap::new(),
			external: Vec::new(),
			arcdps_dirty: true,
		}
	}

	pub fn init() {
		// TODO: try scheduling updates for later
		// sv.refresh_external();
	}

	pub fn imgui_present() {
		Self::dirty_update();
	}

	pub fn unload() {
		let mut sv = match SUPERVISOR.write() {
			Ok(s) => s,
			Err(e) => return,
		};

		sv.shutdown();
	}

	pub fn dirty_update() {
		let dirty = SUPERVISOR.try_read().ok()
			.map(|sv| sv.arcdps_dirty);
		let arcdps_dirty = match dirty {
			None | Some(false) => return,
			ad => ad,
		};
		let mut sv = match SUPERVISOR.try_write() {
			Ok(sv) => sv,
			_ => return,
		};
		if let Some(true) = arcdps_dirty {
			sv.refresh_arcdps();
		}
	}

	pub fn refresh_arcdps(&mut self) {
		if !exports::has_list_extension() {
			return
		}

		let extensions_unloaded = {
			let mut extensions_unloaded: BTreeSet<NonZeroU32> = self.arcdps.keys().copied().collect();

			let extensions = &mut self.arcdps;
			let mut ext = |export: &_| {
				let ext = unsafe { ArcDpsExtensionRef::with_exports(export) };
				if ext.size == 0 || ext.sig == 0 {
					// not yet loaded...
					return
				}
				extensions_unloaded.remove(&ext.sig());
				extensions.entry(ext.sig())
					.or_insert_with(|| ExtArc::with_extension(ext));
			};

			exports::list_extension(&mut ext);

			extensions_unloaded
		};

		for sig in extensions_unloaded {
			if let Some(_ext) = self.arcdps.remove(&sig) {
				info!("lost track of arcdps extension {}", _ext.desc.name);
			} else {
				warn!("what happened to arcdps extension {:08x}?", sig);
			}
		}

		self.arcdps_dirty = false;
	}

	pub fn refresh_external(&mut self) {
		let extensions = external_dirs().into_iter()
			.flat_map(|dir| match dir.enumerate_extensions() {
				Ok(dir) => Some(dir),
				Err(_e) => {
					error!("failed to enumerate {}: {_e}", dir.path.display());
					None
				},
			}).flatten();

		self.external.clear();
		self.external.extend(extensions.map(Arc::new));
	}

	pub fn shutdown(&mut self) {
		self.arcdps.clear();
		self.external.clear();
	}

	/// TODO: channel to another thread!
	pub fn send_command(cmd: SupervisorCommand) -> Result<(), ()> {
		let mut sv = match SUPERVISOR.write() {
			Ok(s) => s,
			Err(_e) => {
				warn!("supervisor poisoned, providing antidote...");
				SUPERVISOR.clear_poison();
				// failure shouldn't be possible now
				let mut sv = SUPERVISOR.write()
					.map_err(drop)?;
				sv.arcdps.clear();
				sv.external.clear();
				sv
			},
		};

		sv.handle_command(cmd);
		Ok(())
	}

	pub fn handle_command(&mut self, cmd: SupervisorCommand) {
		match cmd {
			SupervisorCommand::RefreshArcdps =>
				self.refresh_arcdps(),
			SupervisorCommand::RefreshExternal =>
				self.refresh_external(),
		}
	}

	pub fn notify_arcdps_unload(sig: NonZeroU32, dead_module: Option<HMODULE>) -> WinResult<()> {
		let mut sv = match SUPERVISOR.write() {
			Ok(s) => s,
			Err(_e) => return Err(ERROR_THREAD_WAS_SUSPENDED.into()),
		};
		match (sv.arcdps.remove(&sig), dead_module) {
			(None, _) => Err(ERROR_INDEX_OUT_OF_BOUNDS.into()),
			(Some(ext), Some(module)) if ext.module != module =>
				Err(WinError::new(ERROR_INVALID_HANDLE.to_hresult(), format!("expected handle {:?}, got {module:?}", ext.module))),
			(Some(..), _) => Ok(()),
		}
	}
}

fn get_module_name(handle: HMODULE) -> Arc<str> {
	get_module_path(Some(handle)).ok()
		.and_then(|path| Path::new(&path).file_name().map(|s| s.to_os_string()))
		.map(|s| s.to_string_lossy().into_owned())
		.unwrap_or_else(|| format!("{:?}", handle))
		.into()
}
