use nexus::gamebind::GameBind;
use crate::host::addonapi::NexusHost;

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_game_bind_press_async(bind: GameBind) {
		addonapi_stub!(game_bind::press_async("{:?}", bind) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_game_bind_release_async(bind: GameBind) {
		addonapi_stub!(game_bind::release_async("{:?}", bind) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_game_bind_invoke_async(bind: GameBind, duration: i32) {
		addonapi_stub!(game_bind::invoke_async("{:?}, {:?}", bind, duration) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_game_bind_press(bind: GameBind) {
		addonapi_stub!(game_bind::press("{:?}", bind) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_game_bind_release(bind: GameBind) {
		addonapi_stub!(game_bind::release("{:?}", bind) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_game_bind_is_bound(bind: GameBind) -> bool {
		addonapi_stub!(game_bind::is_bound("{:?}", bind) => false)
	}
}
