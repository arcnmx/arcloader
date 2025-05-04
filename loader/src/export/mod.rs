use std::{ffi::CStr, num::NonZeroU32, sync::atomic::{AtomicBool, Ordering}};
use crate::{supervisor::Supervisor, extensions::Loader, ui::Options};
#[cfg(feature = "host-addonapi")]
use crate::host::addonapi::NexusHost;
use ::arcdps::{
	evtc::{Agent, Event}, imgui::Ui,
};
#[cfg(feature = "arcdps-extras")]
use arcdps::extras::{ExtrasAddonInfo, UserInfoIter};

#[cfg(feature = "arcdps")]
pub mod arcdps;

#[cfg(feature = "arcdps")]
pub use self::arcdps::{imgui_ctx, allocator_fns};

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

pub fn evtc(event: Option<&Event>, src: Option<&Agent>, dst: Option<&Agent>, skill_name: Option<&str>, id: u64, revision: u64) {
}

pub fn wnd_nofilter(vkc: usize, pressed: bool, repeat: bool) -> bool {
	true
}

pub fn wnd_filter(vkc: usize, pressed: bool, repeat: bool) -> bool {
	true
}

pub fn imgui(ui: &Ui, not_charsel_or_loading: bool) {
	if !ARC_LOADED.load(Ordering::Relaxed) {
		return
	}

	#[cfg(feature = "host-addonapi")] {
		NexusHost::imgui_present(ui, not_charsel_or_loading);
	}
	Supervisor::imgui_present();
	Options::imgui_present(ui);
}

pub fn options_end(ui: &Ui) {
	if !ARC_LOADED.load(Ordering::Relaxed) {
		return
	}

	Options::imgui_options_end(ui);
}

pub fn options_windows(ui: &Ui, window_name: Option<&str>) -> bool {
	true
}

#[cfg(feature = "arcdps-extras")]
pub fn extras_init(extras: ExtrasAddonInfo, idk: Option<&str>) {
}

#[cfg(feature = "arcdps-extras")]
pub fn extras_squad_update(users: UserInfoIter) {
}
