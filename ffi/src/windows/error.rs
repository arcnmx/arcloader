#![allow(unreachable_patterns)]

use core::{num::NonZeroU32, slice::from_raw_parts};
use crate::{
	cstr::{CStr, CStrPtr, CStrPtr16, EMPTY_CSTR},
	windows::adapter::{windows_adapter, HRESULT, HMODULE},
};

pub use crate::windows::core::{Error, Result};

#[cfg(windows)]
pub fn get_error_message_a(error: HRESULT, module: Option<HMODULE>, language_id: Option<NonZeroU32>) -> Option<&'static CStr> {
	#![allow(unreachable_code)]

	let mut out = None::<CStrPtr>;
	let language_id = language_id.map(|v| v.get()).unwrap_or(0);
	let res: u32 = windows_adapter! { match windows as windows0xx
		=> unsafe {
			use windows0xx::{
				core::PSTR,
				Win32::System::Diagnostics::Debug::{self, FormatMessageA},
			};

			let mut flags = Debug::FORMAT_MESSAGE_FROM_SYSTEM | Debug::FORMAT_MESSAGE_IGNORE_INSERTS | Debug::FORMAT_MESSAGE_ARGUMENT_ARRAY | Debug::FORMAT_MESSAGE_ALLOCATE_BUFFER;
			if let Some(..) = module {
				flags |= Debug::FORMAT_MESSAGE_FROM_HMODULE;
			}
			FormatMessageA(
				flags,
				module.map(|m| m.0 as *const _),
				-error.0 as u32,
				language_id,
				PSTR(&mut out as *mut _ as *mut _),
				0, None,
			)
		},
		_ => return Some(EMPTY_CSTR),
	};
	
	match res as usize {
		0 => None,
		len => Some(out
			.map(|m| unsafe {
				CStr::from_bytes_with_nul_unchecked(from_raw_parts(m.as_uptr(), len + 1))
			})
			.unwrap_or(EMPTY_CSTR)
		),
	}
}

pub fn try_get_error_message_a(error: HRESULT, module: Option<HMODULE>, language_id: Option<NonZeroU32>) -> Result<&'static CStr> {
	match get_error_message_a(error, module, language_id) {
		None => Err(Error::from_win32()),
		Some(m) if m.is_empty() => Err(winerror!("empty message")),
		Some(m) => Ok(m),
	}
}

#[cfg(windows)]
pub fn get_error_message_w(error: HRESULT, module: Option<HMODULE>, language_id: Option<NonZeroU32>) -> Option<CStrPtr16<'static>> {
	#![allow(unreachable_code)]

	let mut out = None::<CStrPtr16>;
	let language_id = language_id.map(|v| v.get()).unwrap_or(0);
	let res: u32 = windows_adapter! { match windows as windows0xx
		=> unsafe {
			use windows0xx::{
				core::PWSTR,
				Win32::System::Diagnostics::Debug::{self, FormatMessageW},
			};

			let mut flags = Debug::FORMAT_MESSAGE_FROM_SYSTEM | Debug::FORMAT_MESSAGE_IGNORE_INSERTS | Debug::FORMAT_MESSAGE_ARGUMENT_ARRAY | Debug::FORMAT_MESSAGE_ALLOCATE_BUFFER;
			if let Some(..) = module {
				flags |= Debug::FORMAT_MESSAGE_FROM_HMODULE;
			}
			FormatMessageW(
				flags,
				module.map(|m| m.0 as *const _),
				-error.0 as u32,
				language_id,
				PWSTR(&mut out as *mut _ as *mut _),
				0, None,
			)
		},
		_ => return Some(CStrPtr16::EMPTY),
	};
	
	match res as usize {
		0 => None,
		_len => Some(out.unwrap_or(CStrPtr16::EMPTY)),
	}
}

pub fn try_get_error_message_w(error: HRESULT, module: Option<HMODULE>, language_id: Option<NonZeroU32>) -> Result<CStrPtr16<'static>> {
	match get_error_message_w(error, module, language_id) {
		None => Err(Error::from_win32()),
		Some(m) if m.is_empty() => Err(winerror!("empty message")),
		Some(m) => Ok(m),
	}
}

#[macro_export]
macro_rules! winerror {
	($e:ident, $($tt:tt)*) => {
		$crate::windows::winerror! { $crate::windows::Win32::Foundation::$e, $($tt)* }
	};
	(fmt: $($tt:tt)*) => {
		$crate::windows::winerror! { ERROR_INVALID_DATA, fmt: $($tt)* }
	};
	($msg:literal) => {
		$crate::windows::winerror! { ERROR_INVALID_DATA, $msg }
	};
	($e:ident) => {
		$crate::windows::winerror! { $crate::windows::Win32::Foundation::$e }
	};
	($e:expr, fmt: $($tt:tt)*) => {
		$crate::windows::winerror! { $e, format!($($tt)*) }
	};
	($e:expr, $msg:expr) => {
		match () {
			#[cfg(all(debug_assertions, not(windows)))]
			#[allow(unreachable_code)]
			() => {
				let _e: $crate::windows::core::Error = panic!("winerror {:?}: {}", $e, $msg);
				_e
			},
			#[cfg(any(not(debug_assertions), windows))]
			() => $crate::windows::core::Error::new($e.to_hresult(), $msg),
		}
	};
	($e:expr) => {
		$crate::windows::core::Error::from($e.to_hresult())
	};
	() => {
		$crate::windows::winerror! { ERROR_CALL_NOT_IMPLEMENTED, "TODO" }
	};
}
pub use winerror;
