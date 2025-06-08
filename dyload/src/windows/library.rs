#![allow(non_snake_case)]

use std::{ffi::{c_void, c_uchar, CString, OsStr, OsString}, os::windows::ffi::OsStringExt, ptr::NonNull, slice::from_raw_parts};
use windows::{core::{HSTRING, PCWSTR, PCSTR, Owned, Param}, Win32::{Foundation::{FreeLibrary, GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_MOD_NOT_FOUND, HMODULE, MAX_PATH}, System::LibraryLoader::{FindResourceA, GetModuleFileNameW, GetModuleHandleExA, GetModuleHandleExW, LoadLibraryExW, LoadLibraryExA, LoadResource, LockResource, SizeofResource, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, LOAD_LIBRARY_FLAGS}}};
use crate::windows::WinResult;
use crate::log::*;

pub fn load_library_path<P>(path: P, flags: LOAD_LIBRARY_FLAGS) -> WinResult<Owned<HMODULE>> where
	P: AsRef<OsStr> + Into<OsString> + Into<HSTRING>,
{
	let path_str = path.as_ref();
	unsafe {
		match path_str.is_ascii() {
			true => {
				let path: OsString = path.into();
				let path = CString::from_vec_unchecked(Vec::from(path.into_encoded_bytes()));
				load_library_a(PCSTR(path.as_ptr() as *const c_uchar), flags)
			},
			false => {
				let path: HSTRING = path.into();
				load_library_w(&path, flags)
			},
		}
	}
}

pub fn load_library_w<P>(path: P, flags: LOAD_LIBRARY_FLAGS) -> WinResult<Owned<HMODULE>> where
	P: Param<PCWSTR>,
{
	unsafe {
		LoadLibraryExW(path, None, flags)
			.map(|module| Owned::new(module))
	}
}

pub fn load_library_a<P>(path: P, flags: LOAD_LIBRARY_FLAGS) -> WinResult<Owned<HMODULE>> where
	P: Param<PCSTR>,
{
	unsafe {
		LoadLibraryExA(path, None, flags)
			.map(|module| Owned::new(module))
	}
}

pub fn LDR_IS_DATAFILE(handle: HMODULE) -> bool {
	handle.0 as usize & 1 != 0
}

pub fn LDR_IS_IMAGEMAPPING(handle: HMODULE) -> bool {
	handle.0 as usize & 2 != 0
}

pub fn LDR_IS_RESOURCE(handle: HMODULE) -> bool {
	LDR_IS_DATAFILE(handle) || LDR_IS_IMAGEMAPPING(handle)
}

pub fn get_module_path(handle: Option<HMODULE>) -> WinResult<OsString> {
	let mut file_name_buf = [0u16; 128];
	let res = unsafe {
		match GetModuleFileNameW(handle, &mut file_name_buf) {
			sz @ (0 | 128) => GetLastError()
				.ok().map(|()| sz as usize),
			sz => Ok(sz as usize),
		}
	};
	match res {
		Err(e) if e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() => (),
		Err(e) => return Err(e),
		Ok(len @ 0..=128) => return Ok(OsString::from_wide(&file_name_buf[..len])),
		Ok(_) => {
			debug!("weird, I didn't ask for that");
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

pub fn free_library(module: HMODULE) -> WinResult<()> {
	let res = unsafe {
		FreeLibrary(module)
	};
	match res {
		Err(e) if e.code() == ERROR_MOD_NOT_FOUND.to_hresult() => Ok(()),
		res => res,
	}
}

pub fn MAKERESOURCEA(resource: u16) -> PCSTR {
	PCSTR::from_raw(resource as usize as *const _)
}

pub unsafe fn find_resource<I: Param<PCSTR>, T: Param<PCSTR>>(module: &HMODULE, id: I, kind: T) -> WinResult<&[u8]> {
	unsafe {
		let src = FindResourceA(Some(*module), id, kind)?;
		let resource = LoadResource(Some(*module), src)?;
		let size = SizeofResource(Some(*module), src) as usize;
		NonNull::new(LockResource(resource))
			.ok_or_else(|| winerror!(ERROR_RESOURCE_NOT_PRESENT, fmt: "failed to lock resource {module:?}/{src:?}/{resource:?}"))
			.map(|ptr| from_raw_parts(ptr.as_ptr() as *const c_void as *const u8, size))
	}
}
