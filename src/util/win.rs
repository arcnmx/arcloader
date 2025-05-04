use std::{char::DecodeUtf16, ffi::{c_void, OsString}, io::{self, BufRead}, iter, os::windows::ffi::OsStringExt, ptr::NonNull, slice::from_raw_parts};
use windows::{core::{Owned, Param}, Win32::{Foundation::{FreeLibrary, GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_MOD_NOT_FOUND, ERROR_RESOURCE_NOT_PRESENT, HGLOBAL, HMODULE, MAX_PATH}, System::LibraryLoader::{FindResourceA, GetModuleFileNameW, GetModuleHandleExA, GetModuleHandleExW, LoadLibraryW, LoadResource, LockResource, SizeofResource, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT}}};
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

#[allow(non_snake_case)]
pub fn MAKERESOURCEA(resource: u16) -> PCSTR {
	PCSTR::from_raw(resource as usize as *const _)
}

pub unsafe fn find_resource<I: Param<PCSTR>, T: Param<PCSTR>>(module: &HMODULE, id: I, kind: T) -> WinResult<&[u8]> {
	unsafe {
		let src = FindResourceA(Some(*module), id, kind)?;
		let resource = LoadResource(Some(*module), src)?;
		let size = SizeofResource(Some(*module), src) as usize;
		NonNull::new(LockResource(resource))
			.ok_or_else(|| WinError::new(ERROR_RESOURCE_NOT_PRESENT.to_hresult(), format!("failed to lock resource {module:?}/{src:?}/{resource:?}")))
			.map(|ptr| from_raw_parts(ptr.as_ptr() as *const c_void as *const u8, size))
	}
}

pub struct WideUtf8Reader<'a> {
	pub decoder: iter::Fuse<DecodeUtf16<iter::Copied<std::slice::Iter<'a, u16>>>>,
	pub buf: [u8; 4],
	pub buf_pos: usize,
	pub buf_len: usize,
}

impl<'a> WideUtf8Reader<'a> {
	pub fn new(widebuf: &'a [u16]) -> Self {
		Self {
			decoder: char::decode_utf16(widebuf.iter().copied()).fuse(),
			buf: [0u8; 4],
			buf_pos: 0,
			buf_len: 0,
		}
	}
}

impl<'a> io::BufRead for WideUtf8Reader<'a> {
	fn fill_buf(&mut self) -> io::Result<&[u8]> {
		if self.buf_pos >= self.buf_len {
			let res = self.decoder.next().transpose()
				.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
			match res {
				Some(c) => {
					let len = c.encode_utf8(&mut self.buf).len();
					self.buf_len = len;
				},
				None => {
					self.buf_len = 0;
				},
			}
			self.buf_pos = 0;
		}
		Ok(&self.buf[self.buf_pos..self.buf_len])
	}

	fn consume(&mut self, amt: usize) {
		self.buf_pos += amt;
	}
}

impl<'a> io::Read for WideUtf8Reader<'a> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let src = self.fill_buf()?;
		let read_len = src.len().min(buf.len());
		buf[..read_len].copy_from_slice(&src[..read_len]);
		self.consume(read_len);
		Ok(read_len)
	}
}
