use std::ptr;
use ::arcdps::dxgi_swap_chain;
use nexus::{DataLinkApi, EventApi, FontApi, GameBindApi, InputBindsApi, LocalizationApi, MinHookApi, PathApi, QuickAccessApi, RendererApi, TextureApi, UiApi, WndProcApi};
use windows::{core::Error as WinError, Win32::Foundation::ERROR_API_UNAVAILABLE};

pub mod host;
pub mod addon;
pub mod versioned;
pub use self::{
	versioned::AddonApiV,
	host::{NexusHost, NEXUS_HOST},
	addon::{NexusAddon, NexusAddonCache},
	data_link::MumbleIdentity,
	texture::TextureCache,
};

pub use nexus::imgui;

use crate::util::win::WinResult;

impl NexusHost {
	pub const API_V6: nexus::v6::AddonApi = nexus::v6::AddonApi {
		swap_chain: None,
		imgui_context: ptr::null_mut(),
		imgui_malloc: None,
		imgui_free: None,
		request_update: Self::addonapi_request_update,
		ui: UiApi {
			send_alert: Self::addonapi_ui_send_alert,
			register_close_on_escape: Self::addonapi_ui_register_close_on_escape,
			deregister_close_on_escape: Self::addonapi_ui_deregister_close_on_escape,
		},
		renderer: RendererApi {
			register: Self::addonapi_renderer_register,
			deregister: Self::addonapi_renderer_deregister,
		},
		input_binds: InputBindsApi {
			register_with_string: Self::addonapi_input_binds_register_with_string,
			register_with_struct: Self::addonapi_input_binds_register_with_struct,
			deregister: Self::addonapi_input_binds_deregister,
			invoke: Self::addonapi_input_binds_invoke,
		},
		log: Self::addonapi_log,
		quick_access: QuickAccessApi {
			add: Self::addonapi_quick_access_add,
			remove: Self::addonapi_quick_access_remove,
			notify: Self::addonapi_quick_access_notify,
			add_context_menu: Self::addonapi_quick_access_add_context_menu,
			remove_context_menu: Self::addonapi_quick_access_remove_context_menu,
		},
		path: PathApi {
			get_game_dir: Self::addonapi_path_get_game_dir,
			get_addon_dir: Self::addonapi_path_get_addon_dir,
			get_common_dir: Self::addonapi_path_get_common_dir,
		},
		min_hook: MinHookApi {
			create: Self::addonapi_min_hook_create,
			remove: Self::addonapi_min_hook_remove,
			enable: Self::addonapi_min_hook_enable,
			disable: Self::addonapi_min_hook_disable,
		},
		event: EventApi {
			subscribe: Self::addonapi_event_subscribe,
			unsubscribe: Self::addonapi_event_unsubscribe,
			raise: Self::addonapi_event_raise,
			raise_targeted: Self::addonapi_event_raise_targeted,
			raise_notification: Self::addonapi_event_raise_notification,
			raise_notification_targeted: Self::addonapi_event_raise_notification_targeted,
		},
		wnd_proc: WndProcApi {
			register: Self::addonapi_wndproc_register,
			deregister: Self::addonapi_wndproc_deregister,
			send_to_game_only: Self::addonapi_wndproc_send_to_game,
		},
		game_bind: GameBindApi {
			press_async: Self::addonapi_game_bind_press_async,
			release_async: Self::addonapi_game_bind_release_async,
			invoke_async: Self::addonapi_game_bind_invoke_async,
			press: Self::addonapi_game_bind_press,
			release: Self::addonapi_game_bind_release,
			is_bound: Self::addonapi_game_bind_is_bound,
		},
		data_link: DataLinkApi {
			get: Self::addonapi_data_link_get,
			share: Self::addonapi_data_link_share,
		},
		texture: TextureApi {
			load_from_resource: Self::addonapi_texture_load_from_resource,
			load_from_file: Self::addonapi_texture_load_from_file,
			load_from_url: Self::addonapi_texture_load_from_url,
			load_from_memory: Self::addonapi_texture_load_from_memory,
			get_or_create_from_url: Self::addonapi_texture_get_or_create_from_url,
			get_or_create_from_file: Self::addonapi_texture_get_or_create_from_file,
			get_or_create_from_memory: Self::addonapi_texture_get_or_create_from_memory,
			get_or_create_from_resource: Self::addonapi_texture_get_or_create_from_resource,
			get: Self::addonapi_texture_get,
		},
		localization: LocalizationApi {
			translate: Self::addonapi_localization_translate,
			translate_to: Self::addonapi_localization_translate_to,
			set: Self::addonapi_localization_set,
		},
		font: FontApi {
			add_from_file: Self::addonapi_font_add_from_file,
			add_from_resource: Self::addonapi_font_add_from_resource,
			add_from_memory: Self::addonapi_font_add_from_memory,
			release: Self::addonapi_font_release,
			resize: Self::addonapi_font_resize,
			get: Self::addonapi_font_get,
		},
	};
	pub const API_V4: nexus::v4::AddonApi = versioned::AddonApiBackCompat4 {
		api: &Self::API_V6,
		add_simple_shortcut: Self::addonapi_quick_access_add_context_menu_v2,
	}.to_v4();
	pub const API_V3: nexus::v3::AddonApi = versioned::AddonApiBackCompat3 {
		api: &Self::API_V4,
		keybind_register_with_string: Self::addonapi_input_binds_register_with_string_v2,
		keybind_register_with_struct: Self::addonapi_input_binds_register_with_struct_v2,
	}.to_v3();
	pub const API_V2: nexus::v2::AddonApi = versioned::AddonApiBackCompat2 {
		api: &Self::API_V3,
	}.to_v2();

	pub fn api_with_version(&self, api_version: i32) -> WinResult<AddonApiV> {
		if !AddonApiV::supports_api(api_version) {
			return Err(WinError::new(ERROR_API_UNAVAILABLE.to_hresult(), format!("AddonAPI v{} not supported", api_version)))
		}

		let imgui_context = crate::export::imgui_ctx();
		let swap_chain = dxgi_swap_chain();
		let (imgui_malloc, imgui_free, _imgui_allocator_data) = crate::export::allocator_fns();

		Ok(match api_version {
			nexus::v2::AddonApi::VERSION => nexus::v2::AddonApi {
				swap_chain, imgui_context, imgui_malloc, imgui_free,
				.. Self::API_V2
			}.into(),
			nexus::v3::AddonApi::VERSION => nexus::v3::AddonApi {
				swap_chain, imgui_context, imgui_malloc, imgui_free,
				.. Self::API_V3
			}.into(),
			nexus::v4::AddonApi::VERSION => nexus::v4::AddonApi {
				swap_chain, imgui_context, imgui_malloc, imgui_free,
				.. Self::API_V4
			}.into(),
			_ => nexus::v6::AddonApi {
				swap_chain, imgui_context, imgui_malloc, imgui_free,
				.. Self::API_V6
			}.into(),
		})
	}
}

unsafe impl Send for NexusHost {}
unsafe impl Sync for NexusHost {}

macro_rules! addonapi_stub {
	($mod_:ident :: $f:ident $arg:tt => $res:expr) => {
		{
			addonapi_stub! { @log(warn, " unimplemented stub")
				$mod_ :: $f $arg
			}

			$res
		}
	};
	($mod_:ident :: $f:ident $arg:tt) => {
		{
			addonapi_stub! { @log(debug, "")
				$mod_ :: $f $arg
			}
		}
	};
	(@log($level:ident, $postfix:literal) $module:ident :: $f:ident ($($fmt:literal)? $(, $($farg:tt)*)?)) => {
		{
			#[cfg(feature = "log")]
			addonapi_stub! { @log::$level(
				concat!("AddonApi::", stringify!($module), ".", stringify!($f), "(", $($fmt,)? ")", $postfix)
				$(, $($farg)*)?
			) }
		}
	};
	(@log::$level:ident($($tt:tt)*)) => {
		log::$level! { $($tt)* }
	};
}

mod log;
mod path;
mod update;
mod event;
mod wndproc;
mod hook;
pub mod input;
mod binds;
pub mod data_link;
mod font;
mod texture;
mod localization;
mod quick_access;
mod ui;
mod render;
#[cfg(feature = "arcdps")]
pub mod arcdps;
