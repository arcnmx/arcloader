#![allow(non_snake_case, non_camel_case_types)]

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
				let _e: $crate::windows::WinError = panic!("winerror {:?}: {}", $e, $msg);
				_e
			},
			#[cfg(any(not(debug_assertions), windows))]
			() => $crate::windows::WinError::new($e.to_hresult(), $msg),
		}
	};
	($e:expr) => {
		$crate::windows::WinError::from($e.to_hresult())
	};
	() => {
		$crate::windows::winerror! { ERROR_CALL_NOT_IMPLEMENTED, "TODO" }
	};
}
pub use winerror;

pub use windows_core::{self as core, Error as WinError, Result as WinResult};
#[cfg(windows)]
pub use windows::Win32;
#[cfg(not(windows))]
#[path = "win32.rs"]
pub mod Win32;

#[cfg(windows)]
mod library;
#[cfg(windows)]
pub use self::library::{
	free_library, load_library_path, load_library_w,
	get_module_path, get_module_from_name, get_module_from_ptr,
	find_resource,
	LDR_IS_DATAFILE, LDR_IS_RESOURCE, LDR_IS_IMAGEMAPPING,
	MAKERESOURCEA,
};

#[cfg(windows)]
#[cfg(feature = "keyboard")]
mod keyboard;
#[cfg(windows)]
#[cfg(feature = "keyboard")]
pub use self::keyboard::{get_key_name, get_scan_code, get_vk};
