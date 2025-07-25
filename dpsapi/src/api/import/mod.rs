use std::{error::Error as StdError, path::PathBuf};
use core::{fmt, mem::transmute, ptr};
use arcffi::{c_void, cstr::{cstr, CStr, CStrRef, CStrRef16, CStrPtr}, NonNull};
#[cfg(feature = "windows")]
use arcffi::windows::{
	WinResult, winerror,
	library::get_proc_address,
};
#[cfg(all(windows, feature = "windows"))]
use windows::core::Free as WinFree;
use crate::{
	api::header::ExtensionExports,
	sig::Signature,
	combat::CombatEventData,
};

pub use arcffi::windows::Win32::Foundation::HMODULE;

pub type ImVec4 = [f32; 4];

pub use self::{
	ExportFn0 as ExportIniPathFn,
	ExportFn3 as ExportLogFn,
	ExportFn5 as ExportUiColoursFn,
	ExportFn6 as ExportUiFlagsFn,
	ExportFn7 as ExportUiModifiersFn,
	ExportFn8 as ExportLogConsoleFn,
	ExportFn9 as ExportEvtcProcessFn,
	ExportFn10 as ExportEvtcProcessSkillFn,
};

pub type ExportFn0 = unsafe extern "system" fn() -> Option<NonNull<CStrRef16>>;

pub type ExportFn3 = for<'m> unsafe extern "system" fn(message: Option<CStrPtr<'m>>);

pub type Colours5 = [Option<NonNull<ImVec4>>; 5];
pub type ExportFn5 = for<'c> unsafe extern "system" fn(colours: &'c mut Colours5);
pub type ExportFn6 = unsafe extern "system" fn() -> u64;
pub type ExportFn7 = unsafe extern "system" fn() -> u64;

pub type ExportFn8 = ExportFn3;

pub type ExportFn9 = for<'e> unsafe extern "system" fn(event: Option<&'e CombatEventData>, sig: Signature);
pub type ExportFn10 = ExportFn9;

pub type ExtensionListCallbackFn = for<'x> extern "C" fn(exp: &'x ExtensionExports<'x>);
pub type ExportFnExtensionList = unsafe extern "system" fn(callback: Option<ExtensionListCallbackFn>);
pub type ExportFnExtensionAdd2 = unsafe extern "system" fn(module: HMODULE) -> usize;
pub type ExportFnExtensionRemove2 = unsafe extern "system" fn(sig: Signature) -> HMODULE;

pub trait ApiExports {
	type ImportError;

	/// API returned an empty result
	fn import_error_empty(&self) -> Self::ImportError;

	fn arc_ini_path<R, F: for<'p> FnOnce(&'p CStrRef16) -> R>(&self, f: F) -> Result<R, Self::ImportError>;
	fn arc_log(&self, message: &CStrRef) -> Result<(), Self::ImportError>;
	fn arc_log_window(&self, message: &CStrRef) -> Result<(), Self::ImportError>;
	fn arc_ui_colours<R, F: FnOnce(&[Option<&ImVec4>; 5]) -> R>(&self, f: F) -> Result<R, Self::ImportError>;
	fn arc_ui_flags(&self) -> Result<u64, Self::ImportError>;
	fn arc_ui_modifiers(&self) -> Result<u64, Self::ImportError>;
	fn arc_combat_event(&self, ev: &CombatEventData, sig: Signature) -> Result<(), Self::ImportError>;
	fn arc_combat_event_skill(&self, ev: &CombatEventData, sig: Signature) -> Result<(), Self::ImportError>;
	fn arc_extension_list(&self, f: ExtensionListCallbackFn) -> Result<(), Self::ImportError>;
	unsafe fn arc_extension_add2(&self, module: HMODULE) -> Result<usize, Self::ImportError>;
	unsafe fn arc_extension_remove2(&self, sig: Signature) -> Result<HMODULE, Self::ImportError>;

	fn get_ini_path(&self) -> Result<PathBuf, Self::ImportError> {
		self.arc_ini_path(|p| PathBuf::from(p.to_os_string()))
	}

	#[inline]
	fn log<M: AsRef<CStrRef>>(&self, message: M) -> Result<(), Self::ImportError> {
		self.arc_log(message.as_ref())
	}

	#[inline]
	fn log_window<M: AsRef<CStrRef>>(&self, message: M) -> Result<(), Self::ImportError> {
		self.arc_log_window(message.as_ref())
	}

	fn get_ui_colours(&self) -> Result<[Option<ImVec4>; 5], Self::ImportError> {
		self.arc_ui_colours(|colours| {
			let [c0, c1, c2, c3, c4] = colours;
			[c0.copied(), c1.copied(), c2.copied(), c3.copied(), c4.copied()]
		})
	}

	#[inline]
	fn extension_list<F>(&self, mut f: F) -> Result<(), Self::ImportError> where
		F: for<'x> FnMut(&'x ExtensionExports<'x>),
	{
		self.extension_list_dyn(&mut f)
	}

	fn extension_list_dyn(&self, f: &mut dyn ExtensionListCallback) -> Result<(), Self::ImportError> {
		use core::cell::Cell;

		thread_local! {
			static LIST_EXTENSION_CB: Cell<Option<*mut dyn ExtensionListCallback>> = Cell::new(None);
		}

		#[inline(never)]
		extern "C" fn list_extension_cb(export: &ExtensionExports) {
			let cb = match LIST_EXTENSION_CB.get() {
				Some(cb) => unsafe { &mut *cb },
				None => {
					// XXX: should never happen...
					return
				},
			};
			cb.callback(export)
		}

		struct ListExtensionsGuard;

		impl Drop for ListExtensionsGuard {
			fn drop(&mut self) {
				let _cb = LIST_EXTENSION_CB.take();
				debug_assert!(_cb.is_some());
			}
		}

		match LIST_EXTENSION_CB.get() {
			#[cfg(debug_assertions)]
			Some(..) => panic!("extension_list re-entered"),
			#[cfg(not(debug_assertions))]
			Some(..) => {
				#[cfg(feature = "log")] {
					::log::error!("extension_list re-entered");
				}
				return Err(self.import_error_empty())
			},
			None => (),
		}
		let _guard = ListExtensionsGuard;

		let cb: *mut dyn ExtensionListCallback = f as *mut (dyn ExtensionListCallback + '_) as *mut (dyn ExtensionListCallback + 'static);
		LIST_EXTENSION_CB.set(Some(cb));
		self.arc_extension_list(list_extension_cb)
	}
}

#[derive(Debug, Copy, Clone)]
pub struct ApiMissing;
impl fmt::Display for ApiMissing {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("arcdps export unavailable")
	}
}
impl StdError for ApiMissing {
}

pub trait ExtensionListCallback {
	fn callback<'x>(&mut self, exp: &'x ExtensionExports);
}

impl<F: for<'x> FnMut(&'x ExtensionExports<'x>)> ExtensionListCallback for F {
	fn callback<'x>(&mut self, exp: &'x ExtensionExports) {
		(*self)(exp)
	}
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct ModuleExports {
	module: HMODULE,
}

impl ModuleExports {
	pub const INVALID: Self = unsafe {
		Self::new(HMODULE(ptr::null_mut()))
	};

	pub const unsafe fn new(module: HMODULE) -> Self {
		Self {
			module,
		}
	}

	pub const unsafe fn new_ref(module: &HMODULE) -> &Self {
		unsafe {
			transmute(module)
		}
	}

	pub unsafe fn new_mut(module: &mut HMODULE) -> &mut Self {
		unsafe {
			transmute(module)
		}
	}

	pub fn module(&self) -> &HMODULE {
		&self.module
	}

	pub unsafe fn module_mut(&mut self) -> &mut HMODULE {
		&mut self.module
	}

	pub fn lookup_e0(&self) -> Option<ExportFn0> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E0)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_e3(&self) -> Option<ExportFn3> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E3)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_e5(&self) -> Option<ExportFn5> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E5)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_e6(&self) -> Option<ExportFn6> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E6)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_e7(&self) -> Option<ExportFn7> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E7)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_e8(&self) -> Option<ExportFn8> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E8)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_e9(&self) -> Option<ExportFn9> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E9)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_e10(&self) -> Option<ExportFn10> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_E10)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_extension_list(&self) -> Option<ExportFnExtensionList> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_EXTENSION_LIST)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_extension_add2(&self) -> Option<ExportFnExtensionAdd2> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_EXTENSION_ADD2)
				.map(|f| transmute(f))
		}
	}
	pub fn lookup_extension_remove2(&self) -> Option<ExportFnExtensionRemove2> {
		unsafe {
			Self::lookup_export(self.module, Self::SYM_EXTENSION_REMOVE2)
				.map(|f| transmute(f))
		}
	}
}

impl ApiExports for ModuleExports {
	type ImportError = ApiMissing;

	#[inline]
	fn import_error_empty(&self) -> Self::ImportError {
		ApiMissing
	}

	fn arc_ini_path<R, F: for<'p> FnOnce(&'p CStrRef16) -> R>(&self, f: F) -> Result<R, Self::ImportError> {
		unsafe {
			self.lookup_e0().map(|e0| e0())
			.ok_or(self.import_error_empty())
			.and_then(|p| p.ok_or_else(|| self.import_error_empty()))
			.map(move |p| f(&*p.as_ptr()))
		}
	}

	fn arc_log(&self, message: &CStrRef) -> Result<(), Self::ImportError> {
		unsafe {
			self.lookup_e3()
				.map(|e| e(Some(message.as_c_ptr())))
				.ok_or(self.import_error_empty())
		}
	}

	fn arc_log_window(&self, message: &CStrRef) -> Result<(), Self::ImportError> {
		unsafe {
			self.lookup_e8()
				.map(|e| e(Some(message.as_c_ptr())))
				.ok_or(self.import_error_empty())
		}
	}

	fn arc_ui_colours<R, F: FnOnce(&[Option<&ImVec4>; 5]) -> R>(&self, f: F) -> Result<R, Self::ImportError> {
		let mut colours = [Default::default(); 5];
		let colours: &[Option<&_>; 5] = unsafe {
			self.lookup_e5()
				.map(|e| e(&mut colours))
				.ok_or(self.import_error_empty())?;
			transmute(&colours)
		};

		Ok(f(colours))
	}

	fn arc_ui_flags(&self) -> Result<u64, Self::ImportError> {
		unsafe {
			self.lookup_e6()
				.map(|e| e())
				.ok_or(self.import_error_empty())
		}
	}

	fn arc_ui_modifiers(&self) -> Result<u64, Self::ImportError> {
		unsafe {
			self.lookup_e7()
				.map(|e| e())
				.ok_or(self.import_error_empty())
		}
	}

	fn arc_combat_event(&self, ev: &CombatEventData, sig: Signature) -> Result<(), Self::ImportError> {
		unsafe {
			self.lookup_e9()
				.map(|e| e(Some(ev), sig))
				.ok_or(self.import_error_empty())
		}
	}

	fn arc_combat_event_skill(&self, ev: &CombatEventData, sig: Signature) -> Result<(), Self::ImportError> {
		unsafe {
			self.lookup_e10()
				.map(|e| e(Some(ev), sig))
				.ok_or(self.import_error_empty())
		}
	}

	fn arc_extension_list(&self, f: ExtensionListCallbackFn) -> Result<(), Self::ImportError> {
		unsafe {
			self.lookup_extension_list()
				.map(|e| e(Some(f)))
				.ok_or(self.import_error_empty())
		}
	}

	unsafe fn arc_extension_add2(&self, module: HMODULE) -> Result<usize, Self::ImportError> {
		unsafe {
			self.lookup_extension_add2()
				.map(|e| e(module))
				.ok_or(self.import_error_empty())
		}
	}

	unsafe fn arc_extension_remove2(&self, sig: Signature) -> Result<HMODULE, Self::ImportError> {
		unsafe {
			self.lookup_extension_remove2()
				.map(|e| e(sig))
				.ok_or(self.import_error_empty())
		}
	}
}

impl ApiExports for &'_ ModuleExports {
	type ImportError = ApiMissing;

	#[inline]
	fn import_error_empty(&self) -> Self::ImportError {
		ApiMissing
	}

	fn arc_ini_path<R, F: for<'p> FnOnce(&'p CStrRef16) -> R>(&self, f: F) -> Result<R, Self::ImportError> {
		ApiExports::arc_ini_path(*self, f)
	}

	fn arc_log(&self, message: &CStrRef) -> Result<(), Self::ImportError> {
		ApiExports::arc_log(*self, message)
	}

	fn arc_log_window(&self, message: &CStrRef) -> Result<(), Self::ImportError> {
		ApiExports::arc_log_window(*self, message)
	}

	fn arc_ui_colours<R, F: FnOnce(&[Option<&ImVec4>; 5]) -> R>(&self, f: F) -> Result<R, Self::ImportError> {
		ApiExports::arc_ui_colours(*self, f)
	}

	fn arc_ui_flags(&self) -> Result<u64, Self::ImportError> {
		ApiExports::arc_ui_flags(*self)
	}

	fn arc_ui_modifiers(&self) -> Result<u64, Self::ImportError> {
		ApiExports::arc_ui_modifiers(*self)
	}

	fn arc_combat_event(&self, ev: &CombatEventData, sig: Signature) -> Result<(), Self::ImportError> {
		ApiExports::arc_combat_event(*self, ev, sig)
	}

	fn arc_combat_event_skill(&self, ev: &CombatEventData, sig: Signature) -> Result<(), Self::ImportError> {
		ApiExports::arc_combat_event_skill(*self, ev, sig)
	}

	fn arc_extension_list(&self, f: ExtensionListCallbackFn) -> Result<(), Self::ImportError> {
		ApiExports::arc_extension_list(*self, f)
	}

	unsafe fn arc_extension_add2(&self, module: HMODULE) -> Result<usize, Self::ImportError> {
		ApiExports::arc_extension_add2(*self, module)
	}

	unsafe fn arc_extension_remove2(&self, sig: Signature) -> Result<HMODULE, Self::ImportError> {
		ApiExports::arc_extension_remove2(*self, sig)
	}
}

unsafe impl Sync for ModuleExports {}
unsafe impl Send for ModuleExports {}

#[cfg(all(windows, feature = "windows"))]
impl WinFree for ModuleExports {
	unsafe fn free(&mut self) {
		WinFree::free(&mut self.module)
	}
}

impl ModuleExports {
	pub unsafe fn lookup_export_win32(module: HMODULE, sym: CStrPtr) -> Option<NonNull<c_void>> {
		match (module, sym) {
			#[cfg(feature = "windows")]
			(module, sym) if !module.is_invalid() => {
				get_proc_address(module, sym)
					.map(|f| transmute(f))
			},
			_ => {
				#[cfg(all(windows, not(feature = "windows"), feature = "log"))] {
					log::warn!("dpsapi lacks windows platform features to lookup {sym}");
				}
				None
			},
		}
	}

	#[cfg(feature = "windows")]
	pub unsafe fn try_lookup_export_win32(module: HMODULE, sym: CStrPtr) -> WinResult<NonNull<c_void>> {
		Self::lookup_export_win32(module, sym)
			.ok_or_else(|| winerror!(ERROR_NOT_FOUND, fmt:"arcdps export {sym:?} not found"))
	}

	pub unsafe fn lookup_export<'a, P: Into<CStrPtr<'a>>>(module: HMODULE, sym: P) -> Option<NonNull<c_void>> {
		let sym = sym.into();
		match () {
			#[cfg(windows)]
			() => Self::lookup_export_win32(module, sym.into()),
			#[cfg(not(windows))]
			_ => {
				#[cfg(feature = "log")] {
					log::debug!("TODO: lookup_export({sym:?})");
				}
				None
			},
		}
	}

	pub const SYM_E0: &'static CStr = cstr!("e0");
	pub const SYM_E3: &'static CStr = cstr!("e3");
	pub const SYM_E5: &'static CStr = cstr!("e5");
	pub const SYM_E6: &'static CStr = cstr!("e6");
	pub const SYM_E7: &'static CStr = cstr!("e7");
	pub const SYM_E8: &'static CStr = cstr!("e8");
	pub const SYM_E9: &'static CStr = cstr!("e9");
	pub const SYM_E10: &'static CStr = cstr!("e10");
	pub const SYM_EXTENSION_LIST: &'static CStr = cstr!("listextension");
	pub const SYM_EXTENSION_ADD2: &'static CStr = cstr!("addextension2");
	pub const SYM_EXTENSION_REMOVE2: &'static CStr = cstr!("removeextension2");
}

#[test]
fn extension_list() {
	let exports = ModuleExports::INVALID;
	let mut extensions = Vec::new();
	let _res = exports.extension_list(|ex| {
		extensions.push(ex.sig());
	});
	assert!(extensions.is_empty());
}
