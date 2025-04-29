use std::{ffi::{c_void, OsString}, os::windows::ffi::OsStringExt};
use windows::{core::{Owned, Param}, Win32::{Foundation::{FreeLibrary, GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_MOD_NOT_FOUND, HMODULE, MAX_PATH}, System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleExA, GetModuleHandleExW, LoadLibraryW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT}}};
use windows_strings::{PCSTR, PCWSTR};

pub use windows::core::Error as WinError;
pub type WinResult<T> = Result<T, WinError>;

pub fn get_module_path(handle: Option<HMODULE>) -> WinResult<OsString> {
	let mut file_name_buf = [0u16; 128];
	let res = unsafe {
		let sz = GetModuleFileNameW(handle, &mut file_name_buf);
		GetLastError()
			.ok().map(|()| sz as usize)
	};
	match res {
		Err(e) if e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() => (),
		Err(e) => return Err(e),
		Ok(len @ 0..=128) => return Ok(OsString::from_wide(&file_name_buf[..len])),
		Ok(_) => {
			#[cfg(feature = "log")] {
				log::debug!("weird, I didn't ask for that");
			}
		},
	}
	
	let mut buf = vec![0u16; MAX_PATH as usize];
	let res = unsafe {
		let sz = GetModuleFileNameW(handle, &mut buf[..]);
		GetLastError()
			.ok().map(|()| sz as usize)
	};
	res.map(move |len| {
		buf.truncate(len);
		OsString::from_wide(&buf)
	})
}

pub fn get_module_from_name<P: Param<PCWSTR>>(name: P) -> WinResult<Option<HMODULE>> {
	let mut handle = HMODULE::default();
	let res = unsafe {
		let flags = GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
		GetModuleHandleExW(flags, name, &mut handle)
	};
	res.map(|()| if handle.is_invalid() && handle != HMODULE::default() { None } else { Some(handle) })
}

/// [HMODULE] lookup by pointer to a module's memory
///
/// The returned handle will only be [borroed](GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT).
pub fn get_module_from_ptr(p: *const c_void) -> WinResult<Option<HMODULE>> {
	// TODO: owned module???
	let mut handle = HMODULE::default();
	let res = unsafe {
		let flags = GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS;
		GetModuleHandleExA(flags, PCSTR(p as *const _), &mut handle)
	};
	res.map(|()| if handle.is_invalid() && handle != HMODULE::default() { None } else { Some(handle) })
}

pub fn load_library<P>(path: P) -> WinResult<Owned<HMODULE>> where
	P: Param<PCWSTR>,
{
	unsafe {
		LoadLibraryW(path)
			.map(|module| Owned::new(module))
	}
}

pub fn free_library(module: HMODULE) -> WinResult<()> {
	let res = unsafe {
		FreeLibrary(module)
	};
	match res {
		Err(e) if e.code() == ERROR_MOD_NOT_FOUND.to_hresult() => Ok(()),
		res => res,
	}
}
