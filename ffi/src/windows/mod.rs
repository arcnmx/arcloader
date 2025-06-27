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

#[cfg(feature = "windows-result-03")]
pub use ::windows_result_03 as result03;

#[cfg(all(windows, feature = "windows-strings-03"))]
pub use ::windows_strings_03 as strings03;
#[cfg(all(windows, feature = "windows-strings-04"))]
pub use ::{windows_strings_04 as strings0xx, windows_strings_04 as strings04};
#[cfg(all(windows, feature = "windows-strings-03", not(any(feature = "windows-strings-04"))))]
pub use ::windows_strings_03 as strings0xx;

#[cfg(feature = "windows-link")]
pub use ::windows_link::link;
#[cfg(not(feature = "windows-link"))]
#[macro_export]
macro_rules! link {
	($dll:literal $abi:tt $vis:vis fn $name:ident($($arg:ident: $arg_ty:ty),*$(,)?) $(-> $res:ty)?) => {
		/*pub(crate) fn $name($($arg: $arg_ty),*) $(-> $res)? {
		}*/
	};
}

pub mod adapter;

#[cfg(windows)]
#[cfg(feature = "com")]
pub mod com;

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

#[cfg(feature = "library")]
pub mod library;

#[cfg(windows)]
#[cfg(feature = "wic")]
pub mod wic;

#[cfg(feature = "winerror")]
#[path = "error.rs"]
pub mod winerror;

#[path = "win32/mod.rs"]
pub mod Win32;
