use std::{ffi::OsString, num::NonZeroU16, os::windows::ffi::OsStringExt};
use windows::Win32::{Foundation::LPARAM, UI::Input::KeyboardAndMouse::{GetKeyNameTextW, MapVirtualKeyA, MAPVK_VK_TO_VSC, MAPVK_VSC_TO_VK, VIRTUAL_KEY}};
use crate::windows::{WinResult, WinError};
use crate::log::*;

pub fn get_key_name(code: LPARAM) -> WinResult<OsString> {
	let mut buf = [0u16; 128];
	let res = unsafe {
		match GetKeyNameTextW(code.0 as i32, &mut buf) {
			0 => Err(WinError::from_win32()),
			sz => Ok(sz as usize),
		}
	};
	match res {
		Err(e) => Err(e),
		Ok(len @ 0..=128) => Ok(OsString::from_wide(&buf[..len])),
		Ok(_res) => {
			debug!("weird, I didn't ask for {_res}");
			Err(winerror!(ERROR_INSUFFICIENT_BUFFER, "key name too long"))
		},
	}
}

pub fn get_scan_code(vk: VIRTUAL_KEY) -> Option<NonZeroU16> {
	let vsc = unsafe {
		MapVirtualKeyA(vk.0.into(), MAPVK_VK_TO_VSC)
	};
	NonZeroU16::new(vsc as u16)
}

pub fn get_vk(vsc: u16) -> Option<VIRTUAL_KEY> {
	let vk = unsafe {
		MapVirtualKeyA(vsc.into(), MAPVK_VSC_TO_VK)
	};
	NonZeroU16::new(vk as u16)
		.map(|vk| vk.get())
		.map(VIRTUAL_KEY)
}
