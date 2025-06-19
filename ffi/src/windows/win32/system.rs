#[cfg(feature = "library")]
pub mod LibraryLoader {
	use crate::windows::adapter::windows_newtype;

	windows_newtype! {
		pub struct LibraryLoader::LOAD_LIBRARY_FLAGS(pub u32);
	}

	pub const LOAD_WITH_ALTERED_SEARCH_PATH: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x0008);
	pub const LOAD_LIBRARY_REQUIRE_SIGNED_TARGET: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x0080);
	pub const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x0100);
	pub const LOAD_LIBRARY_SEARCH_APPLICATION_DIR: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x0200);
	pub const LOAD_LIBRARY_SEARCH_USER_DIRS: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x0400);
	pub const LOAD_LIBRARY_SEARCH_SYSTEM32: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x0800);
	pub const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x1000);
	pub const LOAD_LIBRARY_SAFE_CURRENT_DIRS: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x2000);
	pub const LOAD_LIBRARY_SEARCH_SYSTEM32_NO_FORWARDER: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0x4000);

	#[cfg(feature = "library")]
	pub use crate::windows::library::{
		load_library_a as LoadLibraryA,
		load_library_w as LoadLibraryW,
		get_proc_address as GetProcAddress,
	};
}
pub mod Diagnostics {
	pub mod Debug {
		#[cfg(all(
			windows,
			feature = "windows-core",
			not(all(feature = "windows", feature = "winerror")),
		))]
		pub use crate::windows::core0xx::imp::EncodePointer;
		#[cfg(all(windows, feature = "windows", feature = "winerror"))]
		pub use crate::windows::Win32_0xx::System::Diagnostics::Debug::{EncodePointer, DecodePointer};
	}
}
pub mod SystemServices {
	pub const DLL_PROCESS_ATTACH: u32 = 1;
	pub const DLL_PROCESS_DETACH: u32 = 0;
	pub const DLL_THREAD_ATTACH: u32 = 2;
	pub const DLL_THREAD_DETACH: u32 = 3;
}

#[cfg(feature = "library")]
pub mod SystemInformation {
	use crate::{
		c_char, c_wchar,
		windows::core::{Error, Result},
	};

	pub fn GetSystemDirectoryA(buffer: &mut [c_char]) -> Result<usize> {
		#![allow(unreachable_patterns, unreachable_code)]
		let res: u32 = unsafe {
			match () {
				#[cfg(all(windows, feature = "windows"))]
				() => crate::windows::Win32_0xx::System::SystemInformation::GetSystemDirectoryA(::core::mem::transmute(buffer)),
				#[cfg(all(windows, not(feature = "windows"), feature = "windows-link"))]
				() => {
					crate::windows::link!("kernel32.dll" "system" fn GetSystemDirectoryA(buffer: *mut c_char, size: u32) -> u32);
					GetSystemDirectoryA(buffer.as_ptr(), buffer.len() as _)
				},
				_ => return Err(crate::windows::Win32::Foundation::ERROR_CALL_NOT_IMPLEMENTED.into()),
			}
		};
		#[cfg(windows)]
		match res {
			0 => Err(Error::from_win32()),
			len => Ok(len as usize),
		}
	}

	pub fn GetSystemDirectoryW(buffer: &mut [c_wchar]) -> Result<usize> {
		#![allow(unreachable_patterns, unreachable_code)]
		let res = unsafe {
			match () {
				#[cfg(all(windows, feature = "windows"))]
				() => crate::windows::Win32_0xx::System::SystemInformation::GetSystemDirectoryW(::core::mem::transmute(buffer)),
				#[cfg(all(windows, not(feature = "windows"), feature = "windows-link"))]
				() => {
					crate::windows::link!("kernel32.dll" "system" fn GetSystemDirectoryW(buffer: *mut c_wchar, size: u32) -> u32);
					GetSystemDirectoryW(buffer.as_ptr(), buffer.len() as _)
				},
				_ => return Err(crate::windows::Win32::Foundation::ERROR_CALL_NOT_IMPLEMENTED.into()),
			}
		};
		#[cfg(windows)]
		match res {
			0 => Err(Error::from_win32()),
			len => Ok(len as usize),
		}
	}
}
