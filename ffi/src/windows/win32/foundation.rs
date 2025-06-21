use crate::{
	c_void,
	cstr::{self, c_char, c_wchar},
	windows::core::HRESULT,
};

pub type ATOM = WORD;
pub type BYTE = u8;
pub type CHAR = c_char;
pub type CCHAR = c_char;
pub type WORD = u16;
pub type DWORD = u32;
pub type DWORDLONG = u64;
pub type INT = crate::c_int;
pub type UINT = crate::c_uint;
pub type LONG = crate::c_long;
pub type ULONG = crate::c_ulong;
pub type LONGLONG = crate::c_longlong;
pub type ULONGLONG = crate::c_ulonglong;
pub type FLOAT = f32;

pub type BOOL = crate::c_bool32;
pub type BOOLEAN = crate::c_bool;
pub type INT_PTR = isize;
pub type LONG_PTR = isize;
#[cfg(target_pointer_width = "32")]
pub type HALF_PTR = i16;
#[cfg(target_pointer_width = "64")]
pub type HALF_PTR = i32;

pub type LPBYTE = *mut BYTE;
pub type PBYTE = *mut BYTE;
pub type LPWORD = *mut WORD;
pub type PWORD = *mut WORD;
pub type LPDWORD = *mut DWORD;
pub type PDWORD = *mut DWORD;
pub type LPDWORDLONG = *mut DWORDLONG;
pub type PDWORDLONG = *mut DWORDLONG;
pub type LPINT = *mut INT;
pub type PINT = *mut INT;
pub type LPUINT = *mut UINT;
pub type PUINT = *mut UINT;
pub type LPLONG = *mut LONG;
pub type PLONG = *mut LONG;
pub type LPULONG = *mut ULONG;
pub type PULONG = *mut ULONG;
pub type LPLONGLONG = *mut LONGLONG;
pub type PLONGLONG = *mut LONGLONG;
pub type LPULONGLONG = *mut ULONGLONG;
pub type PULONGLONG = *mut ULONGLONG;
pub type LPBOOL = *mut BOOL;
pub type PBOOL = *mut BOOL;

pub type PCHAR = *mut CHAR;
pub type PFLOAT = *mut FLOAT;
pub type PBOOLEAN = *mut BOOLEAN;

pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type LPSTR = *mut c_char;
pub type LPWSTR = *mut c_wchar;
pub type LPCSTR = *const cstr::CStrRef;
pub type LPCWSTR = *const cstr::CStrRef16;
pub type LRESULT = LONG_PTR;

pub use crate::windows::adapter::{
	WIN32_ERROR,
	FILETIME, GENERIC_ACCESS_RIGHTS,
	LPARAM, WPARAM,
	HMODULE, HWND,
};
#[cfg(feature = "library")]
pub use crate::windows::library::free_library as FreeLibrary;

#[cfg(any(feature = "windows", feature = "windows-link"))]
pub fn SetLastError<E: Into<WIN32_ERROR>>(code: E) {
	let code = code.into();
	unsafe {
		match code {
			#[cfg(feature = "windows")]
			code => crate::windows::Win32_0xx::Foundation::SetLastError(code.into()),
			#[cfg(not(feature = "windows"))]
			code => {
				crate::windows::link!("kernel32.dll" "system" fn SetLastError(code: WIN32_ERROR));
				SetLastError(code)
			},
		}
	}
}

pub const TRUE: BOOL = BOOL::TRUE;
pub const FALSE: BOOL = BOOL::FALSE;

pub const ERROR_SUCCESS: WIN32_ERROR = WIN32_ERROR(0);
pub const ERROR_INVALID_FUNCTION: WIN32_ERROR = WIN32_ERROR(1);
pub const ERROR_FILE_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(2);
pub const ERROR_PATH_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(3);
pub const ERROR_TOO_MANY_OPEN_FILES: WIN32_ERROR = WIN32_ERROR(4);
pub const ERROR_ACCESS_DENIED: WIN32_ERROR = WIN32_ERROR(5);
pub const ERROR_INVALID_HANDLE: WIN32_ERROR = WIN32_ERROR(6);
pub const ERROR_BAD_FORMAT: WIN32_ERROR = WIN32_ERROR(11);
pub const ERROR_INVALID_ACCESS: WIN32_ERROR = WIN32_ERROR(12);
pub const ERROR_INVALID_DATA: WIN32_ERROR = WIN32_ERROR(13);
pub const ERROR_NOT_READY: WIN32_ERROR = WIN32_ERROR(21);
pub const ERROR_BAD_COMMAND: WIN32_ERROR = WIN32_ERROR(22);
pub const ERROR_BAD_CRC: WIN32_ERROR = WIN32_ERROR(23);
pub const ERROR_BAD_LENGTH: WIN32_ERROR = WIN32_ERROR(24);
pub const ERROR_HANDLE_EOF: WIN32_ERROR = WIN32_ERROR(38);
pub const ERROR_NOT_SUPPORTED: WIN32_ERROR = WIN32_ERROR(50);
pub const ERROR_NETWORK_BUSY: WIN32_ERROR = WIN32_ERROR(54);
pub const ERROR_FILE_EXISTS: WIN32_ERROR = WIN32_ERROR(80);
pub const ERROR_BROKEN_PIPE: WIN32_ERROR = WIN32_ERROR(109);
pub const ERROR_BUFFER_OVERFLOW: WIN32_ERROR = WIN32_ERROR(111);
pub const ERROR_CALL_NOT_IMPLEMENTED: WIN32_ERROR = WIN32_ERROR(120);
pub const ERROR_INSUFFICIENT_BUFFER: WIN32_ERROR = WIN32_ERROR(122);
pub const ERROR_INVALID_NAME: WIN32_ERROR = WIN32_ERROR(123);
pub const ERROR_MOD_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(126);
pub const ERROR_PROC_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(127);
pub const ERROR_BAD_ARGUMENTS: WIN32_ERROR = WIN32_ERROR(160);
pub const ERROR_BAD_PATHNAME: WIN32_ERROR = WIN32_ERROR(161);
pub const ERROR_BUSY: WIN32_ERROR = WIN32_ERROR(170);
pub const ERROR_NOT_FOUND: WIN32_ERROR = WIN32_ERROR(1168);

pub const ERROR_INVALID_PIXEL_FORMAT: WIN32_ERROR = WIN32_ERROR(2000);

pub const S_OK: HRESULT = HRESULT::OK;
pub const S_FALSE: HRESULT = HRESULT::FALSE;
pub const S_EMPTY_ERROR: HRESULT = HRESULT::EMPTY_ERROR;
pub const E_NOTIMPL: HRESULT = HRESULT::NOTIMPL;
pub const E_NOINTERFACE: HRESULT = HRESULT::NOINTERFACE;
pub const E_POINTER: HRESULT = HRESULT::POINTER;
pub const E_OUTOFMEMORY: HRESULT = HRESULT::OUTOFMEMORY;
pub const E_STRING_NOT_NULL_TERMINATED: HRESULT = HRESULT::STRING_NOT_NULL_TERMINATED;
pub const E_UNEXPECTED: HRESULT = HRESULT::UNEXPECTED;
pub const E_UAC_DISABLED: HRESULT = HRESULT::UAC_DISABLED;
pub const E_PROTOCOL_EXTENSIONS_NOT_SUPPORTED: HRESULT = HRESULT::PROTOCOL_EXTENSIONS_NOT_SUPPORTED;
pub const E_PROTOCOL_VERSION_NOT_SUPPORTED: HRESULT = HRESULT::PROTOCOL_VERSION_NOT_SUPPORTED;
pub const E_SUBPROTOCOL_NOT_SUPPORTED: HRESULT = HRESULT::SUBPROTOCOL_NOT_SUPPORTED;

pub const GENERIC_ALL: GENERIC_ACCESS_RIGHTS = GENERIC_ACCESS_RIGHTS(0x1000_0000);
pub const GENERIC_EXECUTE: GENERIC_ACCESS_RIGHTS = GENERIC_ACCESS_RIGHTS(0x2000_0000);
pub const GENERIC_READ: GENERIC_ACCESS_RIGHTS = GENERIC_ACCESS_RIGHTS(0x8000_0000);
pub const GENERIC_WRITE: GENERIC_ACCESS_RIGHTS = GENERIC_ACCESS_RIGHTS(0x4000_0000);

pub const MAX_PATH: usize = 260;

pub type Farproc = unsafe extern "system" fn() -> isize;
pub type FARPROC = Option<Farproc>;
