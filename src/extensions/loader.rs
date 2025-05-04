use crate::{supervisor::Supervisor, util::{arc::{add_extension, remove_extension}, win::{load_library, WinResult}}};
use std::num::NonZeroU32;
#[cfg(feature = "arcdps-extras")]
use arcdps::extras::{ExtrasAddonInfo, UserInfoIter};
use windows::{
	core::{Error as WinError, Owned},
	Win32::{Foundation::{ERROR_CALL_NOT_IMPLEMENTED, ERROR_NOT_FOUND, HMODULE}, System::LibraryLoader::GetProcAddress},
};
use windows_strings::HSTRING;

#[derive(Debug)]
pub enum LoaderCommand {
	LoaderExit,
	LoaderReload,
	LoadPath {
		path: HSTRING
	},
	LoadModule {
		module: Owned<HMODULE>,
		loader: Option<AddonLoader>,
	},
	Unload {
		sig: NonZeroU32,
		//retain_module: bool,
	},
	Reload {
		module: Owned<HMODULE>,
	},
}

#[derive(Copy, Clone, Debug)]
pub enum AddonLoader {
	Arcdps,
	NexusHost,
}

pub struct Loader {
}

impl Loader {
	pub fn init() {
	}

	pub fn unload() {
	}

	pub fn send_command(cmd: LoaderCommand) -> WinResult<()> {
		match cmd {
			LoaderCommand::LoadPath { path } => {
				// TODO: load with LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR or something idk
				let module = load_library(&path)?;
				let loader = match () {
					_ if path.to_string_lossy().contains(crate::supervisor::ExtDir::INFIX_ARCDPS) => Some(AddonLoader::Arcdps),
					_ if path.to_string_lossy().contains(crate::supervisor::ExtDir::INFIX_NEXUS) => Some(AddonLoader::NexusHost),
					_ => None,
				};
				let res = Self::send_command(LoaderCommand::LoadModule { module, loader });
				res
			},
			LoaderCommand::LoadModule { module, loader } => {
				let loader = match loader {
					Some(l) => l,
					None if unsafe { GetProcAddress(*module, windows_strings::s!("get_init_addr")).is_some() } =>
						AddonLoader::Arcdps,
					//None if unsafe { GetProcAddress(*module, windows_strings::s!("arcdps_identifier_export")).is_some() } => AddonLoader::HostArcdps,
					None if unsafe { GetProcAddress(*module, windows_strings::s!("GetAddonDef")).is_some() } =>
						AddonLoader::NexusHost,
					_ => AddonLoader::Arcdps,
				};

				#[allow(unreachable_patterns)]
				match loader {
					AddonLoader::Arcdps => add_extension(module),
					#[cfg(feature = "host-addonapi")]
					AddonLoader::NexusHost => crate::extensions::nexus::NexusHost::enumerate_addon(module).map(drop),
					_ => return Err(WinError::new(ERROR_CALL_NOT_IMPLEMENTED.to_hresult(), format!("arcloader {:?} support disabled", loader))),
				}
			},
			LoaderCommand::Reload { module } => {
				Err(ERROR_CALL_NOT_IMPLEMENTED.into())
			},
			LoaderCommand::Unload { sig } => {
				#[cfg(todo)]
				if retain_handle {
					forget(retain_library(module));
				}

				let module = match remove_extension(sig) {
					Err(()) => Err(ERROR_NOT_FOUND),
					Ok(handle) => {
						#[cfg(feature = "log")] {
							info!("removed {sig:08x}");
						}
						Ok(handle)
					},
				}?;

				if let Err(_e) = Supervisor::notify_arcdps_unload(sig, Some(module)) {
					#[cfg(feature = "log")] {
						warn!("unload failed to notify supervisor: {_e}");
					}
				}

				Ok(())
			},
			LoaderCommand::LoaderExit => Err(ERROR_CALL_NOT_IMPLEMENTED.into()),
			LoaderCommand::LoaderReload => Err(ERROR_CALL_NOT_IMPLEMENTED.into()),
		}
	}
}
