use crate::export::{self, arcdps::SIG};

macro_rules! export_arcloader {
	(!$ex:tt arcdps {
		$($arcdps_export:tt)*
	}) => {
		arcdps::export $ex {
			$($arcdps_export)*

			sig: SIG.get(),

			init: init_macro,
			release: export::release,
			//update_url: exports::update_url,

			imgui: export::imgui,
			/*options_windows: exports::options_windows,*/
			options_end: export::options_end,

			/*wnd_nofilter: exports::wnd_nofilter,
			wnd_filter: exports::wnd_filter,*/

			/*combat: exports::evtc,
			combat_local: exports::combat_local,*/

			/*#[cfg(feature = "extras")]
			extras_init: exports::extras_init,
			#[cfg(feature = "extras")]
			extras_squad_update: exports::extras_squad_update,*/
			/*
			extras_language_changed: exports::extras_language_changed,
			extras_keybind_changed: exports::extras_keybind_changed,
			extras_squad_chat_message: exports::extras_squad_chat_message,
			extras_chat_message: exports::extras_chat_message,*/
		}
	};
}

#[cfg(debug_assertions)]
export_arcloader! {
	!!arcdps {
		name: "arcloader+debug",
	}
}

#[cfg(not(debug_assertions))]
export_arcloader! {
	!!arcdps {
		name: "arcloader",
	}
}

fn init_macro() -> Result<(), String> {
	export::init()
		.map_err(|msg| msg.to_string_lossy().into_owned())
}
