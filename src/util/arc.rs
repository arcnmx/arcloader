use std::{cmp::Ordering, env, ffi::{c_char, c_void, CStr, OsString}, fmt, mem::transmute, num::NonZeroU32, ops::Deref, os::windows::ffi::OsStringExt, path::{Path, PathBuf}, ptr::{self, NonNull}, slice::from_raw_parts, sync::{Arc, OnceLock}};
use arcdps::{callbacks::ArcDpsExport, exports::{self, AddExtensionResult}};
use windows::{core::{Error as WinError, Owned, Free}, Win32::{Foundation::{GetLastError, ERROR_INVALID_DLL, HMODULE}, System::LibraryLoader::GetProcAddress}};
use windows_strings::PCWSTR;

use super::win::{get_module_from_ptr, WinResult};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ArcDpsExtensionRef<'x> {
	extension: &'x ArcDpsExtension,
}

impl<'x> ArcDpsExtensionRef<'x> {
	pub unsafe fn with_exports(exports: *const ArcDpsExport) -> Self {
		let extension = exports as *const ArcDpsExtension;
		Self {
			extension: &*extension,
		}
	}

	pub fn sig(self) -> NonZeroU32 {
		unsafe {
			NonZeroU32::new_unchecked(self.sig)
		}
	}

	#[inline]
	pub fn exports(self) -> &'x ArcDpsExport {
		&self.extension.exports
	}

	#[inline]
	pub fn extension(self) -> &'x ArcDpsExtension {
		self.extension
	}

	#[inline]
	pub fn module(self) -> HMODULE {
		HMODULE(self.exports().size as usize as *mut _)
	}

	/// TODO: use arcdps version to determine max struct size
	pub fn max_size() -> usize {
		size_of::<ArcDpsExtension>()
	}
}

impl Deref for ArcDpsExtensionRef<'_> {
	type Target = ArcDpsExtension;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.extension()
	}
}

impl fmt::Display for ArcDpsExtensionRef<'_> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self.extension(), f)
	}
}

impl fmt::Debug for ArcDpsExtensionRef<'_> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self.extension(), f)
	}
}

impl PartialOrd for ArcDpsExtensionRef<'_> {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for ArcDpsExtensionRef<'_> {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering {
		let rhs = other.extension as *const ArcDpsExtension;
		(self.extension as *const ArcDpsExtension).cmp(&rhs)
	}
}

impl PartialEq for ArcDpsExtensionRef<'_> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		let rhs = other.extension as *const _;
		(self.extension as *const _) == rhs
	}
}

impl Eq for ArcDpsExtensionRef<'_> { }

unsafe impl Sync for ArcDpsExtensionRef<'_> { }
unsafe impl Send for ArcDpsExtensionRef<'_> { }

pub type ArcDpsExtensionExtra = [u32; 26];

#[repr(C)]
pub struct ArcDpsExtension {
	exports: ArcDpsExport,
	extra: ArcDpsExtensionExtra,
}

impl ArcDpsExtension {
	pub const unsafe fn new(exports: ArcDpsExport, extra: ArcDpsExtensionExtra) -> Self {
		Self {
			exports,
			extra,
		}
	}

	pub const unsafe fn with_exports(exports: ArcDpsExport) -> Self {
		Self {
			exports,
			extra: [0u32; 26],
		}
	}

	#[inline]
	pub fn exports(&self) -> &ArcDpsExport {
		&self.exports
	}

	#[inline]
	pub unsafe fn exports_mut(&mut self) -> &mut ArcDpsExport {
		&mut self.exports
	}

	/// TODO
	#[cfg(feature = "extras")]
	#[inline]
	pub fn extras(&self) -> &extras::callbacks::Export {
		&self.extra
	}

	/// TODO
	#[cfg(feature = "extras")]
	#[inline]
	pub unsafe fn extras_mut(&mut self) -> &mut extras::callbacks::Export {
		&mut self.extra
	}

	#[inline]
	pub fn extra(&self) -> &ArcDpsExtensionExtra {
		&self.extra
	}

	#[inline]
	pub fn extra_mut(&mut self) -> &mut ArcDpsExtensionExtra {
		&mut self.extra
	}

	pub fn has_extra(&self) -> bool {
		self.extra.iter().any(|&v| v != 0)
	}

	pub fn name(&self) -> &CStr {
		let s = NonNull::new(self.out_name as *mut c_char);
		s.map(|s| unsafe {
			CStr::from_ptr(s.as_ptr() as *const _)
		}).unwrap_or_default()
	}

	pub fn build(&self) -> &CStr {
		let s = NonNull::new(self.out_build as *mut c_char);
		s.map(|s| unsafe {
			CStr::from_ptr(s.as_ptr() as *const _)
		}).unwrap_or_default()
	}

	pub fn is_loaded(&self) -> bool {
		self.extra[24] != 0
	}

	#[cfg(todo)]
	pub fn init_addr(&self) -> Option<unsafe extern "system" fn(*const c_char, ImGuiContext, *mut c_void, HMODULE, Option<MallocFn>, Option<FreeFn>, u32) -> *mut c_void> {
		let handle = self.module()?;
		unsafe {
			GetProcAddress(handle, windows_strings::s!("get_init_addr"))
				.map(|f| transmute(f))
		}
	}

	pub fn release_addr(&self) -> Option<unsafe extern "system" fn()> {
		let handle = self.module()?;
		unsafe {
			GetProcAddress(handle, windows_strings::s!("get_release_addr"))
				.map(|f| transmute(f))
		}
	}

	const EXPORT_SIZE: usize = size_of::<ArcDpsExport>();
	const EXTENSION_SIZE: usize = size_of::<ArcDpsExtension>();
	pub fn module(&self) -> Option<HMODULE> {
		let handle = match self.size {
			Self::EXPORT_SIZE | Self::EXTENSION_SIZE => HMODULE(ptr::null_mut()),
			h => HMODULE(h as usize as *mut _),
		};
		match handle {
			handle if !handle.0.is_null() && !handle.is_invalid() => Some(handle),
			_ => self.module_by_export(),
		}
	}

	pub fn module_by_export(&self) -> Option<HMODULE> {
		let mut any_fn_ptr = unsafe { [
			transmute(self.exports.wnd_nofilter),
			transmute(self.exports.combat),
			transmute(self.exports.imgui),
			transmute(self.exports.options_end),
			transmute(self.exports.combat_local),
			transmute(self.exports.wnd_filter),
			transmute(self.exports.options_windows),
			NonNull::new(self.out_name as *mut c_void),
		] }.into_iter()
			.filter_map(|p| p);
			//.filter_map(|p| NonNull::new(p as *mut ()));

		let handle = any_fn_ptr.next()
			.and_then(|p| get_module_from_ptr(p.as_ptr() as *const _).transpose())
			.transpose();
		match handle {
			Ok(None) => match get_module_from_ptr(self.out_name as *const _) {
				// TODO: check if this can even be the case!
				Ok(Some(h)) if h != unsafe { exports::raw::handle() } => Some(h),
				_ => None,
			},
			Ok(h) => h,
			Err(_e) => {
				#[cfg(feature = "log")] {
					log::warn!("failed to determine handle for {}: {}", self, _e);
				}
				None
			},
		}
	}
}

impl Deref for ArcDpsExtension {
	type Target = ArcDpsExport;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.exports
	}
}

impl Clone for ArcDpsExtension {
	fn clone(&self) -> Self {
		Self {
			exports: unsafe { ptr::read(&self.exports) },
			extra: self.extra.clone(),
		}
	}
}

impl fmt::Display for ArcDpsExtension {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let build = self.build();
		let name = self.name().to_string_lossy();

		match build.is_empty() {
			true => fmt::Display::fmt(&name, f),
			false => {
				let build = build.to_string_lossy();
				fmt::Display::fmt(&format_args!("{} {}", name, build), f)
			},
		}
	}
}

impl fmt::Debug for ArcDpsExtension {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut f = f.debug_struct("ArcDpsExtension");
		f
			.field("size", &self.size)
			.field("sig", &self.sig)
			.field("imgui_version", &self.imgui_version)
			.field("out_name", &self.name())
			.field("out_build", &self.build())
			.field("wnd_nofilter", &self.wnd_nofilter)
			.field("wnd_filter", &self.wnd_filter)
			.field("combat", &self.combat)
			.field("combat_local", &self.combat_local)
			.field("imgui", &self.imgui)
			.field("options_end", &self.options_end)
			.field("options_windows", &self.options_windows);

		if self.has_extra() {
			match () {
				#[cfg(feature = "extras")]
				_ => f.field("extras", self.extras()),
				#[cfg(not(feature = "extras"))]
				_ => f.field("extra", self.extra()),
			};
		}

		f.finish()
	}
}

pub fn config_path<R, F: FnOnce(Option<&[u16]>) -> R>(f: F) -> R {
	let path = match exports::has_e0_config_path() {
		true => unsafe {
			let p = PCWSTR::from_raw(exports::raw::e0_config_path());
			let os = from_raw_parts(p.as_ptr(), p.len());
			Some(os)
		},
		false => None,
	};

	f(path)
}

pub fn config_dir() -> Option<PathBuf> {
	config_path(|ini| {
		let mut path = PathBuf::from(OsString::from_wide(ini?));
		match path.pop() {
			false => None,
			true => Some(path),
		}
	})
}

static GAME_DIR: OnceLock<Option<Arc<Path>>> = OnceLock::new();

pub fn game_dir() -> Option<&'static Path> {
	let dir = GAME_DIR.get_or_init(||
		env::current_dir().ok()
		.map(|p| Arc::from(p))
	);

	dir.as_ref().map(|p| &**p)
}

pub fn add_extension(mut handle: Owned<HMODULE>) -> WinResult<()> {
	let res = exports::add_extension(*handle);
	unsafe {
		// XXX: arcdps increments handle refcounts by 2???
		handle.free();
		drop(handle);
	}
	match res {
		AddExtensionResult::Ok => Ok(()),
		AddExtensionResult::LoadError => {
			// TODO: does not free library after?
			Err(unsafe { GetLastError().into() })
		},
		failure => Err(WinError::new(ERROR_INVALID_DLL.to_hresult(), format!("{failure:?}"))),
	}
}

pub fn remove_extension(sig: NonZeroU32) -> Result<HMODULE, ()> {
	if !exports::has_remove_extension() {
		return Err(())
	}
	exports::remove_extension(sig.get())
		.ok_or(())
}
