pub mod WindowsAndMessaging {
	use crate::windows::Win32::Foundation::{HWND, WPARAM, LPARAM, LRESULT};

	pub type Wndproc = unsafe extern "system" fn(wnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT;
	pub type WNDPROC = Option<Wndproc>;
}
