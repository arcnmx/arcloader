use nexus::wnd_proc::RawWndProcCallback;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

use crate::host::addonapi::NexusHost;

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_wndproc_register(wnd_proc_callback: RawWndProcCallback) {
		addonapi_stub!(wndproc::register("{:?}", wnd_proc_callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_wndproc_deregister(wnd_proc_callback: RawWndProcCallback) {
		addonapi_stub!(wndproc::deregister("{:?}", wnd_proc_callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_wndproc_send_to_game(h_wnd: HWND, u_msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
		let res = LRESULT::default();
		addonapi_stub!(wndproc::send_to_game("{:?}, {:?}, {:?}, {:?}", h_wnd, u_msg, w_param, l_param) => res)
	}
}
