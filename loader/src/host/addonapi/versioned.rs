use std::fmt;
use nexus::{AddonApi, DataLinkApi, EventApi, FontApi, InputBindsApi, LocalizationApi, MinHookApi, PathApi, QuickAccessApi, RendererApi, TextureApi, UiApi, WndProcApi};
use crate::util::nexus::AddonApiVersion;

pub use nexus::imgui;

pub enum AddonApiV {
	V2(nexus::v2::AddonApi),
	V3(nexus::v3::AddonApi),
	V4(nexus::v4::AddonApi),
	V6(nexus::v6::AddonApi),
}

impl AddonApiV {
	pub fn version(&self) -> AddonApiVersion {
		match self {
			Self::V2(..) => nexus::v2::AddonApi::VERSION,
			Self::V3(..) => nexus::v3::AddonApi::VERSION,
			Self::V4(..) => nexus::v4::AddonApi::VERSION,
			Self::V6(..) => nexus::v6::AddonApi::VERSION,
		}
	}

	pub fn as_ptr(&self) -> *const AddonApi {
		match self {
			Self::V2(api) => api as *const _ as *const _,
			Self::V3(api) => api as *const _ as *const _,
			Self::V4(api) => api as *const _ as *const _,
			Self::V6(api) => api as *const _,
		}
	}

	pub fn supports_api(api_version: AddonApiVersion) -> bool {
		matches!(
			api_version,
			nexus::v6::AddonApi::VERSION
			| nexus::v4::AddonApi::VERSION
			| nexus::v3::AddonApi::VERSION
			| nexus::v2::AddonApi::VERSION
		)
	}

	pub const fn v2_from_v3(api: nexus::v3::AddonApi) -> nexus::v2::AddonApi {
		AddonApiBackCompat2 {
			api: &api,
		}.to_v2()
	}
}

impl fmt::Display for AddonApiV {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let version = self.version();
		write!(f, "AddonApiV{version}")
	}
}

impl fmt::Debug for AddonApiV {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("AddonApiV")
			.field("version", &self.version())
			.finish()
	}
}

impl From<nexus::v6::AddonApi> for AddonApiV {
	#[inline]
	fn from(api: AddonApi) -> Self {
		Self::V6(api)
	}
}

impl From<nexus::v4::AddonApi> for AddonApiV {
	#[inline]
	fn from(api: nexus::v4::AddonApi) -> Self {
		Self::V4(api)
	}
}

impl From<nexus::v3::AddonApi> for AddonApiV {
	#[inline]
	fn from(api: nexus::v3::AddonApi) -> Self {
		Self::V3(api)
	}
}

impl From<nexus::v2::AddonApi> for AddonApiV {
	#[inline]
	fn from(api: nexus::v2::AddonApi) -> Self {
		Self::V2(api)
	}
}

pub struct AddonApiBackCompat4<'a> {
	pub api: &'a nexus::v6::AddonApi,
	pub add_simple_shortcut: nexus::quick_access::RawQuickAccessAddContextMenu,
}

impl AddonApiBackCompat4<'_> {
	pub const fn to_v4(self) -> nexus::v4::AddonApi {
		let Self {
			add_simple_shortcut,
			api: &AddonApi {
				log,
				request_update,
				swap_chain,
				imgui_context,
				imgui_malloc,
				imgui_free,
				renderer: RendererApi {
					register: register_render,
					deregister: deregister_render
				},
				path: PathApi { get_game_dir, get_addon_dir, get_common_dir },
				min_hook: MinHookApi {
					create: hook_create,
					remove: hook_remove,
					enable: hook_enable,
					disable: hook_disable,
				},
				event: EventApi {
					subscribe: event_subscribe,
					unsubscribe: event_unsubscribe,
					raise: event_raise,
					raise_notification: event_raise_notification,
					raise_targeted: event_raise_targeted,
					raise_notification_targeted: event_raise_notification_targeted,
				},
				wnd_proc: WndProcApi {
					register: register_wnd_proc,
					deregister: deregister_wnd_proc,
					send_to_game_only: send_wnd_proc_to_game_only,
				},
				input_binds: InputBindsApi {
					register_with_string: keybind_register_with_string,
					register_with_struct: keybind_register_with_struct,
					deregister: keybind_deregister,
					..
				},
				texture: TextureApi {
					get: get_texture,
					get_or_create_from_url: get_texture_or_create_from_url,
					get_or_create_from_file: get_texture_or_create_from_file,
					get_or_create_from_memory: get_texture_or_create_from_memory,
					get_or_create_from_resource: get_texture_or_create_from_resource,
					load_from_url: load_texture_from_url,
					load_from_file: load_texture_from_file,
					load_from_memory: load_texture_from_memory,
					load_from_resource: load_texture_from_resource,
				},
				font: FontApi {
					get: get_font,
					release: release_font,
					add_from_file: add_font_from_file,
					add_from_memory: add_font_from_memory,
					add_from_resource: add_font_from_resource,
					..
				},
				quick_access: QuickAccessApi {
					add: add_shortcut,
					remove: remove_shortcut,
					notify: notify_shortcut,
					remove_context_menu: remove_simple_shortcut,
					..
				},
				ui: UiApi {
					send_alert: alert_notify,
					..
				},
				data_link: DataLinkApi {
					get: get_resource,
					share: share_resource,
					..
				},
				localization: LocalizationApi { translate, translate_to, .. },
				..
			},
		} = self;
		nexus::v4::AddonApi {
			log, request_update,
			swap_chain,
			imgui_context, imgui_malloc, imgui_free,
			register_render, deregister_render,
			get_game_dir, get_addon_dir, get_common_dir,
			hook_create, hook_remove, hook_enable, hook_disable,
			event_subscribe, event_unsubscribe, event_raise, event_raise_notification,
			event_raise_targeted, event_raise_notification_targeted,
			register_wnd_proc, deregister_wnd_proc, send_wnd_proc_to_game_only,
			keybind_register_with_string, keybind_register_with_struct,
			keybind_deregister,
			get_resource, share_resource,
			get_texture, get_texture_or_create_from_url, get_texture_or_create_from_file, get_texture_or_create_from_memory, get_texture_or_create_from_resource,
			load_texture_from_url, load_texture_from_file, load_texture_from_memory, load_texture_from_resource,
			get_font, release_font,
			add_font_from_file, add_font_from_memory, add_font_from_resource,
			add_shortcut, remove_shortcut, notify_shortcut,
			add_simple_shortcut, remove_simple_shortcut,
			alert_notify,
			translate, translate_to,
		}
	}
}

pub struct AddonApiBackCompat3<'a> {
	pub api: &'a nexus::v4::AddonApi,
	pub keybind_register_with_string: nexus::keybind::RawKeybindRegisterWithStringOld,
	pub keybind_register_with_struct: nexus::keybind::RawKeybindRegisterWithStructOld,
}

impl AddonApiBackCompat3<'_> {
	pub const fn to_v3(self) -> nexus::v3::AddonApi {
		let Self {
			keybind_register_with_string, keybind_register_with_struct,
			api: &nexus::v4::AddonApi {
				log,
				swap_chain,
				imgui_context, imgui_malloc, imgui_free,
				register_render, deregister_render,
				get_game_dir, get_addon_dir, get_common_dir,
				hook_create, hook_remove, hook_enable, hook_disable,
				event_subscribe, event_unsubscribe, event_raise, event_raise_notification,
				event_raise_targeted, event_raise_notification_targeted,
				register_wnd_proc, deregister_wnd_proc, send_wnd_proc_to_game_only,
				keybind_deregister,
				get_resource, share_resource,
				get_texture, get_texture_or_create_from_url, get_texture_or_create_from_file, get_texture_or_create_from_memory, get_texture_or_create_from_resource,
				load_texture_from_url, load_texture_from_file, load_texture_from_memory, load_texture_from_resource,
				add_shortcut, remove_shortcut, notify_shortcut,
				add_simple_shortcut, remove_simple_shortcut,
				alert_notify,
				translate, translate_to,
				..
			},
		} = self;
		nexus::v3::AddonApi {
			log,
			swap_chain,
			imgui_context, imgui_malloc, imgui_free,
			register_render, deregister_render,
			get_game_dir, get_addon_dir, get_common_dir,
			hook_create, hook_remove, hook_enable, hook_disable,
			event_subscribe, event_unsubscribe, event_raise, event_raise_notification,
			event_raise_targeted, event_raise_notification_targeted,
			register_wnd_proc, deregister_wnd_proc, send_wnd_proc_to_game_only,
			keybind_register_with_string, keybind_register_with_struct,
			keybind_deregister,
			get_resource, share_resource,
			get_texture, get_texture_or_create_from_url, get_texture_or_create_from_file, get_texture_or_create_from_memory, get_texture_or_create_from_resource,
			load_texture_from_url, load_texture_from_file, load_texture_from_memory, load_texture_from_resource,
			add_shortcut, remove_shortcut, notify_shortcut,
			add_simple_shortcut, remove_simple_shortcut,
			alert_notify,
			translate, translate_to,
		}
	}
}

pub struct AddonApiBackCompat2<'a> {
	pub api: &'a nexus::v3::AddonApi,
}

impl AddonApiBackCompat2<'_> {
	pub const fn to_v2(self) -> nexus::v2::AddonApi {
		let Self {
			api: &nexus::v3::AddonApi {
				log,
				swap_chain,
				imgui_context, imgui_malloc, imgui_free,
				register_render, deregister_render,
				get_game_dir, get_addon_dir, get_common_dir,
				hook_create, hook_remove, hook_enable, hook_disable,
				event_subscribe, event_unsubscribe, event_raise, event_raise_notification,
				register_wnd_proc, deregister_wnd_proc, send_wnd_proc_to_game_only,
				keybind_register_with_string, keybind_register_with_struct, keybind_deregister,
				get_resource, share_resource,
				get_texture, get_texture_or_create_from_url, get_texture_or_create_from_file, get_texture_or_create_from_memory, get_texture_or_create_from_resource,
				load_texture_from_url, load_texture_from_file, load_texture_from_memory, load_texture_from_resource,
				add_shortcut, remove_shortcut, notify_shortcut,
				add_simple_shortcut, remove_simple_shortcut,
				translate, translate_to,
				..
			},
		} = self;

		nexus::v2::AddonApi {
			log,
			swap_chain,
			imgui_context, imgui_malloc, imgui_free,
			register_render, deregister_render,
			get_game_dir, get_addon_dir, get_common_dir,
			hook_create, hook_remove, hook_enable, hook_disable,
			event_subscribe, event_unsubscribe, event_raise, event_raise_notification,
			register_wnd_proc, deregister_wnd_proc, send_wnd_proc_to_game_only,
			keybind_register_with_string, keybind_register_with_struct, keybind_deregister,
			get_resource, share_resource,
			get_texture, get_texture_or_create_from_url, get_texture_or_create_from_file, get_texture_or_create_from_memory, get_texture_or_create_from_resource,
			load_texture_from_url, load_texture_from_file, load_texture_from_memory, load_texture_from_resource,
			add_shortcut, remove_shortcut, notify_shortcut,
			add_simple_shortcut, remove_simple_shortcut,
			translate, translate_to,
		}
	}
}
