use std::cell::RefCell;
use crate::extensions::Exts;

arcdps::export! {
	name: "arcloader",
	sig: exports::SIG,
	init: exports::init,
	release: exports::release,

	options_end: exports::options_end,

	/*
	update_url: exports::update_url,

	imgui: exports::imgui,
	options_windows: exports::options_windows,

	wnd_nofilter: exports::wnd_nofilter,
	wnd_filter: exports::wnd_filter,

	combat: exports::evtc,
	//combat_local: exports::combat_local,

	/*#[cfg(feature = "extras")]
	extras_init: exports::extras_init,
	#[cfg(feature = "extras")]
	extras_squad_update: exports::extras_squad_update,*/
	/*
	extras_language_changed: exports::extras_language_changed,
	extras_keybind_changed: exports::extras_keybind_changed,
	extras_squad_chat_message: exports::extras_squad_chat_message,
	extras_chat_message: exports::extras_chat_message,*/
	*/
}

pub mod extensions;

thread_local! {
	static EXTS: RefCell<Exts> = RefCell::new(Exts::new());
}

pub mod exports {
	use crate::EXTS;
	use arcdps::{
		evtc::{Agent, Event}, imgui::Ui,
	};
	#[cfg(feature = "extras")]
	use arcdps::extras::{ExtrasAddonInfo, UserInfoIter};
	#[cfg(feature = "log")]
	use log::debug;

	pub const SIG: u32 = u32::from_le_bytes([b'm', b'e', b'w', 3]);

	pub fn init() -> Result<(), String> {
		#[cfg(feature = "log")] {
			debug!("arcloader *waves*");
		}

		Ok(())
	}

	pub fn release() {
		#[cfg(feature = "log")] {
			debug!("arcloader *hides*");
		}
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
	}

	pub fn options_end(ui: &Ui) {
		EXTS.with_borrow_mut(|exts| {
			match exts.imgui_options_table(ui) {
				Ok(()) => (),
				Err(msg) => ui.text_disabled(msg),
			}
		});
	}

	pub fn options_windows(ui: &Ui, window_name: Option<&str>) -> bool {
		true
	}

	#[cfg(feature = "extras")]
	pub fn extras_init(extras: ExtrasAddonInfo, idk: Option<&str>) {
	}

	#[cfg(feature = "extras")]
	pub fn extras_squad_update(users: UserInfoIter) {
	}
}
