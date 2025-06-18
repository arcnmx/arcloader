use core::{fmt, hash, mem::{size_of, transmute}, num::{NonZeroUsize, NonZeroU64}, ptr};
use arcffi::{c_bool32, c_void, cstr::{cstr, CStr, CStrPtr, CStrPtr16}, NonNull, UserMallocFn, UserFreeFn};
#[cfg(feature = "dyload")]
#[cfg(feature = "windows")]
use dyload::windows::{
	WinResult,
	winerror,
};
#[cfg(all(windows, feature = "windows"))]
use windows::{
	Win32::System::LibraryLoader::GetProcAddress,
	core::Free as WinFree,
};
use crate::{
	sig::{Signature, Sig, SigRepr},
	combat::{CombatArgs, CombatEventData, CombatAgent},
};

pub use arcffi::windows::adapter::{HMODULE, HWND, LPARAM, WPARAM};

pub type ImVec4 = [f32; 4];

pub type GetInitFn = unsafe extern "system" fn(
	arc_version: CStrPtr<'static>,
	imgui_ctx: Option<NonNull<c_void>>,
	id3d: Option<NonNull<c_void>>,
	module: ModuleExports,
	malloc: Option<UserMallocFn>,
	free: Option<UserFreeFn>,
	d3d_version: u32,
) -> Option<InitFn>;
pub type GetReleaseFn = unsafe extern "system" fn() -> Option<ReleaseFn>;
pub type InitFn = unsafe extern "C" fn() -> Option<NonNull<ExtensionExports<'static>>>;
pub type ReleaseFn = unsafe extern "C" fn();

pub struct InitArgs {
	pub arc_version: CStrPtr<'static>,
	pub imgui_ctx: Option<NonNull<c_void>>,
	pub id3d: Option<NonNull<c_void>>,
	pub module: ModuleExports,
	pub malloc: Option<UserMallocFn>,
	pub free: Option<UserFreeFn>,
	pub d3d_version: u32,
}

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

pub type ExportFn0 = unsafe extern "system" fn() -> Option<NonNull<u16>>;

pub type ExportFn3_<'m> = unsafe extern "system" fn(message: CStrPtr<'m>);
pub type ExportFn3 = for<'m> unsafe extern "system" fn(message: CStrPtr<'m>);

pub type Colours5 = [Option<NonNull<ImVec4>>; 5];
pub type ExportFn5 = for<'c> unsafe extern "system" fn(colours: &'c mut Colours5);
pub type ExportFn6 = unsafe extern "system" fn() -> u64;
pub type ExportFn7 = unsafe extern "system" fn() -> u64;

pub type ExportFn8 = ExportFn3;

pub type ExportFn9 = for<'e> unsafe extern "system" fn(event: Option<&'e CombatEventData>, sig: Signature);
pub type ExportFn10 = ExportFn9;

pub type ExtensionListCallbackFn = for<'x> extern "system" fn(exp: &'x ExtensionExports);
pub type ExportFnExtensionList = unsafe extern "system" fn(callback: Option<ExtensionListCallbackFn>);
pub type ExportFnExtensionAdd2 = unsafe extern "system" fn(module: HMODULE) -> usize;
pub type ExportFnExtensionRemove2 = unsafe extern "system" fn(sig: Signature) -> HMODULE;

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

	pub fn arc_ini_path<R, F: for<'p> FnOnce(CStrPtr16<'p>) -> R>(&self, f: F) -> Result<R, ()> {
		unsafe {
			match self.lookup_e0().and_then(|e0| e0()) {
				Some(ini_path) => Ok(f(CStrPtr16::new(ini_path))),
				None => Err(()),
			}
		}
	}

	pub fn arc_log<P: for<'m> Into<CStrPtr<'m>>>(&self, message: P) -> Result<(), ()> {
		let message = message.into();
		unsafe {
			self.lookup_e3()
				.map(|e| e(message))
				.ok_or(())
		}
	}

	pub fn arc_log_window<P: for<'m> Into<CStrPtr<'m>>>(&self, message: P) -> Result<(), ()> {
		let message = message.into();
		unsafe {
			self.lookup_e8()
				.map(|e| e(message))
				.ok_or(())
		}
	}

	pub fn arc_ui_colours<R, F: FnOnce(&[Option<&ImVec4>; 5]) -> R>(&self, f: F) -> Result<R, ()> {
		let mut colours = [Default::default(); 5];
		let colours: &[Option<&_>; 5] = unsafe {
			self.lookup_e5()
				.map(|e| e(&mut colours))
				.ok_or(())?;
			transmute(&colours)
		};

		Ok(f(colours))
	}

	pub fn arc_ui_flags(&self) -> Result<u64, ()> {
		unsafe {
			self.lookup_e6()
				.map(|e| e())
				.ok_or(())
		}
	}

	pub fn arc_ui_modifiers(&self) -> Result<u64, ()> {
		unsafe {
			self.lookup_e7()
				.map(|e| e())
				.ok_or(())
		}
	}

	pub fn arc_combat_event(&self, ev: &CombatEventData, sig: Signature) -> Result<(), ()> {
		unsafe {
			self.lookup_e9()
				.map(|e| e(Some(ev), sig))
				.ok_or(())
		}
	}

	pub fn arc_combat_event_skill(&self, ev: &CombatEventData, sig: Signature) -> Result<(), ()> {
		unsafe {
			self.lookup_e10()
				.map(|e| e(Some(ev), sig))
				.ok_or(())
		}
	}

	pub fn arc_extension_list(&self, f: ExtensionListCallbackFn) -> Result<(), ()> {
		unsafe {
			self.lookup_extension_list()
				.map(|e| e(Some(f)))
				.ok_or(())
		}
	}

	pub unsafe fn arc_extension_add2(&self, module: HMODULE) -> Result<usize, ()> {
		unsafe {
			self.lookup_extension_add2()
				.map(|e| e(module))
				.ok_or(())
		}
	}

	pub unsafe fn arc_extension_remove2(&self, sig: Signature) -> Result<HMODULE, ()> {
		unsafe {
			self.lookup_extension_remove2()
				.map(|e| e(sig))
				.ok_or(())
		}
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

#[cfg(all(windows, feature = "windows"))]
impl WinFree for ModuleExports {
	unsafe fn free(&mut self) {
		WinFree::free(&mut self.module)
	}
}

impl ModuleExports {
	#[cfg(windows)]
	pub unsafe fn lookup_export_win32(module: HMODULE, sym: CStrPtr) -> Option<NonNull<c_void>> {
		match (module, sym) {
			#[cfg(all(windows, feature = "windows"))]
			(module, sym) if !module.is_invalid() => {
				GetProcAddress(module.into(), sym)
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

	#[cfg(feature = "dyload")]
	#[cfg(all(windows, feature = "windows"))]
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

pub type ExtensionFnCombat = for<'c, 's> unsafe extern "C" fn(
	ev: Option<&'c CombatEventData>,
	src: Option<&'c CombatAgent>,
	dst: Option<&'c CombatAgent>,
	skill_name: Option<CStrPtr<'s>>,
	id: Option<NonZeroU64>,
	revision: u64,
);
pub type ExtensionFnWnd = unsafe extern "C" fn(wnd: *mut c_void, msg: u32, w: usize, l: isize);
pub type ExtensionFnUiImgui = unsafe extern "C" fn(not_charset_or_loading: c_bool32, hide_if_combat_or_ooc: c_bool32);
pub type ExtensionFnUiOptionsTab = unsafe extern "C" fn();
pub type ExtensionFnUiOptionsWindows = for<'w> unsafe extern "C" fn(window_name: Option<CStrPtr<'w>>);

#[cfg(feature = "arcdps")]
pub use arcdps::callbacks::ArcDpsExport as ImpExtensionExports;

#[doc(alias = "arcdps_exports")]
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash)]
pub struct ExtensionExports<'s> {
	pub header: ExtensionHeader<'s>,
	pub name: Option<CStrPtr<'s>>,
	pub build: Option<CStrPtr<'s>>,
	#[doc(alias = "wnd_nofilter")]
	pub cb_wnd: Option<ExtensionFnWnd>,
	#[doc(alias = "combat")]
	pub cb_combat: Option<ExtensionFnCombat>,
	#[doc(alias = "imgui")]
	pub cb_ui_imgui: Option<ExtensionFnUiImgui>,
	#[doc(alias = "options_tab")]
	pub cb_ui_options_tab: Option<ExtensionFnUiOptionsTab>,
	#[doc(alias = "combat_local")]
	pub cb_combat_local: Option<ExtensionFnCombat>,
	#[doc(alias = "wnd_filter")]
	pub cb_wnd_filter: Option<ExtensionFnWnd>,
	#[doc(alias = "options_windows")]
	pub cb_ui_options_windows: Option<ExtensionFnUiOptionsWindows>,
}

impl<'s> ExtensionExports<'s> {
	pub const SIZE: usize = size_of::<Self>();
	pub const EMPTY: Self = Self {
		header: ExtensionHeader::EMPTY,
		name: None,
		build: None,
		cb_wnd: None,
		cb_combat: None,
		cb_ui_imgui: None,
		cb_ui_options_tab: None,
		cb_combat_local: None,
		cb_wnd_filter: None,
		cb_ui_options_windows: None,
	};

	pub const fn new_failed(error_message: Option<CStrPtr<'s>>) -> Self {
		Self {
			header: ExtensionHeader::new_failed(error_message),
			.. Self::EMPTY
		}
	}

	#[inline]
	pub fn sig(&self) -> Signature {
		self.header.sig().copied()
	}

	#[inline]
	pub fn imgui_version(&self) -> u32 {
		*self.header.imgui_version()
	}

	#[inline]
	pub fn load_status(&self) -> Result<Sig, Option<CStrPtr<'s>>> {
		match self.header.load_status() {
			Ok(&s) => Ok(s),
			Err(failed) => Err(failed.error),
		}
	}

	pub unsafe fn lookup_init(module: HMODULE) -> Option<GetInitFn> {
		ModuleExports::lookup_export(module, ExtensionExports::SYM_GET_INIT)
			.map(|f| unsafe { transmute(f) })
	}

	pub unsafe fn lookup_release(module: HMODULE) -> Option<GetReleaseFn> {
		ModuleExports::lookup_export(module, ExtensionExports::SYM_GET_RELEASE)
			.map(|f| unsafe { transmute(f) })
	}

	pub unsafe fn get_release(module: HMODULE) -> Result<Option<ReleaseFn>, ()> {
		Self::lookup_release(module)
			.map(|f| f())
			.ok_or(())
	}

	pub unsafe fn get_init(module: HMODULE, args: InitArgs) -> Result<Option<InitFn>, ()> {
		Self::lookup_init(module)
			.map(|f| f(
				args.arc_version,
				args.imgui_ctx,
				args.id3d,
				args.module,
				args.malloc,
				args.free,
				args.d3d_version,
			)).ok_or(())
	}
}

impl ExtensionExports<'static> {
	pub unsafe extern "system" fn wrap_init_fn<F, R>(
		arc_version: CStrPtr<'static>,
		imgui_ctx: Option<NonNull<c_void>>,
		id3d: Option<NonNull<c_void>>,
		module: ModuleExports,
		malloc: Option<UserMallocFn>,
		free: Option<UserFreeFn>,
		d3d_version: u32,
	) -> Option<InitFn> where
		F: Fn(InitArgs) -> R,
		R: Into<Option<InitFn>>
	{
		let f = ();
		let f: F = unsafe {
			arcffi::transmute_unchecked(f)
		};

		f(InitArgs {
			arc_version,
			imgui_ctx,
			id3d,
			module,
			malloc,
			free,
			d3d_version,
		}).into()
	}

	pub const fn wrap_init_fn_item<F, R>(_f: &'_ F) -> GetInitFn where
		F: Fn(InitArgs) -> R + 'static,
		R: Into<Option<InitFn>>
	{
		match size_of::<F>() {
			0 => Self::wrap_init_fn::<F, R>,
			_ => panic!("init_fn is required to be a ZST fn item type"),
		}
	}

	pub unsafe extern "C" fn wrap_combat_fn<'c, 's, F>(
		ev: Option<&'c CombatEventData>,
		src: Option<&'c CombatAgent>,
		dst: Option<&'c CombatAgent>,
		skill_name: Option<CStrPtr<'s>>,
		id: Option<NonZeroU64>,
		revision: u64,
	) where
		F: Fn(CombatArgs),
	{
		use std::borrow::Cow;

		let f = ();
		let f: F = unsafe {
			arcffi::transmute_unchecked(f)
		};

		let args = CombatArgs {
			ev: ev.map(Cow::Borrowed),
			src: src.map(Cow::Borrowed),
			dst: dst.map(Cow::Borrowed),
			skill_name: Cow::Borrowed(skill_name.unwrap_or(CStrPtr::EMPTY).as_c_ref()),
			id,
			revision,
		};
		f(args)
	}

	pub const fn wrap_combat_fn_item<F>(_f: &'_ F) -> ExtensionFnCombat where
		F: Fn(CombatArgs) + 'static,
	{
		match size_of::<F>() {
			0 => Self::wrap_combat_fn::<F> as ExtensionFnCombat,
			_ => panic!("combat_fn is required to be a ZST fn item type"),
		}
	}

	pub const SYM_GET_INIT: &'static CStr = cstr!("get_init_addr");
	pub const SYM_GET_RELEASE: &'static CStr = cstr!("get_release_addr");
}

#[test]
fn wrap_init_fn_const() {
	const ARG_CHECK: u32 = 10;
	const ERR_CHECK: &'static CStr = cstr!("hello");
	const EXT_CHECK: ExtensionExports<'static> = ExtensionExports::new_failed(Some(CStrPtr::with_cstr(ERR_CHECK)));

	fn get_init_fn(args: InitArgs) -> Option<InitFn> {
		match args.d3d_version {
			ARG_CHECK => Some(init_fn),
			_ => None
		}
	}

	extern "C" fn init_fn() -> Option<NonNull<ExtensionExports<'static>>> {
		Some(arcffi::nonnull_ref(&EXT_CHECK))
	}

	const FN: GetInitFn = ExtensionExports::wrap_init_fn_item(&get_init_fn);
	let init = unsafe {
		FN(CStrPtr::EMPTY, None, None, ModuleExports::INVALID, None, None, ARG_CHECK).unwrap()
	};

	let exports = unsafe { &*init().unwrap().as_ptr() };

	assert_eq!(exports.load_status(), Err(Some(CStrPtr::with_cstr(ERR_CHECK))));
}

#[cfg(feature = "arcdps")]
impl<'s> ExtensionExports<'s> {
	pub unsafe fn to_imp(self) -> ImpExtensionExports {
		transmute(self)
	}

	#[cfg(feature = "arcdps")]
	pub unsafe fn as_imp(&self) -> &ImpExtensionExports {
		transmute(self)
	}

	#[cfg(feature = "arcdps")]
	pub unsafe fn from_imp(exports: ImpExtensionExports) -> Self {
		transmute(exports)
	}

	#[cfg(feature = "arcdps")]
	pub unsafe fn from_imp_ref(exports: &ImpExtensionExports) -> &Self {
		transmute(exports)
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ExtensionHeader<'s> {
	loaded: ExtensionHeaderLoaded,
	failed: ExtensionHeaderFailed<'s>,
}

impl<'s> ExtensionHeader<'s> {
	pub const EMPTY: Self = unsafe {
		Self::new(None, 0, 0)
	};

	pub const unsafe fn new(sig: Option<Sig>, size: u64, imgui_version: u32) -> Self {
		Self {
			loaded: ExtensionHeaderLoaded {
				size,
				sig,
				imgui_version,
			},
		}
	}

	pub const fn new_loaded(sig: Sig, size: usize, imgui_version: u32) -> Self {
		unsafe {
			Self::new(Some(sig), size as u64, imgui_version)
		}
	}

	pub const fn new_failed(error: Option<CStrPtr<'s>>) -> Self {
		Self {
			failed: ExtensionHeaderFailed::new(error),
		}
	}

	pub fn set_failed(&mut self, error: Option<CStrPtr<'s>>) {
		self.failed.sig = ExtensionHeaderFailed::SIG_FAILED;
		self.failed.error = error;
	}

	#[inline]
	pub fn sig(&self) -> Option<&Sig> {
		unsafe {
			self.loaded.sig.as_ref()
		}
	}

	#[inline]
	pub fn size(&self) -> Option<NonZeroUsize> {
		unsafe {
			self.loaded.size()
		}
	}

	#[inline]
	pub fn imgui_version(&self) -> &u32 {
		unsafe {
			&self.loaded.imgui_version
		}
	}

	#[inline]
	pub fn load_status(&self) -> Result<&Sig, &ExtensionHeaderFailed<'s>> {
		self.sig()
			.ok_or_else(|| unsafe { &self.failed })
	}
}

impl fmt::Debug for ExtensionHeader<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.load_status(), f)
	}
}

impl hash::Hash for ExtensionHeader<'_> {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		hash::Hash::hash(unsafe { &self.loaded }, state)
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Hash)]
pub struct ExtensionHeaderLoaded {
	pub size: u64,
	pub sig: Signature,
	pub imgui_version: u32,
}

impl ExtensionHeaderLoaded {
	pub fn size(&self) -> Option<NonZeroUsize> {
		match self.sig {
			Some(..) => NonZeroUsize::new(self.size as usize),
			None => None,
		}
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Hash)]
pub struct ExtensionHeaderFailed<'s> {
	#[cfg(all(target_pointer_width = "32", target_endian = "big"))]
	pub _padding_be: u32,
	pub error: Option<CStrPtr<'s>>,
	#[cfg(target_pointer_width = "32")]
	pub _align_64: [u64; 0],
	pub sig: SigRepr,
	pub _imgui_version: u32,
}

impl<'s> ExtensionHeaderFailed<'s> {
	pub const SIG_FAILED: SigRepr = 0;

	pub const fn new(error: Option<CStrPtr<'s>>) -> Self {
		Self {
			error,
			sig: Self::SIG_FAILED,
			_imgui_version: 0,
			#[cfg(all(target_endian = "big", not(target_pointer_width = "64")))]
			_padding_be: 0,
			#[cfg(not(target_pointer_width = "64"))]
			_align_64: [0; 0],
		}
	}
}
