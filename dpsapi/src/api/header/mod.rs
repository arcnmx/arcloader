use core::{ffi::c_void, fmt, hash, mem::{size_of, transmute}, num::{NonZeroUsize, NonZeroU64}, ptr::{self, NonNull}};
use arcffi::{
	cstr::{cstr, CStr, CStrPtr},
	windows::Win32::Foundation::HMODULE,
};
use crate::{
	api::import::ModuleExports,
	sig::{Signature, Sig, SigRepr},
	combat::{CombatEventData, CombatAgent},
};

pub use arcffi::{
	c_bool32,
	windows::Win32::Foundation::{HWND, LPARAM, WPARAM},
	UserMallocFn, UserFreeFn,
};
pub use crate::combat::CombatArgs;

pub type GetInitFn = unsafe extern "system" fn(
	arc_version: Option<CStrPtr<'static>>,
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

#[derive(Debug, Clone)]
pub struct InitArgs {
	pub arc_version: Option<CStrPtr<'static>>,
	pub imgui_ctx: Option<NonNull<c_void>>,
	pub id3d: Option<NonNull<c_void>>,
	pub module: ModuleExports,
	pub malloc: Option<UserMallocFn>,
	pub free: Option<UserFreeFn>,
	pub d3d_version: u32,
}

impl InitArgs {
	pub const EMPTY: Self = InitArgs {
		arc_version: None,
		imgui_ctx: None,
		id3d: None,
		module: ModuleExports::INVALID,
		malloc: None,
		free: None,
		d3d_version: 0,
	};

	pub const ALLOC_USER_DATA: *mut c_void = ptr::null_mut();
}

unsafe impl Send for InitArgs {}
unsafe impl Sync for InitArgs {}

pub type ExtensionFnCombat = for<'c, 's> unsafe extern "C" fn(
	ev: Option<&'c CombatEventData>,
	src: Option<&'c CombatAgent>,
	dst: Option<&'c CombatAgent>,
	skill_name: Option<CStrPtr<'s>>,
	id: Option<NonZeroU64>,
	revision: u64,
);
pub type ExtensionFnWnd = unsafe extern "C" fn(wnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> u32;
pub type ExtensionFnUiImgui = unsafe extern "C" fn(not_charsel_or_loading: c_bool32, hide_if_combat_or_ooc: c_bool32);
pub type ExtensionFnUiOptionsTab = unsafe extern "C" fn();
pub type ExtensionFnUiOptionsWindows = for<'w> unsafe extern "C" fn(window_name: Option<CStrPtr<'w>>) -> c_bool32;

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

	pub unsafe fn lookup_init<M: Into<HMODULE>>(module: M) -> Option<GetInitFn> {
		ModuleExports::lookup_export(module.into(), ExtensionExports::SYM_GET_INIT)
			.map(|f| unsafe { transmute(f) })
	}

	pub unsafe fn lookup_release<M: Into<HMODULE>>(module: M) -> Option<GetReleaseFn> {
		ModuleExports::lookup_export(module.into(), ExtensionExports::SYM_GET_RELEASE)
			.map(|f| unsafe { transmute(f) })
	}

	pub unsafe fn get_release<M: Into<HMODULE>>(module: M) -> Result<Option<ReleaseFn>, ()> {
		Self::lookup_release(module)
			.map(|f| f())
			.ok_or(())
	}

	pub unsafe fn get_init<M: Into<HMODULE>>(module: M, args: InitArgs) -> Result<Option<InitFn>, ()> {
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
		arc_version: Option<CStrPtr<'static>>,
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
	pub const IMGUI_VERSION_20210202: u32 = 18000;

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

#[macro_export]
macro_rules! wrap_init_addr {
	(#[naked] unsafe extern fn get_init_addr() => $unwrapped:path;) => {
		#[naked]
		#[no_mangle]
		pub unsafe extern "system" fn get_init_addr() -> ! {
			const GET_INIT_ADDR: GetInitFn = ExtensionExports::wrap_init_fn_item(&$unwrapped);
			core::arch::asm! {
				"jmp {get_init}",
				get_init = in(GET_INIT_ADDR) _,
				options(noreturn),
			}
		}
	};
	(#[global_asm] unsafe extern fn get_init_addr() => $unwrapped:path;) => {
		#[link_section = ".text"]
		static __DPSAPI_GET_INIT_ADDR_PTR: GetInitFn = $crate::api::header::ExtensionExports::wrap_init_fn_item(&$unwrapped);
		#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
		core::arch::global_asm! {
			".global get_init_addr",
			"get_init_addr:",
			"jmp [rip+{get_init_addr}]",
			/*"mov [rip+get_init_addr_indirect], eax",
			"jmp eax",
			".balign 8",
			"get_init_addr_indirect:",
			".8byte {get_init_addr}",*/
			/*".def",
			//".type get_init_addr, @function",
			".type get_init_addr, int",
			".endef",*/
			get_init_addr = sym __DPSAPI_GET_INIT_ADDR_PTR,
		}
	};
	(unsafe extern fn get_init_addr() => $unwrapped:path;) => {
		#[no_mangle]
		pub unsafe extern "system" fn get_init_addr(
			arc_version: Option<$crate::_extern::arcffi::cstr::CStrPtr<'static>>,
			imgui_ctx: Option<::core::ptr::NonNull<::core::ffi::c_void>>,
			id3d: Option<::core::ptr::NonNull<::core::ffi::c_void>>,
			module: $crate::api::import::ModuleExports,
			malloc: Option<$crate::_extern::arcffi::UserMallocFn>,
			free: Option<$crate::_extern::arcffi::UserFreeFn>,
			d3d_version: u32,
		) -> Option<$crate::api::header::InitFn> {
			$unwrapped($crate::api::header::InitArgs {
				arc_version,
				imgui_ctx,
				id3d,
				module,
				malloc,
				free,
				d3d_version,
			}).into()
		}
	};
}
pub use wrap_init_addr;
