use std::{ffi::CStr, num::NonZeroU32, sync::atomic::{AtomicBool, Ordering}};
use crate::{extensions::Loader, supervisor::Supervisor, ui::Options, RenderThread};
#[cfg(feature = "host-addonapi")]
use crate::host::addonapi::NexusHost;
use ::arcdps::evtc::{Agent, Event};
#[cfg(feature = "arcdps-extras")]
use arcdps::extras::{ExtrasAddonInfo, UserInfoIter};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows_strings::PCSTR;

#[cfg(feature = "arcdps")]
pub mod arcdps;

#[cfg(feature = "arcdps")]
pub use self::arcdps::{imgui_ctx, imgui_ui, allocator_fns};

pub const SIG: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(
	u32::from_le_bytes([b'm', b'e', b'w', 3])
) };
pub const SIG_DEBUG: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(
	SIG.get() | 0x40000000
)};

pub static ARC_LOADED: AtomicBool = AtomicBool::new(false);

pub unsafe fn init() -> Result<(), &'static CStr> {
	debug!("arcloader *waves*");

	if ARC_LOADED.load(Ordering::Relaxed) {
		return Err(cstr!("extension loaded twice"))
	}

	#[cfg(all(debug_assertions, panic = "unwind"))] {
		use std::env;
		env::set_var("RUST_LIB_BACKTRACE", "1");
	}

	Supervisor::init();
	Loader::init();
	Options::init();
	#[cfg(feature = "host-addonapi")] {
		NexusHost::init();
	}

	ARC_LOADED.store(true, Ordering::Relaxed);

	Ok(())
}

pub fn release() {
	debug!("arcloader *hides*");

	#[cfg(feature = "host-addonapi")] {
		NexusHost::unload();
	}
	Options::unload();
	Loader::unload();
	Supervisor::unload();

	ARC_LOADED.store(false, Ordering::Relaxed);
}

pub fn update_url() -> Option<String> {
	None
}

pub fn evtc(event: Option<&Event>, src: Option<&Agent>, dst: Option<&Agent>, skill_name: PCSTR, id: u64, revision: u64, is_local: bool) {
	#[cfg(feature = "host-addonapi")] {
		NexusHost::evtc(event, src, dst, skill_name, id, revision, is_local);
	}
}

pub fn wnd_nofilter(window: HWND, message: u32, param_w: WPARAM, param_l: LPARAM) -> u32 {
	match ARC_LOADED.load(Ordering::Relaxed) {
		#[cfg(feature = "host-addonapi")]
		true => NexusHost::wndproc_nofilter(window, message, param_w, param_l),
		_ => message,
	}
}

pub fn wnd_filter(window: HWND, message: u32, param_w: WPARAM, param_l: LPARAM) -> u32 {
	match ARC_LOADED.load(Ordering::Relaxed) {
		#[cfg(feature = "host-addonapi")]
		true => NexusHost::wndproc_filter(window, message, param_w, param_l),
		_ => message,
	}
}

pub fn imgui(not_charsel_or_loading: bool) {
	if !ARC_LOADED.load(Ordering::Relaxed) {
		return
	}

	RenderThread::render_start();

	#[cfg(feature = "host-addonapi")] {
		NexusHost::imgui_present(not_charsel_or_loading);
	}
	Supervisor::imgui_present();
	Options::imgui_present();
}

pub fn options_end() {
	if !ARC_LOADED.load(Ordering::Relaxed) {
		return
	}

	Options::imgui_options_end();
}

pub fn options_windows(window_name: Option<&str>) -> bool {
	true
}

#[cfg(feature = "arcdps-extras")]
pub fn extras_init(extras: ExtrasAddonInfo, idk: Option<&str>) {
}

#[cfg(feature = "arcdps-extras")]
pub fn extras_squad_update(users: UserInfoIter) {
}
