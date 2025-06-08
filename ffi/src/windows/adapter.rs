#![allow(unreachable_patterns)]

use core::mem::transmute;
use crate::c_void;

#[cfg(windows)]
#[cfg(feature = "windows-060")]
pub use windows_060::Win32::{
	self as Win32_060,
	Foundation as Foundation060,
};
#[cfg(feature = "windows-core-060")]
pub use windows_core_060 as core060;
#[cfg(windows)]
#[cfg(feature = "windows-061")]
pub use windows_061::Win32::{
	self as Win32_061,
	Foundation as Foundation061,
};
#[cfg(feature = "windows-core-061")]
pub use windows_core_061 as core061;

foundation_newtype! {
	pub struct LPARAM(pub isize);
}

foundation_newtype! {
	pub struct WPARAM(pub usize);
}

foundation_newtype! {
	pub struct HMODULE(pub *mut c_void);
}

impl HMODULE {
	pub fn is_invalid(&self) -> bool {
		match self {
			#[cfg(windows)]
			#[cfg(feature = "windows-061")]
			m => Foundation061::HMODULE::is_invalid(m.into()),
			#[cfg(windows)]
			#[cfg(feature = "windows-060")]
			m => Foundation060::HMODULE::is_invalid(m.into()),
			m => !m.0.is_null(),
		}
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-060")]
impl core060::Free for HMODULE {
	unsafe fn free(&mut self) {
		<Foundation060::HMODULE as core060::Free>::free(self.into())
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-061")]
impl core061::Free for HMODULE {
	unsafe fn free(&mut self) {
		<Foundation061::HMODULE as core061::Free>::free(self.into())
	}
}

foundation_newtype! {
	pub struct HWND(pub *mut c_void);
}

impl HWND {
	pub fn is_invalid(&self) -> bool {
		match self {
			#[cfg(windows)]
			#[cfg(feature = "windows-061")]
			m => Foundation061::HWND::from(*m).is_invalid(),
			#[cfg(windows)]
			#[cfg(feature = "windows-060")]
			m => Foundation060::HWND::from(*m).is_invalid(),
			m => !m.0.is_null(),
		}
	}
}

macro_rules! foundation_newtype {
	(
		$vis:vis struct $name:ident($field_vis:vis $field_ty:ty);
	) => {
		#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
		#[repr(transparent)]
		$vis struct $name($field_vis $field_ty);

		impl Default for $name {
			#[inline]
			fn default() -> Self {
				Self(unsafe {
					::core::mem::MaybeUninit::zeroed().assume_init()
				})
			}
		}

		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		impl From<Foundation060::$name> for $name {
			fn from(v: Foundation060::$name) -> Self {
				Self(v.0)
			}
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		impl From<$name> for Foundation060::$name {
			fn from(v: $name) -> Self {
				Self(v.0)
			}
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		impl<'a> From<&'a $name> for &'a Foundation060::$name {
			fn from(v: &'a $name) -> Self {
				unsafe {
					transmute(v)
				}
			}
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		impl<'a> From<&'a mut $name> for &'a mut Foundation060::$name {
			fn from(v: &'a mut $name) -> Self {
				unsafe {
					transmute(v)
				}
			}
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		impl From<Foundation061::$name> for $name {
			fn from(v: Foundation061::$name) -> Self {
				Self(v.0)
			}
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		impl From<$name> for Foundation061::$name {
			fn from(v: $name) -> Self {
				Self(v.0)
			}
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		impl<'a> From<&'a $name> for &'a Foundation061::$name {
			fn from(v: &'a $name) -> Self {
				unsafe {
					transmute(v)
				}
			}
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		impl<'a> From<&'a mut $name> for &'a mut Foundation061::$name {
			fn from(v: &'a mut $name) -> Self {
				unsafe {
					transmute(v)
				}
			}
		}
	};
}
pub(crate) use foundation_newtype;
