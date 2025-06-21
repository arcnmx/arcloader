#![allow(unreachable_patterns)]

use core::{fmt, mem::transmute, num::NonZeroI32, ops};
#[cfg(feature = "alloc")]
use std::borrow::Cow;
#[cfg(feature = "std")]
use std::{error::Error as StdError, io};
use crate::{c_void, c_char, c_wchar, cstr::{CStrPtr, CStrPtr16}};

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

#[cfg(feature = "std")]
pub type ErrorInfo = String;
#[cfg(not(feature = "std"))]
pub type ErrorInfo = ::core::convert::Infallible;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error {
	code: NonZeroI32,
	info: Option<ErrorInfo>,
}

impl Error {
	#[doc(hidden)]
	pub const S_EMPTY_ERROR: NonZeroI32 = unsafe { NonZeroI32::new_unchecked(HRESULT::EMPTY_ERROR.0) };
	#[doc(hidden)]
	pub const fn hresult_code(code: HRESULT) -> NonZeroI32 {
		match code.err_code() {
			Some(c) => c,
			None => Self::S_EMPTY_ERROR,
		}
	}

	#[inline]
	pub const fn empty() -> Self {
		Self {
			code: Self::S_EMPTY_ERROR,
			info: None,
		}
	}

	#[inline]
	pub const fn from_parts(code: HRESULT, info: Option<ErrorInfo>) -> Self {
		Self {
			code: Self::hresult_code(code),
			info,
		}
	}

	#[inline]
	pub const fn with_hresult_code(code: NonZeroI32) -> Self {
		Self {
			code,
			info: None,
		}
	}

	#[inline]
	pub const fn with_hresult(code: HRESULT) -> Self {
		Self::with_hresult_code(Self::hresult_code(code))
	}

	#[inline]
	pub fn from_hresult<C: Into<HRESULT>>(code: C) -> Self {
		Self::with_hresult(code.into())
	}

	pub fn new<C: Into<HRESULT>, T: AsRef<str> + Into<String>>(code: C, msg: T) -> Self {
		match msg.as_ref() {
			msg if msg.is_empty() =>
				Self::from_hresult(code),
			_ => Self::from_parts(code.into(), match msg {
				#[cfg(feature = "std")]
				msg => Some(msg.into()),
				#[cfg(not(feature = "std"))]
				_ => None,
			}),
		}
	}

	pub const fn code(&self) -> HRESULT {
		HRESULT(match self.code {
			Self::S_EMPTY_ERROR => 0,
			code => code.get(),
		})
	}

	#[cfg(feature = "std")]
	pub fn message(&self) -> Cow<'_, str> {
		match self.extra_info() {
			Some(msg) => Cow::Borrowed(msg),
			None => self.code().message(),
		}
	}

	#[inline]
	pub const fn extra_info(&self) -> Option<&ErrorInfo> {
		self.info.as_ref()
	}

	#[cfg(windows)]
	pub fn from_win32() -> Self {
		Self::from_hresult(HRESULT::last_error())
	}

	#[cfg(windows)]
	#[cfg(any(feature = "windows", feature = "windows-link"))]
	pub fn set_win32(&self) {
		WIN32_ERROR::from(self.code()).set_last_error()
	}
}

#[cfg(feature = "windows-core-060")]
impl From<Error> for core060::Error {
	fn from(e: Error) -> Self {
		match e.extra_info() {
			None => core060::Error::from_hresult(e.code().into()),
			#[cfg(feature = "std")]
			Some(info) =>
				core060::Error::new(e.code().into(), info),
			#[cfg(not(feature = "std"))]
			Some(&info) => match info {},
		}
	}
}
#[cfg(feature = "windows-core-060")]
impl From<core060::Error> for Error {
	fn from(e: core060::Error) -> Self {
		match e.as_ptr() {
			p if p.is_null() => Error::from_hresult(e.code()),
			_ => Error::new(e.code(), e.message()),
		}
	}
}
#[cfg(feature = "windows-core-061")]
impl From<Error> for core061::Error {
	fn from(e: Error) -> Self {
		match e.extra_info() {
			None => core061::Error::from_hresult(e.code().into()),
			#[cfg(feature = "std")]
			Some(info) =>
				core061::Error::new(e.code().into(), info),
			#[cfg(not(feature = "std"))]
			Some(&info) => match info {},
		}
	}
}
#[cfg(feature = "windows-core-061")]
impl From<core061::Error> for Error {
	fn from(e: core061::Error) -> Self {
		match e.as_ptr() {
			p if p.is_null() => Error::from_hresult(e.code()),
			_ => Error::new(e.code(), e.message()),
		}
	}
}

impl From<HRESULT> for Error {
	fn from(code: HRESULT) -> Self {
		Self::with_hresult(code)
	}
}
impl From<Error> for HRESULT {
	fn from(e: Error) -> Self {
		e.code()
	}
}
#[cfg(feature = "windows-core-061")]
impl From<core061::HRESULT> for Error {
	fn from(code: core061::HRESULT) -> Self {
		Self::from_hresult(code)
	}
}
#[cfg(feature = "windows-core-060")]
impl From<core060::HRESULT> for Error {
	fn from(code: core060::HRESULT) -> Self {
		Self::from_hresult(code)
	}
}
#[cfg(feature = "windows-core-061")]
impl From<Error> for core061::HRESULT {
	fn from(e: Error) -> Self {
		e.code().into()
	}
}
#[cfg(feature = "windows-core-060")]
impl From<Error> for core060::HRESULT {
	fn from(e: Error) -> Self {
		e.code().into()
	}
}

#[cfg(feature = "std")]
impl From<HRESULT> for io::Error {
	fn from(e: HRESULT) -> Self {
		io::Error::from_raw_os_error(e.into())
	}
}

#[cfg(feature = "std")]
impl From<io::Error> for HRESULT {
	fn from(e: io::Error) -> Self {
		match e.raw_os_error() {
			Some(code) => HRESULT(code),
			None => WIN32_ERROR::from(e.kind()).into(),
		}
	}
}

#[cfg(feature = "std")]
impl StdError for Error {
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:#08X}", self.code())?;
		#[cfg(feature = "std")]
		if let Some(msg) = self.extra_info() {
			return write!(f, ": {}", msg)
		}
		#[cfg(feature = "alloc")]
		if let Some(msg) = self.code().try_message() {
			return write!(f, ": {}", msg)
		}

		Ok(())
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut f = f.debug_struct("windows::Error");

		f.field("code", &format_args!("{:#08X}", self.code()));
		#[cfg(feature = "std")] {
			f.field("info", &self.info);
		}
		#[cfg(feature = "alloc")]
		if let Some(msg) = self.code().try_message() {
			f.field("message", &msg);
		}

		f.finish()
	}
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct GUID(pub [u32; 4]);

windows_newtype! {
	impl From for core::GUID(pub [u32; 4]);
}
windows_newtype! {
	impl From@transparent for GUID(pub [u32; 4]);
}
impl AsRef<GUID> for GUID {
	#[inline]
	fn as_ref(&self) -> &GUID {
		self
	}
}

impl GUID {
	pub fn new<D: Into<GuidData>>(data: D) -> Self {
		Self::with_data(data.into())
	}

	pub const fn zeroed() -> Self {
		Self([0u32; 4])
	}

	pub const fn from_values(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
		Self::with_data(GuidData {
			data1,
			data2,
			data3,
			data4,
		})
	}

	pub const fn with_data(data: GuidData) -> Self {
		unsafe {
			transmute(data)
		}
	}

	pub const fn into_data(self) -> GuidData {
		unsafe {
			transmute(self)
		}
	}

	pub const fn data(&self) -> &GuidData {
		unsafe {
			transmute(self)
		}
	}

	pub fn data_mut(&mut self) -> &mut GuidData {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "std")]
	pub const fn to_u128(&self) -> u128 {
		self.into_data().to_u128()
	}

	#[cfg(feature = "std")]
	pub const fn from_u128(data: u128) -> Self {
		Self::with_data(GuidData::from_u128(data))
	}

	#[cfg(feature = "windows-core-060")]
	pub const fn from_060(data: core060::GUID) -> Self {
		unsafe {
			transmute(data)
		}
	}
	#[cfg(feature = "windows-core-061")]
	pub const fn from_061(data: core061::GUID) -> Self {
		unsafe {
			transmute(data)
		}
	}
	#[cfg(feature = "windows-core-060")]
	pub const fn to_060(self) -> core060::GUID {
		unsafe {
			transmute(self)
		}
	}
	#[cfg(feature = "windows-core-061")]
	pub const fn to_061(self) -> core061::GUID {
		unsafe {
			transmute(self)
		}
	}
}

impl ops::Deref for GUID {
	type Target = GuidData;

	fn deref(&self) -> &Self::Target {
		self.data()
	}
}

impl ops::DerefMut for GUID {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.data_mut()
	}
}

impl fmt::Display for GUID {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self.data(), f)
	}
}

impl fmt::Debug for GUID {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("GUID")
			.field(&format_args!("{}", self))
			.finish()
	}
}

#[cfg(feature = "std")]
impl From<u128> for GUID {
	fn from(data: u128) -> Self {
		Self::from_u128(data)
	}
}

#[cfg(feature = "std")]
impl From<GUID> for u128 {
	fn from(data: GUID) -> Self {
		data.to_u128()
	}
}

impl From<GuidData> for GUID {
	fn from(data: GuidData) -> Self {
		Self::with_data(data)
	}
}

#[test]
#[cfg(feature = "std")]
fn guid_u128() {
	let value128 = 0xf5c7ad2d_6a8d_43dd_a7a8_a29935261ae9;
	let parts = GUID::from_values(0xf5c7ad2d, 0x6a8d, 0x43dd, [0xa7, 0xa8, 0xa2, 0x99, 0x35, 0x26, 0x1a, 0xe9]);
	let to_u128 = parts.to_u128();
	if to_u128 != value128 {
		panic!("to_u128() = {to_u128:16x}, expected {value128:16x}")
	}
	let guid128 = GUID::from_u128(value128);
	assert_eq!(parts, guid128);
	assert_eq!(&parts.to_string()[..], "F5C7AD2D-6A8D-43DD-A7A8-A29935261AE9");
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GuidData {
	pub data1: u32,
	pub data2: u16,
	pub data3: u16,
	pub data4: [u8; 8],
}

impl GuidData {
	pub const fn data4_16(&self) -> u16 {
		let [b0, b1, ..] = &self.data4;
		u16::from_be_bytes([*b0, *b1])
	}

	pub const fn data4_64(&self) -> u64 {
		let [_, _, b2, b3, b4, b5, b6, b7] = self.data4;
		u64::from_be_bytes([0, 0, b2, b3, b4, b5, b6, b7])
	}

	#[cfg(all(feature = "std", target_endian = "little"))]
	pub const fn to_u128(self) -> u128 {
		let Self { data1, data2, data3, data4 } = self;
		let data4 = u64::from_be_bytes(data4);
		unsafe {
			let data32: u32 = transmute([data3, data2]);
			let data321: u64 = transmute([data32, data1]);
			transmute([data4, data321])
		}
	}

	#[cfg(all(feature = "std", target_endian = "big"))]
	pub const fn to_u128(self) -> u128 {
		unsafe {
			transmute(self)
		}
	}

	#[cfg(feature = "std")]
	pub const fn from_u128(data: u128) -> Self {
		unsafe {
			let data: Self = transmute(data.to_be());
			#[cfg(target_endian = "little")]
			let data = {
				let mut data = data;
				data.data1 = u32::from_be(data.data1);
				data.data2 = u16::from_be(data.data2);
				data.data3 = u16::from_be(data.data3);
				data
			};
			data
		}
	}
}

impl From<GUID> for GuidData {
	#[inline]
	fn from(data: GUID) -> Self {
		data.into_data()
	}
}
#[cfg(feature = "windows-core-060")]
impl From<core060::GUID> for GuidData {
	#[inline]
	fn from(data: core060::GUID) -> Self {
		GUID::from_060(data).into_data()
	}
}
#[cfg(feature = "windows-core-061")]
impl From<core061::GUID> for GuidData {
	#[inline]
	fn from(data: core061::GUID) -> Self {
		GUID::from_061(data).into_data()
	}
}

impl fmt::Display for GuidData {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { data1, data2, data3, .. } = self;
		write!(f, "{data1:08X}-{data2:04X}-{data3:04X}-{:04X}-{:012X}", self.data4_16(), self.data4_64())
	}
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct FILETIME {
	pub dwLowDateTime: u32,
	pub dwHighDateTime: u32,
}

windows_newtype! {
	impl From for Foundation::FILETIME(pub [u32; 2]);
}
windows_newtype! {
	impl From@transparent for FILETIME(pub [u32; 2]);
}
windows_newtype! {
	impl From@transparent for FILETIME(pub u64);
}

windows_newtype! {
	pub struct core::HRESULT(pub i32);
}

impl HRESULT {
	pub const fn with_win32(e: WIN32_ERROR) -> Self {
		e.to_hresult()
	}

	pub fn from_win32<E: Into<WIN32_ERROR>>(e: E) -> Self {
		Self::with_win32(e.into())
	}

	pub const fn err_code(self) -> Option<NonZeroI32> {
		unsafe {
			transmute(self)
		}
	}

	pub const fn ok(self) -> Result<(), Error> {
		match self.err_code() {
			None => Ok(()),
			Some(code) => Err(Error {
				code,
				info: None,
			}),
		}
	}

	#[cfg(windows)]
	pub fn last_error() -> Self {
		Self::from_win32(WIN32_ERROR::last_error())
	}

	#[cfg(feature = "alloc")]
	pub fn message(self) -> Cow<'static, str> {
		#[cfg(feature = "winerror")]
		if let Some(m) = self.try_message() {
			return m
		}

		match self.err_code().map(|c| c.get() as u32) {
			None => Cow::Borrowed("ERROR_SUCCESS"),
			Some(code) => Cow::Owned(format!("{code:#08x}")),
		}
	}

	#[cfg(feature = "alloc")]
	pub fn try_message(self) -> Option<Cow<'static, str>> {
		match self.err_code() {
			#[cfg(feature = "winerror")]
			Some(..) => match crate::windows::winerror::get_error_message_a(self, None, None) {
				Some(m) if !m.is_empty() => Some(m.to_string_lossy()),
				_ => None,
			},
			_ => None,
		}
	}
}

impl HRESULT {
	pub const OK: Self = Self(0);
	pub const FALSE: Self = Self(1);
	pub const EMPTY_ERROR: Self = Self(i32::from_be_bytes(*b"S_OK"));

	pub const BOUNDS: Self = Self(0x8000_000b_u32 as i32);
	pub const CHANGED_STATE: Self = Self(0x8000_000c_u32 as i32);
	pub const ILLEGAL_METHOD_CALL: Self = Self(0x8000_000e_u32 as i32);
	pub const ILLEGAL_STATE_CHANGE: Self = Self(0x8000_000d_u32 as i32);
	pub const ILLEGAL_DELEGATE_ASSIGNMENT: Self = Self(0x8000_0018u32 as i32);
	pub const NOTIMPL: Self = Self(0x8000_4001u32 as i32);
	pub const NOINTERFACE: Self = Self(0x8000_4002u32 as i32);
	pub const POINTER: Self = Self(0x8000_4003u32 as i32);
	pub const ABORT: Self = Self(0x8000_4004u32 as i32);
	pub const FAIL: Self = Self(0x8000_4005u32 as i32);
	pub const OUTOFMEMORY: Self = Self(0x8000_700e_u32 as i32);
	pub const STRING_NOT_NULL_TERMINATED: Self = Self(0x8000_0017u32 as i32);
	pub const UNEXPECTED: Self = Self(0x8000_ffff_u32 as i32);
	pub const ACCESSDENIED: Self = Self(0x8007_0005u32 as i32);
	pub const HANDLE: Self = Self(0x8007_0006u32 as i32);
	pub const INVALIDARG: Self = Self(0x8007_0057u32 as i32);
	pub const UAC_DISABLED: Self = Self(0x8027_0252u32 as i32);
	pub const PROTOCOL_EXTENSIONS_NOT_SUPPORTED: Self = Self(0x8376_0003u32 as i32);
	pub const PROTOCOL_VERSION_NOT_SUPPORTED: Self = Self(0x8376_0005u32 as i32);
	pub const SUBPROTOCOL_NOT_SUPPORTED: Self = Self(0x8376_0004u32 as i32);
}

impl HRESULT {
	pub const CO_ASYNC_WORK_REJECTED: Self = Self(0x8000_4029u32 as i32);
	pub const CO_ATTEMPT_TO_CREATE_OUTSIDE_CLIENT_CONTEXT: Self = Self(0x8000_4024u32 as i32);
	pub const CO_BAD_SERVER_NAME: Self = Self(0x8000_4014u32 as i32);
	pub const CO_CANT_REMOTE: Self = Self(0x8000_4013u32 as i32);
	pub const CO_CLASS_DISABLED: Self = Self(0x8000_4027u32 as i32);
	pub const CO_CLRNOTAVAILABLE: Self = Self(0x8000_4028u32 as i32);
	pub const CO_CLSREG_INCONSISTENT: Self = Self(0x8000_401f_u32 as i32);
	pub const CO_CREATEPROCESS_FAILURE: Self = Self(0x8000_4018u32 as i32);
	pub const CO_IIDREG_INCONSISTENT: Self = Self(0x8000_4020u32 as i32);
	pub const CO_INIT_CLASS_CACHE: Self = Self(0x8000_4009u32 as i32);
	pub const CO_INIT_MEMORY_ALLOCATOR: Self = Self(0x8000_4008u32 as i32);
	pub const CO_INIT_ONLY_SINGLE_THREADED: Self = Self(0x8000_4012u32 as i32);
	pub const CO_INIT_RPC_CHANNEL: Self = Self(0x8000_400a_u32 as i32);
	pub const CO_INIT_SCM_EXEC_FAILURE: Self = Self(0x8000_4011u32 as i32);
	pub const CO_INIT_SCM_FILE_MAPPING_EXISTS: Self = Self(0x8000_400f_u32 as i32);
	pub const CO_INIT_SCM_MAP_VIEW_OF_FILE: Self = Self(0x8000_4010u32 as i32);
	pub const CO_INIT_SCM_MUTEX_EXISTS: Self = Self(0x8000_400e_u32 as i32);
	pub const CO_INIT_SHARED_ALLOCATOR: Self = Self(0x8000_4007u32 as i32);
	pub const CO_INIT_TLS: Self = Self(0x8000_4006u32 as i32);
	pub const CO_INIT_TLS_CHANNEL_CONTROL: Self = Self(0x8000_400c_u32 as i32);
	pub const CO_INIT_TLS_SET_CHANNEL_CONTROL: Self = Self(0x8000_400b_u32 as i32);
	pub const CO_INIT_UNACCEPTED_USER_ALLOCATOR: Self = Self(0x8000_400d_u32 as i32);
	pub const CO_LAUNCH_PERMSSION_DENIED: Self = Self(0x8000_401b_u32 as i32);
	pub const CO_MALFORMED_SPN: Self = Self(0x8000_4033u32 as i32);
	pub const CO_MSI_ERROR: Self = Self(0x8000_4023u32 as i32);
	pub const CO_NOT_SUPPORTED: Self = Self(0x8000_4021u32 as i32);
	pub const CO_NO_SECCTX_IN_ACTIVATE: Self = Self(0x8000_402b_u32 as i32);
	pub const CO_OLE1DDE_DISABLED: Self = Self(0x8000_4016u32 as i32);
	pub const CO_PREMATURE_STUB_RUNDOWN: Self = Self(0x8000_4035u32 as i32);
	pub const CO_RELOAD_DLL: Self = Self(0x8000_4022u32 as i32);
	pub const CO_REMOTE_COMMUNICATION_FAILURE: Self = Self(0x8000_401d_u32 as i32);
	pub const CO_RUNAS_CREATEPROCESS_FAILURE: Self = Self(0x8000_4019u32 as i32);
	pub const CO_RUNAS_LOGON_FAILURE: Self = Self(0x8000_401a_u32 as i32);
	pub const CO_RUNAS_SYNTAX: Self = Self(0x8000_4017u32 as i32);
	pub const CO_SERVER_INIT_TIMEOUT: Self = Self(0x8000_402a_u32 as i32);
	pub const CO_SERVER_NOT_PAUSED: Self = Self(0x8000_4026u32 as i32);
	pub const CO_SERVER_PAUSED: Self = Self(0x8000_4025u32 as i32);
	pub const CO_SERVER_START_TIMEOUT: Self = Self(0x8000_401e_u32 as i32);
	pub const CO_START_SERVICE_FAILURE: Self = Self(0x8000_401c_u32 as i32);
	pub const CO_SXS_CONFIG: Self = Self(0x8000_4032u32 as i32);
	pub const CO_THREADPOOL_CONFIG: Self = Self(0x8000_4031u32 as i32);
	pub const CO_TRACKER_CONFIG: Self = Self(0x8000_4030u32 as i32);
	pub const CO_UNREVOKED_REGISTRATION_ON_APARTMENT_SHUTDOWN: Self = Self(0x8000_4034u32 as i32);
	pub const CO_WRONG_SERVER_IDENTITY: Self = Self(0x8000_4015u32 as i32);

	pub const CO_ACCESSCHECKFAILED: Self = Self(0x8001_012a_u32 as i32);
	pub const CO_ACESINWRONGORDER: Self = Self(0x8001_013a_u32 as i32);
	pub const CO_ACNOTINITIALIZED: Self = Self(0x8001_013f_u32 as i32);
	pub const CO_CANCEL_DISABLED: Self = Self(0x8001_0140u32 as i32);
	pub const CO_CONVERSIONFAILED: Self = Self(0x8001_012e_u32 as i32);
	pub const CO_DECODEFAILED: Self = Self(0x8001_013d_u32 as i32);
	pub const CO_EXCEEDSYSACLLIMIT: Self = Self(0x8001_0139u32 as i32);
	pub const CO_FAILEDTOCLOSEHANDLE: Self = Self(0x8001_0138u32 as i32);
	pub const CO_FAILEDTOCREATEFILE: Self = Self(0x8001_0137u32 as i32);
	pub const CO_FAILEDTOGENUUID: Self = Self(0x8001_0136u32 as i32);
	pub const CO_FAILEDTOGETSECCTX: Self = Self(0x8001_0124u32 as i32);
	pub const CO_FAILEDTOGETTOKENINFO: Self = Self(0x8001_0126u32 as i32);
	pub const CO_FAILEDTOGETWINDIR: Self = Self(0x8001_0134u32 as i32);
	pub const CO_FAILEDTOIMPERSONATE: Self = Self(0x8001_0123u32 as i32);
	pub const CO_FAILEDTOOPENPROCESSTOKEN: Self = Self(0x8001_013c_u32 as i32);
	pub const CO_FAILEDTOOPENTHREADTOKEN: Self = Self(0x8001_0125u32 as i32);
	pub const CO_FAILEDTOQUERYCLIENTBLANKET: Self = Self(0x8001_0128u32 as i32);
	pub const CO_FAILEDTOSETDACL: Self = Self(0x8001_0129u32 as i32);
	pub const CO_INCOMPATIBLESTREAMVERSION: Self = Self(0x8001_013b_u32 as i32);
	pub const CO_INVALIDSID: Self = Self(0x8001_012d_u32 as i32);
	pub const CO_LOOKUPACCNAMEFAILED: Self = Self(0x8001_0132u32 as i32);
	pub const CO_LOOKUPACCSIDFAILED: Self = Self(0x8001_0130u32 as i32);
	pub const CO_NETACCESSAPIFAILED: Self = Self(0x8001_012b_u32 as i32);
	pub const CO_NOMATCHINGNAMEFOUND: Self = Self(0x8001_0131u32 as i32);
	pub const CO_NOMATCHINGSIDFOUND: Self = Self(0x8001_012f_u32 as i32);
	pub const CO_PATHTOOLONG: Self = Self(0x8001_0135u32 as i32);
	pub const CO_SETSERLHNDLFAILED: Self = Self(0x8001_0133u32 as i32);
	pub const CO_TRUSTEEDOESNTMATCHCLIENT: Self = Self(0x8001_0127u32 as i32);
	pub const CO_WRONGTRUSTEENAMESYNTAX: Self = Self(0x8001_012c_u32 as i32);

	pub const CO_ACTIVATIONFAILED: Self = Self(0x8004_e021u32 as i32);
	pub const CO_ACTIVATIONFAILED_CATALOGERROR: Self = Self(0x8004_e023u32 as i32);
	pub const CO_ACTIVATIONFAILED_EVENTLOGGED: Self = Self(0x8004_e022u32 as i32);
	pub const CO_ACTIVATIONFAILED_TIMEOUT: Self = Self(0x8004_e024u32 as i32);
	pub const CO_ALREADYINITIALIZED: Self = Self(0x8004_01f1u32 as i32);
	pub const CO_APPDIDNTREG: Self = Self(0x8004_01fe_u32 as i32);
	pub const CO_APPNOTFOUND: Self = Self(0x8004_01f5u32 as i32);
	pub const CO_APPSINGLEUSE: Self = Self(0x8004_01f6u32 as i32);
	pub const CO_CALL_OUT_OF_TX_SCOPE_NOT_ALLOWED: Self = Self(0x8004_e030u32 as i32);
	pub const CO_CANTDETERMINECLASS: Self = Self(0x8004_01f2u32 as i32);
	pub const CO_CLASSSTRING: Self = Self(0x8004_01f3u32 as i32);
	pub const CO_DBERROR: Self = Self(0x8004_e02b_u32 as i32);
	pub const CO_DLLNOTFOUND: Self = Self(0x8004_01f8u32 as i32);
	pub const CO_ERRORINAPP: Self = Self(0x8004_01f7u32 as i32);
	pub const CO_ERRORINDLL: Self = Self(0x8004_01f9u32 as i32);
	pub const CO_EXIT_TRANSACTION_SCOPE_NOT_CALLED: Self = Self(0x8004_e031u32 as i32);
	pub const CO_IIDSTRING: Self = Self(0x8004_01f4u32 as i32);
	pub const CO_INITIALIZATIONFAILED: Self = Self(0x8004_e025u32 as i32);
	pub const CO_ISOLEVELMISMATCH: Self = Self(0x8004_e02f_u32 as i32);
	pub const CO_NOCOOKIES: Self = Self(0x8004_e02a_u32 as i32);
	pub const CO_NOIISINTRINSICS: Self = Self(0x8004_e029u32 as i32);
	pub const CO_NOSYNCHRONIZATION: Self = Self(0x8004_e02e_u32 as i32);
	pub const CO_NOTCONSTRUCTED: Self = Self(0x8004_e02d_u32 as i32);
	pub const CO_NOTINITIALIZED: Self = Self(0x8004_01f0u32 as i32);
	pub const CO_NOTPOOLED: Self = Self(0x8004_e02c_u32 as i32);
	pub const CO_OBJISREG: Self = Self(0x8004_01fc_u32 as i32);
	pub const CO_OBJNOTCONNECTED: Self = Self(0x8004_01fd_u32 as i32);
	pub const CO_OBJNOTREG: Self = Self(0x8004_01fb_u32 as i32);
	pub const CO_RELEASED: Self = Self(0x8004_01ff_u32 as i32);
	pub const CO_THREADINGMODEL_CHANGED: Self = Self(0x8004_e028u32 as i32);
	pub const CO_WRONGOSFORAPP: Self = Self(0x8004_01fa_u32 as i32);

	pub const CO_BAD_PATH: Self = Self(0x8008_0004u32 as i32);
	pub const CO_CLASS_CREATE_FAILED: Self = Self(0x8008_0001u32 as i32);
	pub const CO_ELEVATION_DISABLED: Self = Self(0x8008_0017u32 as i32);
	pub const CO_MISSING_DISPLAYNAME: Self = Self(0x8008_0015u32 as i32);
	pub const CO_OBJSRV_RPC_FAILURE: Self = Self(0x8008_0006u32 as i32);
	pub const CO_RUNAS_VALUE_MUST_BE_AAA: Self = Self(0x8008_0016u32 as i32);
	pub const CO_SCM_ERROR: Self = Self(0x8008_0002u32 as i32);
	pub const CO_SCM_RPC_FAILURE: Self = Self(0x8008_0003u32 as i32);
	pub const CO_SERVER_EXEC_FAILURE: Self = Self(0x8008_0005u32 as i32);
	pub const CO_SERVER_STOPPING: Self = Self(0x8008_0008u32 as i32);

	#[doc(alias = "CO_E_FIRST")]
	pub const CO_FIRST: Self = Self((Self::CO_S_FIRST.0 as u32 | 0x8000_0000) as i32);
	#[doc(alias = "CO_E_LAST")]
	pub const CO_LAST: Self = Self((Self::CO_S_LAST.0 as u32 | 0x8000_0000) as i32);
	pub const CO_S_FIRST: Self = Self(0x0004_01f0);
	pub const CO_S_LAST: Self = Self(0x0004_01ff);
}

impl fmt::Display for HRESULT {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:#010X}", self.0)
	}
}

windows_newtype! {
	pub struct Foundation::WIN32_ERROR(pub u32);
}

impl WIN32_ERROR {
	pub const fn to_hresult(self) -> HRESULT {
		HRESULT(-(self.0 as i32))
	}

	pub const fn from_hresult(res: HRESULT) -> Self {
		Self((-res.0) as u32)
	}

	pub const fn is_ok(self) -> bool {
		self.0 == 0
	}

	pub const fn is_err(self) -> bool {
		!self.is_ok()
	}

	pub const fn ok_code(self) -> Result<(), NonZeroI32> {
		unsafe {
			transmute(self.to_hresult().err_code())
		}
	}

	#[cfg(windows)]
	pub fn last_error() -> Self {
		match () {
			#[cfg(feature = "windows-060")]
			() => unsafe {
				Foundation060::GetLastError().into()
			},
			#[cfg(feature = "windows-061")]
			() => unsafe {
				Foundation061::GetLastError().into()
			},
			#[cfg(feature = "std")]
			() => io::Error::last_os_error().into(),
			_ => unimplemented!(),
		}
	}

	#[cfg(windows)]
	#[cfg(any(feature = "windows", feature = "windows-link"))]
	pub fn set_last_error(self) {
		crate::windows::Win32::Foundation::SetLastError(self)
	}
}

impl From<WIN32_ERROR> for HRESULT {
	fn from(e: WIN32_ERROR) -> Self {
		e.to_hresult()
	}
}

impl From<HRESULT> for WIN32_ERROR {
	fn from(e: HRESULT) -> Self {
		Self::from_hresult(e)
	}
}

impl From<WIN32_ERROR> for Error {
	fn from(e: WIN32_ERROR) -> Self {
		Self::with_hresult(e.to_hresult())
	}
}

impl From<Error> for WIN32_ERROR {
	fn from(e: Error) -> Self {
		Self::from_hresult(e.code())
	}
}

#[cfg(feature = "std")]
impl From<io::Error> for WIN32_ERROR {
	fn from(e: io::Error) -> Self {
		Self::from_hresult(HRESULT::from(e))
	}
}

#[cfg(feature = "std")]
impl From<WIN32_ERROR> for io::Error {
	fn from(e: WIN32_ERROR) -> Self {
		io::Error::from_raw_os_error(e.to_hresult().into())
	}
}

#[cfg(feature = "std")]
impl From<io::ErrorKind> for WIN32_ERROR {
	fn from(e: io::ErrorKind) -> Self {
		use crate::windows::Win32::Foundation;
		match e {
			io::ErrorKind::NotFound =>
				Foundation::ERROR_FILE_NOT_FOUND,
			io::ErrorKind::PermissionDenied =>
				Foundation::ERROR_ACCESS_DENIED,
			io::ErrorKind::BrokenPipe =>
				Foundation::ERROR_BROKEN_PIPE,
			io::ErrorKind::WouldBlock =>
				Foundation::ERROR_BUSY,
			io::ErrorKind::AddrInUse =>
				Foundation::ERROR_NETWORK_BUSY,
			io::ErrorKind::InvalidData =>
				Foundation::ERROR_INVALID_DATA,
			io::ErrorKind::InvalidInput =>
				Foundation::ERROR_BAD_ARGUMENTS,
			io::ErrorKind::UnexpectedEof =>
				Foundation::ERROR_HANDLE_EOF,
			io::ErrorKind::Unsupported =>
				Foundation::ERROR_NOT_SUPPORTED,
			_ =>
				Foundation::ERROR_INVALID_NAME,
		}
	}
}

#[cfg(feature = "windows-core-060")]
impl From<WIN32_ERROR> for core060::Error {
	fn from(e: WIN32_ERROR) -> Self {
		core060::Error::from_hresult(e.to_hresult().into())
	}
}
#[cfg(feature = "windows-core-061")]
impl From<WIN32_ERROR> for core061::Error {
	fn from(e: WIN32_ERROR) -> Self {
		core061::Error::from_hresult(e.to_hresult().into())
	}
}
#[cfg(feature = "windows-core-060")]
impl From<WIN32_ERROR> for Option<core060::Error> {
	fn from(e: WIN32_ERROR) -> Self {
		e.to_hresult().err_code()
			.map(|c| core060::Error::from_hresult(core060::HRESULT(c.get())))
	}
}
#[cfg(feature = "windows-core-061")]
impl From<WIN32_ERROR> for Option<core061::Error> {
	fn from(e: WIN32_ERROR) -> Self {
		e.to_hresult().err_code()
			.map(|c| core061::Error::from_hresult(core061::HRESULT(c.get())))
	}
}

windows_newtype! {
	pub struct core::PSTR(pub *mut c_char);
}

windows_newtype! {
	pub struct core::PWSTR(pub *mut c_wchar);
}

pub type PCSTR = Option<CStrPtr<'static>>;
pub type PCWSTR = Option<CStrPtr16<'static>>;

windows_newtype! {
	pub struct Foundation::GENERIC_ACCESS_RIGHTS(pub u32);
}

windows_newtype! {
	pub struct Foundation::LPARAM(pub isize);
}

windows_newtype! {
	pub struct Foundation::WPARAM(pub usize);
}

windows_newtype! {
	pub struct Foundation::HMODULE(pub *mut c_void);
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

windows_newtype! {
	pub struct Foundation::HWND(pub *mut c_void);
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

macro_rules! windows_newtype {
	(
		$vis:vis struct $module:ident :: $name:ident($($field_ty:tt)*);
	) => {
		#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
		#[repr(transparent)]
		$vis struct $name($($field_ty)*);

		impl Default for $name {
			#[inline]
			fn default() -> Self {
				Self(unsafe {
					::core::mem::MaybeUninit::zeroed().assume_init()
				})
			}
		}

		$crate::windows::adapter::windows_newtype! {
			impl From for $module :: $name($($field_ty)*);
		}
		$crate::windows::adapter::windows_newtype! {
			impl From@transparent for $name($($field_ty)*);
		}
	};
	(
		impl From@transparent for $name:ident(pub u32);
	) => {
		$crate::windows::adapter::windows_newtype! {
			impl From@transparent_imp for $name{pub u32};
		}
		$crate::windows::adapter::windows_newtype! {
			impl From@transparent for $name(pub Option<::core::num::NonZeroU32>);
		}

		impl From<::core::num::NonZeroU32> for $name {
			fn from(v: ::core::num::NonZeroU32) -> Self {
				u32::from(v).into()
			}
		}

		impl ::core::convert::TryFrom<$name> for ::core::num::NonZeroU32 {
			type Error = ::core::num::TryFromIntError;

			fn try_from(v: $name) -> ::core::result::Result<Self, Self::Error> {
				::core::convert::TryFrom::<u32>::try_from(v.into())
			}
		}

		impl ::core::fmt::LowerHex for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				::core::fmt::LowerHex::fmt(&self.0, f)
			}
		}
		impl ::core::fmt::UpperHex for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				::core::fmt::UpperHex::fmt(&self.0, f)
			}
		}
		impl ::core::fmt::Binary for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				::core::fmt::Binary::fmt(&self.0, f)
			}
		}
	};
	(
		impl From@transparent for $name:ident(pub i32);
	) => {
		$crate::windows::adapter::windows_newtype! {
			impl From@transparent_imp for $name{pub i32};
		}

		$crate::windows::adapter::windows_newtype! {
			impl From@transparent for $name(pub Option<::core::num::NonZeroI32>);
		}

		impl From<::core::num::NonZeroI32> for $name {
			fn from(v: ::core::num::NonZeroI32) -> Self {
				i32::from(v).into()
			}
		}

		impl ::core::convert::TryFrom<$name> for ::core::num::NonZeroI32 {
			type Error = ::core::num::TryFromIntError;

			fn try_from(v: $name) -> ::core::result::Result<Self, Self::Error> {
				::core::convert::TryFrom::<i32>::try_from(v.into())
			}
		}

		impl ::core::fmt::LowerHex for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				::core::fmt::LowerHex::fmt(&self.0, f)
			}
		}
		impl ::core::fmt::UpperHex for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				::core::fmt::UpperHex::fmt(&self.0, f)
			}
		}
		impl ::core::fmt::Binary for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				::core::fmt::Binary::fmt(&self.0, f)
			}
		}
	};
	(
		impl From@transparent for $name:ident(pub *mut $($ty:tt)*);
	) => {
		$crate::windows::adapter::windows_newtype! {
			impl From@transparent_imp for $name{pub *mut $($ty)*};
		}

		$crate::windows::adapter::windows_newtype! {
			impl From@transparent for $name(pub Option<::core::ptr::NonNull<$($ty)*>>);
		}

		impl From<::core::ptr::NonNull<$($ty)*>> for $name {
			fn from(p: ::core::ptr::NonNull<$($ty)*>) -> Self {
				p.as_ptr().into()
			}
		}

		impl ::core::convert::TryFrom<$name> for ::core::ptr::NonNull<$($ty)*> {
			type Error = ::core::num::TryFromIntError;

			fn try_from(v: $name) -> ::core::result::Result<Self, Self::Error> {
				<::core::num::NonZeroUsize as ::core::convert::TryFrom<usize>>::try_from(v.0 as usize)
					.map(|_| unsafe {
						::core::ptr::NonNull::new_unchecked(v.0)
					})
			}
		}

		impl ::core::fmt::Pointer for $name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				::core::fmt::Pointer::fmt(&self.0, f)
			}
		}
	};
	(
		impl From@transparent for $name:ident(pub $field_ty:ty);
	) => {
		$crate::windows::adapter::windows_newtype! {
			impl From@transparent_imp for $name{pub $field_ty};
		}
	};
	(
		impl From@repr for $name:ident(pub $field_ty:ty);
	) => {
		impl From<$field_ty> for $name {
			fn from(v: $field_ty) -> Self {
				Self(v)
			}
		}
		impl From<$name> for $field_ty {
			fn from(v: $name) -> Self {
				v.0
			}
		}
	};
	(
		impl From@transparent_imp for $name:ty{pub $field_ty:ty};
	) => {
		$crate::windows::adapter::windows_newtype! {
			impl From@transparent_imp for $name{$field_ty};
		}

		impl<'a> From<&'a mut $field_ty> for &'a mut $name {
			fn from(v: &'a mut $field_ty) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl<'a> From<&'a mut $name> for &'a mut $field_ty {
			fn from(v: &'a mut $name) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
	};
	(
		impl From@transparent_imp for $name:ty{$field_ty:ty};
	) => {
		impl From<$field_ty> for $name {
			fn from(v: $field_ty) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl From<$name> for $field_ty {
			fn from(v: $name) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl<'a> From<&'a $field_ty> for &'a $name {
			fn from(v: &'a $field_ty) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl<'a> From<&'a $name> for &'a $field_ty {
			fn from(v: &'a $name) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
	};
	(
		impl From for core::$name:ident($field_vis:vis $field_ty:ty);
	) => {
		#[cfg(feature = "windows-core-060")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::core060::$name} for $name($field_vis $field_ty);
		}
		#[cfg(feature = "windows-core-061")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::core061::$name} for $name($field_vis $field_ty);
		}
	};
	(
		impl From for Foundation::$name:ident($field_vis:vis $field_ty:ty);
	) => {
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_060::Foundation::$name} for $name($field_vis $field_ty);
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_061::Foundation::$name} for $name($field_vis $field_ty);
		}
	};
	(
		impl From for LibraryLoader::$name:ident($field_vis:vis $field_ty:ty);
	) => {
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_060::System::LibraryLoader::$name} for $name($field_vis $field_ty);
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_061::System::LibraryLoader::$name} for $name($field_vis $field_ty);
		}
	};
	(
		impl From for Com::$name:ident($field_vis:vis $field_ty:ty);
	) => {
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_060::System::Com::$name} for $name($field_vis $field_ty);
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_061::System::Com::$name} for $name($field_vis $field_ty);
		}
	};
	(
		impl From for Imaging::$name:ident($field_vis:vis $field_ty:ty);
	) => {
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_060::Graphics::Imaging::$name} for $name($field_vis $field_ty);
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_061::Graphics::Imaging::$name} for $name($field_vis $field_ty);
		}
	};
	(
		impl From for Dxgi_Common::$name:ident($field_vis:vis $field_ty:ty);
	) => {
		#[cfg(windows)]
		#[cfg(feature = "windows-060")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_060::Graphics::Dxgi::Common::$name} for $name($field_vis $field_ty);
		}
		#[cfg(windows)]
		#[cfg(feature = "windows-061")]
		$crate::windows::adapter::windows_newtype! {
			impl From@path{$crate::windows::Win32_061::Graphics::Dxgi::Common::$name} for $name($field_vis $field_ty);
		}
	};
	(
		impl From@path{$win:path} for $name:ident($field_vis:vis $field_ty:ty);
	) => {
		impl From<$win> for $name {
			fn from(v: $win) -> Self {
				//Self(v.0)
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl From<$name> for $win {
			fn from(v: $name) -> Self {
				//Self(v.0)
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl<'a> From<&'a $name> for &'a $win {
			fn from(v: &'a $name) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl<'a> From<&'a mut $name> for &'a mut $win {
			fn from(v: &'a mut $name) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl<'a> From<&'a $win> for &'a $name {
			fn from(v: &'a $win) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}
		impl<'a> From<&'a mut $win> for &'a mut $name {
			fn from(v: &'a mut $win) -> Self {
				unsafe {
					::core::mem::transmute(v)
				}
			}
		}

		impl AsRef<$win> for $name {
			fn as_ref(&self) -> &$win {
				Into::into(self)
			}
		}
		impl AsMut<$win> for $name {
			fn as_mut(&mut self) -> &mut $win {
				Into::into(self)
			}
		}
		impl AsRef<$name> for $win {
			fn as_ref(&self) -> &$name {
				Into::into(self)
			}
		}
		impl AsMut<$name> for $win {
			fn as_mut(&mut self) -> &mut $name {
				Into::into(self)
			}
		}
	};
}
pub(crate) use windows_newtype;

#[macro_export]
macro_rules! windows_adapter {
	($vis:vis mod core as $id:ident => $($it:item)*) => {
		$crate::windows::adapter::windows_adapter_core061! { $id =>
			$vis mod adapter_core061 {
				$($it)*
			}
		}
		$crate::windows::adapter::windows_adapter_core060! { $id =>
			$vis mod adapter_core060 {
				$($it)*
			}
		}
	};
	($vis:vis mod windows as $id:ident => $($it:item)*) => {
		$crate::windows::adapter::windows_adapter_windows061! { $id =>
			$vis mod adapter_windows061 {
				$($it)*
			}
		}
		$crate::windows::adapter::windows_adapter_windows060! { $id =>
			$vis mod adapter_windows060 {
				$($it)*
			}
		}
	};
	($vis:vis mod win32 as $id:ident => $($it:item)*) => {
		$crate::windows::adapter::windows_adapter! {
			$vis mod windows as adapter_windows0xx =>
				use adapter_windows0xx as $id;
				$($it)*
		}
	};
	(match core as $id:ident => $expr:expr $(, _ => $fallback:expr$(,)?)?) => {{
		'windows_adapter_core: loop {
			#![allow(unreachable_code, unused_imports)]

			{
				$crate::windows::adapter::windows_adapter_core061! { $id@tt =>
					break 'windows_adapter_core ($expr);
				}
			}
			{
				$crate::windows::adapter::windows_adapter_core060! { $id@tt =>
					break 'windows_adapter_core ($expr);
				}
			}
			break 'windows_adapter_core ($(
				$crate::windows::adapter::windows_adapter_core_not! { @exp =>
					$fallback
				}
			)?);
		}
	}};
	(match windows as $id:ident => $expr:expr $(, _ => $fallback:expr$(,)?)?) => {{
		'windows_adapter_windows: loop {
			#![allow(unreachable_code, unused_imports)]

			{
				$crate::windows::adapter::windows_adapter_windows061! { $id@tt =>
					break 'windows_adapter_windows ($expr);
				}
			}
			{
				$crate::windows::adapter::windows_adapter_windows060! { $id@tt =>
					break 'windows_adapter_windows ($expr);
				}
			}
			break 'windows_adapter_windows ($(
				$crate::windows::adapter::windows_adapter_windows_not! { @exp =>
					$fallback
				}
			)?);
		}
	}};
	(match win32 as $id:ident => $expr:expr $(, _ => $fallback:expr$(,)?)?) => {
		$crate::windows::adapter::windows_adapter! {
			match windows as adapter_windows0xx => {
				use adapter_windows0xx as $id;
				$expr
			}
			$(, _ => $fallback:expr)?
		}
	};
	(match self::core as $adapter:ident, $id:ident => $expr:expr $(, _ => $fallback:expr$(,)?)?) => {{
		'windows_adapter_core: loop {
			#![allow(unreachable_code, unused_imports)]

			{
				$crate::windows::adapter::windows_adapter_core061! { $id@tt =>
					use self::adapter_core061 as $adapter;
					break 'windows_adapter_core ($expr);
				}
			}
			{
				$crate::windows::adapter::windows_adapter_core060! { $id@tt =>
					use self::adapter_core060 as $adapter;
					break 'windows_adapter_core ($expr);
				}
			}
			break 'windows_adapter_core ($(
				$crate::windows::adapter::windows_adapter_core_not! { @exp =>
					$fallback
				}
			)?);
		}
	}};
	(match self::windows as $adapter:ident, $id:ident => $expr:expr $(, _ => $fallback:expr$(,)?)?) => {{
		'windows_adapter_windows: loop {
			#![allow(unreachable_code, unused_imports)]

			{
				$crate::windows::adapter::windows_adapter_windows061! { $id@tt =>
					use self::adapter_windows061 as $adapter;
					break 'windows_adapter_windows ($expr);
				}
			}
			{
				$crate::windows::adapter::windows_adapter_windows060! { $id@tt =>
					use self::adapter_windows060 as $adapter;
					break 'windows_adapter_windows ($expr);
				}
			}
			break 'windows_adapter_windows ($(
				$crate::windows::adapter::windows_adapter_windows_not! { @exp =>
					$fallback
				}
			)?);
		}
	}};
}
pub use windows_adapter;
#[cfg(feature = "windows-core-060")] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_core060 {
	($id:ident => $vis:vis mod $mod_:ident { $($tt:tt)* }) => {
		$vis mod $mod_ {
			$crate::windows::adapter::windows_adapter_core060! { $id@tt => $($tt)* }
		}
	};
	($id:ident@tt => $($tt:tt)*) => {
		use $crate::windows::core060 as $id;
		$($tt)*
	};
	(@tt => $($tt:tt)*) => {
		$($tt)*
	};
}
#[cfg(not(feature = "windows-core-060"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_core060 { ($($tt:tt)*) => {}; }
#[cfg(feature = "windows-core-061")] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_core061 {
	($id:ident => $vis:vis mod $mod_:ident { $($tt:tt)* }) => {
		$vis mod $mod_ {
			$crate::windows::adapter::windows_adapter_core061! { $id@tt => $($tt)* }
		}
	};
	($id:ident@tt => $($tt:tt)*) => {
		use $crate::windows::core061 as $id;
		$($tt)*
	};
	(@tt => $($tt:tt)*) => {
		$($tt)*
	};
}
#[cfg(not(feature = "windows-core-061"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_core061 { ($($tt:tt)*) => {}; }
#[cfg(not(feature = "windows-core"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_core_not {
	(@exp => $($tt:tt)*) => {
		$($tt)*
	};
	(@tt => $($tt:tt)*) => {
		$($tt)*
	};
}
#[cfg(feature = "windows-core")] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_core_not {
	(@exp => $($tt:tt)*) => {
		match () {
			#[cfg(debug_assertions)]
			() => unreachable!(),
			#[cfg(not(debug_assertions))]
			#[allow(unused_unsafe)]
			() => unsafe { ::core::hint::unreachable_unchecked() },
		}
	};
	($($tt:tt)*) => {};
}
#[cfg(all(windows, feature = "windows-060"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows060 {
	($id:ident => $vis:vis mod $mod_:ident { $($tt:tt)* }) => {
		$vis mod $mod_ {
			$crate::windows::adapter::windows_adapter_windows060! { $id@tt => $($tt)* }
		}
	};
	($id:ident@tt => $($tt:tt)*) => {
		use $crate::windows::windows060 as $id;
		$($tt)*
	};
	(@tt => $($tt:tt)*) => {
		$($tt)*
	};
}
#[cfg(any(not(windows), not(feature = "windows-060")))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows060 { ($($tt:tt)*) => {}; }
#[cfg(all(windows, feature = "windows-061"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows061 {
	($id:ident => $vis:vis mod $mod_:ident { $($tt:tt)* }) => {
		$vis mod $mod_ {
			$crate::windows::adapter::windows_adapter_windows061! { $id@tt => $($tt)* }
		}
	};
	($id:ident => $($tt:tt)*) => {
		use $crate::windows::windows061 as $id;
		$($tt)*
	};
	(@tt => $($tt:tt)*) => {
		$($tt)*
	};
}
#[cfg(any(not(windows), not(feature = "windows-061")))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows061 { ($($tt:tt)*) => {}; }
#[cfg(any(not(windows), not(feature = "windows")))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows_not {
	(@exp => $($tt:tt)*) => {
		$($tt)*
	};
	(@tt => $($tt:tt)*) => {
		$($tt)*
	};
}
#[cfg(all(windows, feature = "windows"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows_not {
	(@exp => $($tt:tt)*) => {
		match () {
			#[cfg(debug_assertions)]
			() => unreachable!(),
			#[cfg(not(debug_assertions))]
			#[allow(unused_unsafe)]
			() => unsafe { ::core::hint::unreachable_unchecked() },
		}
	};
	($($tt:tt)*) => {};
}
pub use {
	windows_adapter_core_not, windows_adapter_core060, windows_adapter_core061,
	windows_adapter_windows_not, windows_adapter_windows060, windows_adapter_windows061,
};
