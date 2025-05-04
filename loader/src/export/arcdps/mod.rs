use std::{ffi::c_void, num::NonZeroU32, ptr};
use arcdps::{
	imgui::sys as imgui_sys,
	__macro::{MallocFn, FreeFn},
};
use crate::export;

#[cfg(feature = "arcdps-codegen")]
mod codegen;

pub const SIG: NonZeroU32 = match () {
	#[cfg(debug_assertions)]
	_ => export::SIG_DEBUG,
	#[cfg(not(debug_assertions))]
	_ => export::SIG,
};

pub fn imgui_ctx() -> *mut imgui_sys::ImGuiContext {
	match () {
		#[cfg(not(feature = "arcdps-codegen"))]
		() => unsafe {
			ptr::read(ptr::addr_of!(extern_::IMGUI_CTX))
		},
	}
}

pub fn allocator_fns() -> (Option<MallocFn>, Option<FreeFn>, *mut c_void) {
	match () {
		#[cfg(not(feature = "arcdps-codegen"))]
		() => unsafe {
			let user_data = ptr::null_mut();
			(
				ptr::read(ptr::addr_of!(extern_::MALLOC)),
				ptr::read(ptr::addr_of!(extern_::FREE)),
				user_data,
			)
		},
	}
}

#[cfg(not(feature = "arcdps-codegen"))]
pub mod extern_ {
	use std::{alloc::{GlobalAlloc, Layout}, ffi::{c_char, c_void, CStr}, ptr::{self, NonNull}};
	use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, WPARAM};
	use windows_strings::PCSTR;
	use arcdps::{
		imgui::sys as imgui_sys,
		callbacks::ArcDpsExport,
		__macro::{MallocFn, FreeFn, init as arcdps_rs_init, ui as imgui_ui},
		Event, Agent,
	};
	use crate::{export::{self, arcdps::SIG}, util::arc::{InitFn, ReleaseFn}};

	pub(crate) static mut IMGUI_CTX: *mut imgui_sys::ImGuiContext = ptr::null_mut();
	pub(crate) static mut MALLOC: Option<MallocFn> = None;
	pub(crate) static mut FREE: Option<FreeFn> = None;

	pub const NAME: &'static CStr = match () {
		#[cfg(debug_assertions)]
		() => cstr!(env!("CARGO_PKG_NAME"), "+debug"),
		#[cfg(not(debug_assertions))]
		() => cstr!(env!("CARGO_PKG_NAME")),
	};

	pub const BUILD: &'static CStr = cstr!(env!("CARGO_PKG_VERSION"));

	const IMGUI_VERSION_20210202: u32 = 18000;

	static mut ARCDPS_EXPORT: ArcDpsExport = ArcDpsExport {
		size: 0,
		sig: 0,
		imgui_version: IMGUI_VERSION_20210202,
		out_name: NAME.as_ptr() as *const _,
		out_build: BUILD.as_ptr() as *const _,
		combat: Some(combat),
		combat_local: Some(combat_local),
		imgui: Some(imgui),
		options_end: Some(options_end),
		options_windows: None,
		wnd_filter: Some(wnd_filter),
		wnd_nofilter: Some(wnd_nofilter),
	};

	#[no_mangle]
	pub unsafe extern "system" fn get_init_addr(
		arc_version: *const c_char,
		imgui_ctx: *mut imgui_sys::ImGuiContext,
		id3d: *mut c_void,
		arcdps: HMODULE,
		malloc: Option<MallocFn>,
		free: Option<FreeFn>,
		d3d_version: u32,
	) -> Option<InitFn> {
		ptr::write(ptr::addr_of_mut!(MALLOC), malloc);
		ptr::write(ptr::addr_of_mut!(FREE), free);
		ptr::write(ptr::addr_of_mut!(IMGUI_CTX), imgui_ctx);
		arcdps_rs_init(arc_version, arcdps, imgui_ctx, malloc, free, id3d, d3d_version, env!("CARGO_PKG_NAME"));
		Some(init)
	}

	unsafe extern "C" fn init() -> *const ArcDpsExport {
		match export::init() {
			Ok(()) => {
				ptr::write(ptr::addr_of_mut!(ARCDPS_EXPORT.size), size_of::<ArcDpsExport>());
				ptr::write(ptr::addr_of_mut!(ARCDPS_EXPORT.sig), SIG.get());
			},
			Err(message) => {
				ptr::write(ptr::addr_of_mut!(ARCDPS_EXPORT.size), message.as_ptr() as usize);
			},
		}
		ptr::addr_of!(ARCDPS_EXPORT)
	}

extern_fns! {
	unsafe extern "C" fn combat(
		cbtevent: *const Event,
		src: *const Agent,
		dst: *const Agent,
		skillname: *const c_char,
		id: u64,
		revision: u64
	) {
		let ev = NonNull::new(cbtevent as *mut Event);
		let src = NonNull::new(src as *mut Agent);
		let dst = NonNull::new(dst as *mut Agent);
		let skillname = PCSTR::from_raw(skillname as *const _);
		export::evtc(
			ev.map(|e| &*e.as_ptr()),
			src.map(|e| &*e.as_ptr()),
			dst.map(|e| &*e.as_ptr()),
			skillname,
			id,
			revision,
			false,
		)
	}

	unsafe extern "C" fn combat_local(
		cbtevent: *const Event,
		src: *const Agent,
		dst: *const Agent,
		skillname: *const c_char,
		id: u64,
		revision: u64
	) {
		let ev = NonNull::new(cbtevent as *mut Event);
		let src = NonNull::new(src as *mut Agent);
		let dst = NonNull::new(dst as *mut Agent);
		let skillname = PCSTR::from_raw(skillname as *const _);
		export::evtc(
			ev.map(|e| &*e.as_ptr()),
			src.map(|e| &*e.as_ptr()),
			dst.map(|e| &*e.as_ptr()),
			skillname,
			id,
			revision,
			true,
		)
	}

	unsafe extern "C" fn wnd_filter(window: HWND, message: u32, param_w: WPARAM, param_l: LPARAM) -> u32 {
		export::wnd_filter(window, message, param_w, param_l)
	}

	unsafe extern "C" fn wnd_nofilter(window: HWND, message: u32, param_w: WPARAM, param_l: LPARAM) -> u32 {
		export::wnd_nofilter(window, message, param_w, param_l)
	}

	unsafe extern "C" fn imgui(not_charsel_or_loading: u32) {
		export::imgui(&imgui_ui(), not_charsel_or_loading != 0)
	}

	unsafe extern "C" fn options_end() {
		export::options_end(&imgui_ui())
	}
}

	#[no_mangle]
	pub unsafe extern "system" fn get_release_addr() -> Option<ReleaseFn> {
		Some(release)
	}

	unsafe extern "C" fn release() {
		export::release()
	}

	#[cfg(feature = "arcdps-extras")]
	pub unsafe extern "system" fn arcdps_unofficial_extras_subscriber_init(
		addon: *const extras::RawExtrasAddonInfo,
		sub: *mut extras::ExtrasSubscriberInfo,
	) {
		compile_error!("TODO")
	}

	#[global_allocator]
	static GLOBAL: ArcDpsAllocator = ArcDpsAllocator;

	pub struct ArcDpsAllocator;

	unsafe impl GlobalAlloc for ArcDpsAllocator {
		unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
			match ptr::read(ptr::addr_of!(MALLOC)) {
				Some(malloc) => malloc(layout.size(), ptr::null_mut()) as *mut _,
				//None => std::alloc::System.alloc(layout),
				//None => std::hint::unreachable_unchecked(),
				None => ptr::null_mut(),
			}
		}
		unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
			match ptr::read(ptr::addr_of!(FREE)) {
				Some(free) => free(ptr as *mut _, ptr::null_mut()),
				//None => std::alloc::System.dealloc(_layout),
				//None => std::hint::unreachable_unchecked(),
				None => (),
			}
		}
	}
}
