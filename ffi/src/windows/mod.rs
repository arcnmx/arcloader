#![allow(non_snake_case, non_camel_case_types)]

pub use self::core::{Error as WinError, Result as WinResult};
#[cfg(feature = "winerror")]
pub use self::winerror::winerror;

#[cfg(feature = "windows-060")]
pub use ::windows_060 as windows060;
#[cfg(feature = "windows-061")]
pub use ::{windows_061 as windows0xx, windows_061 as windows061};
#[cfg(all(feature = "windows-060", not(any(feature = "windows-061"))))]
pub use ::windows_060 as windows0xx;

#[cfg(feature = "windows-060")]
pub use self::adapter::Win32_060;
#[cfg(feature = "windows-061")]
pub use self::adapter::{Win32_061 as Win32_0xx, Win32_061};
#[cfg(all(feature = "windows-060", not(any(feature = "windows-061"))))]
pub use self::adapter::Win32_060 as Win32_0xx;

#[cfg(feature = "windows-core-060")]
pub use self::adapter::core060;
#[cfg(feature = "windows-core-061")]
pub use self::adapter::{core061 as core0xx, core061};
#[cfg(all(feature = "windows-core-060", not(any(feature = "windows-core-061"))))]
pub use self::adapter::core060 as core0xx;

pub mod adapter;
#[cfg(feature = "winerror")]
#[path = "error.rs"]
pub mod winerror;

pub mod core {
	#[doc(no_inline)]
	pub use super::adapter::{
		GUID,
		HRESULT,
		PSTR, PCSTR,
		PWSTR, PCWSTR,
		Error,
	};
	pub type Result<T> = ::core::result::Result<T, Error>;
}

#[path = "win32/mod.rs"]
pub mod Win32;
