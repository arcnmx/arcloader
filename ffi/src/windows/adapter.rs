#![allow(unreachable_patterns)]

use core::{fmt, mem::transmute, num::NonZeroI32, ops};
#[cfg(feature = "std")]
use std::{borrow::Cow, error::Error as StdError, io};
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
	pub const S_EMPTY_ERROR: NonZeroI32 = unsafe { NonZeroI32::new_unchecked(i32::from_be_bytes(*b"S_OK")) };

	pub const fn empty() -> Self {
		Self {
			code: Self::S_EMPTY_ERROR,
			info: None,
		}
	}

	pub const fn from_parts(code: HRESULT, info: Option<ErrorInfo>) -> Self {
		Self {
			code: match code.err_code() {
				Some(c) => c,
				None => Self::S_EMPTY_ERROR,
			},
			info,
		}
	}

	pub const fn with_hresult(code: HRESULT) -> Self {
		Self::from_parts(code, None)
	}

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
	pub fn message(&self) -> Cow<str> {
		match &self.info {
			Some(msg) => Cow::Borrowed(msg),
			None => self.code().message(),
		}
	}

	#[cfg(windows)]
	pub fn from_win32() -> Self {
		Self::from_hresult(HRESULT::last_error())
	}
}

#[cfg(feature = "windows-core-060")]
impl From<Error> for core060::Error {
	fn from(e: Error) -> Self {
		match &e.info {
			None => core060::Error::from_hresult(e.code().into()),
			#[cfg(feature = "std")]
			Some(info) =>
				core060::Error::new(e.code().into(), info),
			#[cfg(not(feature = "std"))]
			&Some(info) => match info {},
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
		match &e.info {
			None => core061::Error::from_hresult(e.code().into()),
			#[cfg(feature = "std")]
			Some(info) =>
				core061::Error::new(e.code().into(), info),
			#[cfg(not(feature = "std"))]
			&Some(info) => match info {},
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
		if let Some(msg) = &self.info {
			write!(f, ": {}", msg)?;
		} else if let Some(msg) = self.code().try_message() {
			write!(f, ": {}", msg)?;
		}

		Ok(())
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut f = f.debug_struct("windows::Error");

		f.field("code", &format_args!("{:#08X}", self.code()));
		f.field("info", &self.info);
		#[cfg(feature = "std")]
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

	#[cfg(feature = "std")]
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

	#[cfg(feature = "std")]
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
}

impl From<WIN32_ERROR> for HRESULT {
	fn from(e: WIN32_ERROR) -> Self {
		e.to_hresult()
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
			break 'windows_adapter_core ($($fallback)?);
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
			break 'windows_adapter_windows ($($fallback)?);
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
			break 'windows_adapter_core ($($fallback)?);
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
			break 'windows_adapter_windows ($($fallback)?);
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
#[cfg(feature = "windows-060")] #[doc(hidden)] #[macro_export]
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
#[cfg(not(feature = "windows-060"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows060 { ($($tt:tt)*) => {}; }
#[cfg(feature = "windows-061")] #[doc(hidden)] #[macro_export]
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
#[cfg(not(feature = "windows-061"))] #[doc(hidden)] #[macro_export]
macro_rules! windows_adapter_windows061 { ($($tt:tt)*) => {}; }
pub use {
	windows_adapter_core060, windows_adapter_core061,
	windows_adapter_windows060, windows_adapter_windows061,
};
