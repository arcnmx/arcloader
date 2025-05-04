use std::{collections::BTreeSet, ptr, sync::{atomic::{AtomicUsize, Ordering}, RwLock}};

use nexus::wnd_proc::RawWndProcCallback;
use windows::Win32::{Foundation::{HWND, LPARAM, LRESULT, WPARAM}, UI::WindowsAndMessaging::{self as wnd, PostMessageA}};

use crate::host::addonapi::NexusHost;

pub static WNDPROC_CALLBACKS: RwLock<BTreeSet<WndRegistration>> = RwLock::new(BTreeSet::new());
pub static WNDPROC_WINDOW: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct WndRegistration {
	callback: RawWndProcCallback,
}

unsafe impl Send for WndRegistration {}
unsafe impl Sync for WndRegistration {}

impl NexusHost {
	pub const WM_PASSTHROUGH_FIRST: u32 = wnd::WM_USER + 7997;

	pub fn wndproc_window() -> HWND {
		HWND(WNDPROC_WINDOW.load(Ordering::Relaxed) as *mut _)
	}

	pub fn wndproc_call(window: HWND, message: u32, param_w: WPARAM, param_l: LPARAM) -> u32 {
		let callbacks = match WNDPROC_CALLBACKS.read() {
			Ok(cb) => cb,
			Err(..) => return message,
		};

		for cb in callbacks.iter() {
			match (cb.callback)(window, message, param_w, param_l) {
				0 => return 0,
				m if m != 1 && m != message => {
					warn!("wndproc callback wanted to replace {message} with {m}?");
				},
				_ => (),
			}
		}

		message
	}

	pub fn wndproc_filter(window: HWND, message: u32, param_w: WPARAM, param_l: LPARAM) -> u32 {
		message
	}

	pub fn wndproc_nofilter(window: HWND, mut message: u32, param_w: WPARAM, param_l: LPARAM) -> u32 {
		WNDPROC_WINDOW.store(window.0 as usize, Ordering::Relaxed);

		match message {
			wnd::WM_SIZE => {
				// XXX: nexus triggers this from hooking DXGIResizeBuffers instead
				Self::event_broadcast(Self::EV_WINDOW_RESIZED, ptr::null());
			},
			_ => (),
		}

		message = Self::wndproc_call(window, message, param_w, param_l);

		message
	}

	pub unsafe extern "C-unwind" fn addonapi_wndproc_register(wnd_proc_callback: RawWndProcCallback) {
		addonapi_stub!(wndproc::register("{:?}", wnd_proc_callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_wndproc_deregister(wnd_proc_callback: RawWndProcCallback) {
		addonapi_stub!(wndproc::deregister("{:?}", wnd_proc_callback) => ())
	}

	pub unsafe extern "C-unwind" fn addonapi_wndproc_send_to_game(h_wnd: HWND, message: u32, param_w: WPARAM, param_l: LPARAM) -> LRESULT {
		addonapi_stub!(wndproc::send_to_game("{:?}, {:?}, {:?}, {:?}", h_wnd, message, param_w, param_l));

		let window_handle = match Self::wndproc_window() {
			ours if h_wnd.is_invalid() || ours == h_wnd => ours,
			_ours => {
				warn!("wndproc_send supposed to use {h_wnd:?} instead of {_ours:?}");
				h_wnd
			},
		};

		let res = match message {
			m if m < wnd::WM_USER => unsafe {
				PostMessageA(Some(window_handle), message + Self::WM_PASSTHROUGH_FIRST, param_w, param_l)
			},
			_ => unsafe {
				PostMessageA(Some(window_handle), message, param_w, param_l)
			},
		};

		match res {
			Ok(()) => LRESULT(0),
			Err(e) => LRESULT(e.code().0 as isize),
		}
	}
}
