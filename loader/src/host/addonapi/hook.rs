use nexus::hook::HookStatus;
use crate::host::addonapi::NexusHost;
use std::ffi::c_void;

impl NexusHost {
	pub unsafe extern "stdcall-unwind" fn addonapi_min_hook_create(target: *const c_void, detour: *const c_void, trampoline: *mut *const c_void) -> HookStatus {
		addonapi_stub!(min_hook::create("{:?}, {:?}, {:?}", target, detour, trampoline) => HookStatus::ErrorUnsupportedFunction)
	}

	pub unsafe extern "stdcall-unwind" fn addonapi_min_hook_remove(target: *const c_void) -> HookStatus {
		addonapi_stub!(min_hook::remove("{:?}", target) => HookStatus::ErrorUnsupportedFunction)
	}

	pub unsafe extern "stdcall-unwind" fn addonapi_min_hook_enable(target: *const c_void) -> HookStatus {
		addonapi_stub!(min_hook::enable("{:?}", target) => HookStatus::ErrorUnsupportedFunction)
	}

	pub unsafe extern "stdcall-unwind" fn addonapi_min_hook_disable(target: *const c_void) -> HookStatus {
		addonapi_stub!(min_hook::disable("{:?}", target) => HookStatus::ErrorUnsupportedFunction)
	}
}
