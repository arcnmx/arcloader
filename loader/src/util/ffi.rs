use std::{
	borrow::{Borrow, Cow}, ffi::{c_char, c_schar, c_uchar, CStr, CString, OsStr, OsString}, marker::PhantomData, mem::transmute, ops::Deref, ptr::{self, NonNull}, sync::Arc
};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

pub use cstr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CStrPtr<'a> {
	ptr: NonNull<c_char>,
	_borrow: PhantomData<&'a [c_char]>
}

impl CStrPtr<'static> {
	pub const EMPTY: Self = unsafe {
		const EMPTY_ZERO: c_char = 0;
		Self::new(nonnull_ref(&EMPTY_ZERO))
	};
}

impl<'a> CStrPtr<'a> {
	pub const unsafe fn new(ptr: NonNull<c_char>) -> Self {
		Self {
			ptr,
			_borrow: PhantomData,
		}
	}

	pub const unsafe fn newu(ptr: NonNull<c_uchar>) -> Self {
		Self::new(ptr.cast())
	}

	pub const unsafe fn new_ref(ptr: &'a c_char) -> Self {
		Self::new(nonnull_ref(ptr))
	}

	pub const unsafe fn newu_ref(ptr: &'a c_uchar) -> Self {
		Self::newu(nonnull_ref(ptr))
	}

	pub const fn with_cstr(cstr: &'a CStr) -> Self {
		unsafe {
			Self::new(nonnull_ref_unchecked(cstr.as_ptr()))
		}
	}

	pub const unsafe fn from_ptr(ptr: *const c_char) -> Option<Self> {
		unsafe {
			transmute(ptr)
		}
	}

	pub fn as_ptr(self) -> *const c_char {
		self.ptr.as_ptr()
	}

	pub fn as_sptr(self) -> *const c_schar {
		self.as_ptr() as *const c_schar
	}

	pub fn as_uptr(self) -> *const c_uchar {
		self.as_ptr() as *const c_uchar
	}

	pub fn as_ptr_ref(self) -> &'a c_char {
		unsafe {
			transmute(self.ptr)
		}
	}

	pub fn ptr(self) -> NonNull<c_char> {
		self.ptr
	}

	pub fn ptr_ref(&self) -> &NonNull<c_char> {
		&self.ptr
	}

	pub fn to_c_str(self) -> &'a CStr {
		unsafe {
			CStr::from_ptr(self.as_ptr())
		}
	}

	pub fn to_bytes(self) -> &'a [u8] {
		self.to_c_str().to_bytes()
	}

	pub fn to_bytes_with_nul(self) -> &'a [u8] {
		self.to_c_str().to_bytes_with_nul()
	}

	#[cfg(unix)]
	pub fn as_os_str(self) -> &'a OsStr {
		OsStr::from_bytes(self.to_bytes())
	}

	pub fn to_os_str(self) -> Cow<'a, OsStr> {
		match () {
			#[cfg(unix)]
			() => self.as_os_str(),
			#[cfg(not(unix))]
			() => match self.to_c_str().to_string_lossy() {
				Cow::Borrowed(s) => Cow::Borrowed(OsStr::new(s)),
				Cow::Owned(s) => Cow::Owned(OsString::from(s)),
			},
		}
	}

	#[cfg(windows)]
	pub fn as_pcstr(self) -> windows_strings::PCSTR {
		windows_strings::PCSTR::from_raw(self.as_uptr())
	}

	#[cfg(windows)]
	pub fn to_hstring(self) -> windows_strings::HSTRING {
		(&*self.to_os_str()).into()
	}
}

impl Deref for CStrPtr<'_> {
	type Target = CStr;

	fn deref(&self) -> &Self::Target {
		self.to_c_str()
	}
}

impl AsRef<CStr> for CStrPtr<'_> {
	fn as_ref(&self) -> &CStr {
		self.to_c_str()
	}
}

impl AsRef<[u8]> for CStrPtr<'_> {
	fn as_ref(&self) -> &[u8] {
		self.to_c_str().to_bytes()
	}
}

impl Borrow<CStr> for CStrPtr<'_> {
	fn borrow(&self) -> &CStr {
		self.to_c_str()
	}
}

impl<'a> From<&'a CStr> for CStrPtr<'a> {
	fn from(cstr: &'a CStr) -> Self {
		Self::with_cstr(cstr)
	}
}

impl<'a> From<&'a CString> for CStrPtr<'a> {
	fn from(cstr: &'a CString) -> Self {
		Self::with_cstr(cstr.as_c_str())
	}
}

impl<'a> From<CStrPtr<'a>> for &'a CStr {
	fn from(cstr: CStrPtr<'a>) -> Self {
		cstr.to_c_str()
	}
}

impl<'a> From<CStrPtr<'a>> for Cow<'a, CStr> {
	fn from(cstr: CStrPtr<'a>) -> Self {
		Cow::Borrowed(cstr.to_c_str())
	}
}

impl From<CStrPtr<'_>> for CString {
	fn from(cstr: CStrPtr) -> Self {
		cstr.to_c_str().to_owned()
	}
}

#[cfg(windows)]
impl From<CStrPtr<'_>> for windows_strings::PCSTR {
	fn from(cstr: CStrPtr) -> Self {
		cstr.as_pcstr()
	}
}

#[cfg(windows)]
impl From<CStrPtr<'_>> for windows_strings::HSTRING {
	fn from(cstr: CStrPtr) -> Self {
		cstr.to_hstring()
	}
}

#[cfg(windows)]
impl windows::core::Param<windows_strings::PCSTR> for CStrPtr<'_> {
	unsafe fn param(self) -> windows::core::ParamValue<windows_strings::PCSTR> {
		self.as_pcstr().param()
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct CStrRef(pub CStr);

impl CStrRef {
	#[inline]
	pub fn new(cstr: &CStr) -> &Self {
		unsafe {
			transmute(cstr)
		}
	}
}

impl Borrow<CStr> for CStrRef {
	fn borrow(&self) -> &CStr {
		&self.0
	}
}

impl Borrow<CStrRef> for Arc<CStr> {
	fn borrow(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

impl Borrow<CStrRef> for CStrPtr<'_> {
	fn borrow(&self) -> &CStrRef {
		CStrRef::new(self.to_c_str())
	}
}

pub unsafe fn cstr_opt<'a>(s: &'a *const c_char) -> Option<&'a CStr> {
	NonNull::new(*s as *mut c_char)
		.map(|p| CStr::from_ptr(p.as_ptr() as *const c_char))
}

pub fn cstr_write(dst: &mut [c_char], src: &CStr) -> usize {
	let len = dst.len().min(src.to_bytes().len());
	unsafe {
		ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), len);
	}
	len
}

pub const fn nonnull_const<P: ?Sized>(p: *const P) -> Option<NonNull<P>> {
	match p {
		#[cfg(feature = "unstable")]
		p => NonNull::new(p as *mut P),
		#[cfg(not(feature = "unstable"))]
		p => unsafe {
			transmute(p)
		},
	}
}

pub const fn nonnull_ref<P: ?Sized>(p: &P) -> NonNull<P> {
	unsafe {
		nonnull_ref_unchecked(p)
	}
}

pub const unsafe fn nonnull_ref_unchecked<P: ?Sized>(p: *const P) -> NonNull<P> {
	unsafe {
		NonNull::new_unchecked(p as *mut P)
	}
}

pub fn nonnull_ref_bytes<P: ?Sized>(p: &P) -> NonNull<[u8]> {
	let size = size_of_val(p);
	let ptr = ptr::slice_from_raw_parts_mut(p as *const P as *const u8 as *mut u8, size);
	unsafe {
		nonnull_ref_unchecked(ptr)
	}
}

pub fn nonnull_bytes<P>(p: NonNull<P>) -> NonNull<[u8]> {
	let size = size_of::<P>();
	let ptr = ptr::slice_from_raw_parts_mut(p.cast::<u8>().as_ptr(), size);
	unsafe {
		nonnull_ref_unchecked(ptr)
	}
}

pub fn nonnull_ptr<P: ?Sized, NN>(p: NN) -> *const P where
	NN: Into<Option<NonNull<P>>>,
{
	let p = p.into();
	unsafe {
		transmute(p)
	}
}

pub fn nonnull_ptr_mut<P: ?Sized, NN>(p: NN) -> *mut P where
	NN: Into<Option<NonNull<P>>>,
{
	let p = p.into();
	unsafe {
		transmute(p)
	}
}
