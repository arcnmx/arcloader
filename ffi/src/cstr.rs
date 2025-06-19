use core::{
	borrow::Borrow,
	char::{decode_utf16, DecodeUtf16},
	cmp,
	fmt,
	hash,
	iter,
	marker::PhantomData,
	mem::{transmute, ManuallyDrop},
	ops::Deref,
	ptr::{self, NonNull},
	slice::{self, from_raw_parts},
	str::Utf8Chunks,
};
#[cfg(feature = "alloc")]
use crate::alloc::{
	borrow::Cow,
	rc::Rc,
	sync::Arc,
};
#[cfg(feature = "std")]
use std::ffi::{OsStr, OsString};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;
use crate::{nonnull_ref, nonnull_ref_unchecked};
use crate::wide::WideUtf8Reader;

#[allow(non_camel_case_types)]
pub type c_wchar = u16;
pub use crate::{c_char, c_uchar, c_schar};

pub use core::ffi::CStr;
#[cfg(feature = "alloc")]
pub use crate::alloc::ffi::CString;

pub const EMPTY_CSTR: &'static CStr = unsafe {
	CStr::from_bytes_with_nul_unchecked(&[0u8])
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

	const fn cstr_ptr_ref<'p>(cstr: &'p &'a CStr) -> &'p NonNull<c_char> {
		nonnull_ref_cast::<c_char, _>(nonnull_ref_ref(cstr))
	}

	const fn cstr_ref_ref<'p>(cstr: &'p &'a CStr) -> &'p &'a c_char {
		unsafe {
			transmute(Self::cstr_ptr_ref(cstr))
		}
	}

	pub const fn with_cstr_ref<'p>(cstr: &'p &'a CStr) -> &'p Self {
		unsafe {
			Self::new_ref_ref(Self::cstr_ref_ref(cstr))
		}
	}

	pub const unsafe fn new_ref_ref<'p>(ptr: &'p &'a c_char) -> &'p Self {
		unsafe {
			transmute(ptr)
		}
	}

	pub const unsafe fn from_ptr(ptr: *const c_char) -> Option<Self> {
		unsafe {
			transmute(ptr)
		}
	}

	pub const fn to_ptr(ptr: Option<Self>) -> *const c_char {
		unsafe {
			transmute(ptr)
		}
	}

	pub const unsafe fn immortal<'p>(self) -> CStrPtr<'p> {
		transmute(self)
	}

	pub const fn as_ptr(self) -> *const c_char {
		self.ptr.as_ptr()
	}

	pub const fn as_sptr(self) -> *const c_schar {
		self.as_ptr() as *const c_schar
	}

	pub const fn as_uptr(self) -> *const c_uchar {
		self.as_ptr() as *const c_uchar
	}

	pub const fn as_ptr_ref(self) -> &'a c_char {
		unsafe {
			transmute(self.ptr)
		}
	}

	pub const fn ptr(self) -> NonNull<c_char> {
		self.ptr
	}

	pub const fn ptr_ref(&self) -> &NonNull<c_char> {
		&self.ptr
	}

	#[cfg(feature = "alloc")]
	pub const fn as_c_box(&self) -> &CStrBox {
		unsafe {
			transmute(self)
		}
	}

	pub const fn ptr_opt(ptr: Option<Self>) -> Option<NonNull<c_char>> {
		unsafe {
			transmute(ptr)
		}
	}

	pub const fn ptr_ref_opt(ptr: &Option<Self>) -> &Option<NonNull<c_char>> {
		unsafe {
			transmute(ptr)
		}
	}

	pub const fn as_c_ref(self) -> &'a CStrRef {
		CStrRef::with_c_ptr(self)
	}

	pub fn to_c_str(self) -> &'a CStr {
		unsafe {
			CStr::from_ptr(self.as_ptr())
		}
	}

	pub fn to_c_slice(self) -> &'a CSlice {
		CSlice::new(self.to_c_str())
	}

	pub fn to_bytes(self) -> &'a [u8] {
		self.to_c_str().to_bytes()
	}

	pub fn to_bytes_with_nul(self) -> &'a [u8] {
		self.to_c_str().to_bytes_with_nul()
	}

	#[cfg(unix)]
	#[cfg(feature = "std")]
	pub fn as_os_str(self) -> &'a OsStr {
		OsStr::from_bytes(self.to_bytes())
	}

	#[cfg(feature = "std")]
	pub fn to_os_str(self) -> Cow<'a, OsStr> {
		#[cfg(not(unix))]
		use std::ffi::OsString;

		match () {
			#[cfg(unix)]
			() => Cow::Borrowed(self.as_os_str()),
			#[cfg(not(unix))]
			() => match self.to_c_str().to_string_lossy() {
				Cow::Borrowed(s) => Cow::Borrowed(OsStr::new(s)),
				Cow::Owned(s) => Cow::Owned(OsString::from(s)),
			},
		}
	}

	pub fn utf8(self) -> Utf8Chunks<'a> {
		self.to_bytes().utf8_chunks()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		unsafe { *self.ptr.as_ptr() == 0 }
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

impl<'a> AsRef<CStrPtr<'a>> for &'a CStr {
	fn as_ref(&self) -> &CStrPtr<'a> {
		CStrPtr::with_cstr_ref(self)
	}
}

impl<'a> Borrow<CStrPtr<'a>> for &'a CStr {
	fn borrow(&self) -> &CStrPtr<'a> {
		self.as_ref()
	}
}

impl<'a> Borrow<CStrPtr<'a>> for &'a Box<CStr> {
	fn borrow<'s>(&'s self) -> &'s CStrPtr<'a> {
		let cstr: &'s &'a CStr = unsafe { transmute(self) };
		Borrow::borrow(cstr)
	}
}

impl<'a> From<&'a CStr> for CStrPtr<'a> {
	fn from(cstr: &'a CStr) -> Self {
		Self::with_cstr(cstr)
	}
}

impl<'a> From<&'a CStrRef> for CStrPtr<'a> {
	fn from(cstr: &'a CStrRef) -> Self {
		cstr.as_c_ptr()
	}
}

impl<'a> From<&'a CSlice> for CStrPtr<'a> {
	fn from(cstr: &'a CSlice) -> Self {
		Self::with_cstr(cstr)
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a CStrBox> for CStrPtr<'a> {
	fn from(cstr: &'a CStrBox) -> Self {
		cstr.as_c_ptr()
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a CString> for CStrPtr<'a> {
	fn from(cstr: &'a CString) -> Self {
		Self::with_cstr(cstr.as_c_str())
	}
}

impl<'a, T: ?Sized + AsRef<CStr>> From<&'a Box<T>> for CStrPtr<'a> {
	fn from(cstr: &'a Box<T>) -> Self {
		let cstr: &T = &*cstr;
		Self::with_cstr(cstr.as_ref())
	}
}

#[cfg(feature = "alloc")]
impl<'a, T: ?Sized + AsRef<CStr>> From<&'a Arc<T>> for CStrPtr<'a> {
	fn from(cstr: &'a Arc<T>) -> Self {
		let cstr: &T = &*cstr;
		Self::with_cstr(cstr.as_ref())
	}
}

#[cfg(feature = "alloc")]
impl<'a, T: ?Sized + AsRef<CStr>> From<&'a Rc<T>> for CStrPtr<'a> {
	fn from(cstr: &'a Rc<T>) -> Self {
		let cstr: &T = &*cstr;
		Self::with_cstr(cstr.as_ref())
	}
}

impl<'a, T: ?Sized> From<&'a mut core::cell::RefCell<T>> for CStrPtr<'a> where
	&'a T: Into<Self>,
{
	fn from(cstr: &'a mut core::cell::RefCell<T>) -> Self {
		let cstr: &T = &*cstr.get_mut();
		cstr.into()
	}
}

#[cfg(todo)]
impl<'a, T: ?Sized> From<&'a mut std::sync::Mutex<T>> for CStrPtr<'a> where
	&'a T: Into<Self>,
{
	fn from(cstr: &'a mut std::sync::Mutex<T>) -> Self {
		let cstr: &T = &*cstr.get_mut().unwrap();
		cstr.into()
	}
}

#[cfg(todo)]
impl<'a, T: ?Sized> From<&'a mut std::sync::RwLock<T>> for CStrPtr<'a> where
	&'a T: Into<Self>,
{
	fn from(cstr: &'a mut std::sync::RwLock<T>) -> Self {
		let cstr: &T = &*cstr.get_mut().unwrap();
		cstr.into()
	}
}

impl<'a> From<CStrPtr<'a>> for &'a CStr {
	fn from(cstr: CStrPtr<'a>) -> Self {
		cstr.to_c_str()
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<CStrPtr<'a>> for Cow<'a, CStr> {
	fn from(cstr: CStrPtr<'a>) -> Self {
		Cow::Borrowed(cstr.to_c_str())
	}
}

impl<'a> From<CStrPtr<'a>> for &'a CSlice {
	fn from(cstr: CStrPtr<'a>) -> Self {
		cstr.to_c_slice()
	}
}

impl<'a> From<CStrPtr<'a>> for &'a CStrRef {
	fn from(cstr: CStrPtr<'a>) -> Self {
		cstr.as_c_ref()
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<CStrPtr<'a>> for Cow<'a, CStrRef> {
	fn from(cstr: CStrPtr<'a>) -> Self {
		Cow::Borrowed(cstr.as_c_ref())
	}
}

#[cfg(feature = "alloc")]
impl From<CStrPtr<'_>> for CString {
	fn from(cstr: CStrPtr) -> Self {
		cstr.to_c_str().to_owned()
	}
}

impl From<CStrPtr<'_>> for Box<CStr> {
	fn from(cstr: CStrPtr) -> Self {
		cstr.to_c_str().into()
	}
}

#[cfg(feature = "alloc")]
impl From<CStrPtr<'_>> for Arc<CStr> {
	fn from(cstr: CStrPtr) -> Self {
		cstr.to_c_str().into()
	}
}

impl From<CStrPtr<'_>> for *const c_char {
	fn from(cstr: CStrPtr) -> Self {
		cstr.as_ptr()
	}
}

impl From<CStrPtr<'_>> for NonNull<c_char> {
	fn from(cstr: CStrPtr) -> Self {
		cstr.ptr()
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-03")]
impl From<CStrPtr<'_>> for windows_strings_03::HSTRING {
	fn from(cstr: CStrPtr) -> Self {
		(&*cstr.to_os_str()).into()
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-04")]
impl From<CStrPtr<'_>> for windows_strings_04::HSTRING {
	fn from(cstr: CStrPtr) -> Self {
		(&*cstr.to_os_str()).into()
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-03")]
impl From<CStrPtr<'_>> for windows_strings_03::PCSTR {
	fn from(cstr: CStrPtr) -> Self {
		Self::from_raw(cstr.as_uptr())
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-04")]
impl From<CStrPtr<'_>> for windows_strings_04::PCSTR {
	fn from(cstr: CStrPtr) -> Self {
		Self::from_raw(cstr.as_uptr())
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-core-060")]
impl windows_core_060::Param<windows_core_060::PCSTR> for CStrPtr<'_> {
	unsafe fn param(self) -> windows_core_060::ParamValue<windows_core_060::PCSTR> {
		windows_core_060::PCSTR::from(self).param()
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-core-061")]
impl windows_core_061::Param<windows_core_061::PCSTR> for CStrPtr<'_> {
	unsafe fn param(self) -> windows_core_061::ParamValue<windows_core_061::PCSTR> {
		windows_core_061::PCSTR::from(self).param()
	}
}

unsafe impl Send for CStrPtr<'_> {}
unsafe impl Sync for CStrPtr<'_> {}

impl fmt::Debug for CStrPtr<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("CStrPtr")
			.field(&self.ptr)
			.field(&self.to_c_str())
			.finish()
	}
}

impl fmt::Display for CStrPtr<'_> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self.as_c_ref(), f)
	}
}

#[repr(transparent)]
#[cfg(feature = "alloc")]
pub struct CStrBox {
	ptr: CStrPtr<'static>,
}

#[cfg(feature = "alloc")]
impl CStrBox {
	#[inline]
	pub fn new<T: Into<CString>>(cstr: T) -> Self {
		Self::with_cstring(cstr.into())
	}

	#[inline]
	pub const unsafe fn new_unchecked(ptr: CStrPtr<'static>) -> Self {
		Self {
			ptr,
		}
	}

	#[inline]
	pub fn with_cstring(cstr: CString) -> Self {
		unsafe {
			let ptr = CStrPtr::new(NonNull::new_unchecked(cstr.into_raw()));
			Self::new_unchecked(ptr)
		}
	}

	#[inline]
	pub fn into_ptr(self) -> CStrPtr<'static> {
		ManuallyDrop::new(self).ptr
	}

	#[inline]
	pub const fn as_c_ptr<'a>(&'a self) -> CStrPtr<'a> {
		self.ptr
	}

	pub const fn as_c_ref(&self) -> &CStrRef {
		CStrRef::with_c_ptr(self.ptr)
	}

	#[inline]
	pub fn ptr_ref<'p>(&'p self) -> &'p CStrPtr<'p> {
		&self.ptr
	}

	#[inline]
	pub unsafe fn ptr_mut(&mut self) -> &mut CStrPtr<'static> {
		&mut self.ptr
	}

	#[inline]
	pub fn into_cstring(self) -> CString {
		let cstr = self.into_ptr();
		unsafe {
			CString::from_raw(cstr.as_ptr() as *mut _)
		}
	}
}

#[cfg(feature = "alloc")]
impl Clone for CStrBox {
	fn clone(&self) -> Self {
		Self::with_cstring(self.to_c_str().to_owned())
	}
}

#[cfg(feature = "alloc")]
impl Drop for CStrBox {
	fn drop(&mut self) {
		drop(unsafe {
			CString::from_raw(self.ptr.as_ptr() as *mut _)
		});
	}
}

#[cfg(feature = "alloc")]
impl Deref for CStrBox {
	type Target = CStrRef;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.as_c_ref()
	}
}

#[cfg(feature = "alloc")]
impl PartialOrd for CStrBox {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.as_c_ref().partial_cmp(other.as_c_ref())
	}
}

#[cfg(feature = "alloc")]
impl Ord for CStrBox {
	#[inline]
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.as_c_ref().cmp(other.as_c_ref())
	}
}

#[cfg(feature = "alloc")]
impl PartialEq for CStrBox {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.as_c_ref().eq(other.as_c_ref())
	}
}

#[cfg(feature = "alloc")]
impl Eq for CStrBox {}

#[cfg(feature = "alloc")]
impl hash::Hash for CStrBox {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.as_c_ref().hash(state)
	}
}

#[cfg(feature = "alloc")]
impl fmt::Debug for CStrBox {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self.as_c_ref(), f)
	}
}

#[cfg(feature = "alloc")]
impl fmt::Display for CStrBox {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self.as_c_ref(), f)
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CSlice(pub CStr);

impl CSlice {
	#[inline]
	pub fn new<T: ?Sized + AsRef<CStr>>(cstr: &T) -> &Self {
		Self::with_cstr(cstr.as_ref())
	}

	#[inline]
	pub const fn with_cstr(cstr: &CStr) -> &Self {
		unsafe {
			transmute(cstr)
		}
	}
}

impl Borrow<CStr> for CSlice {
	#[inline]
	fn borrow(&self) -> &CStr {
		&self.0
	}
}

impl Borrow<CSlice> for CStr {
	#[inline]
	fn borrow(&self) -> &CSlice {
		CSlice::new(self)
	}
}

impl Borrow<CSlice> for CString {
	#[inline]
	fn borrow(&self) -> &CSlice {
		CSlice::new(self)
	}
}

impl Borrow<CSlice> for Box<CStr> {
	#[inline]
	fn borrow(&self) -> &CSlice {
		CSlice::new(self)
	}
}

#[cfg(feature = "alloc")]
impl Borrow<CSlice> for Rc<CStr> {
	#[inline]
	fn borrow(&self) -> &CSlice {
		CSlice::new(self)
	}
}

#[cfg(feature = "alloc")]
impl Borrow<CSlice> for Arc<CStr> {
	#[inline]
	fn borrow(&self) -> &CSlice {
		CSlice::new(self)
	}
}

impl Borrow<CSlice> for CStrPtr<'_> {
	#[inline]
	fn borrow(&self) -> &CSlice {
		CSlice::new(self.to_c_str())
	}
}

#[cfg(feature = "alloc")]
impl Borrow<CSlice> for CStrBox {
	#[inline]
	fn borrow(&self) -> &CSlice {
		self.to_c_str().borrow()
	}
}

impl Borrow<CSlice> for CStrRef {
	#[inline]
	fn borrow(&self) -> &CSlice {
		self.to_c_str().borrow()
	}
}

impl AsRef<CStr> for CSlice {
	#[inline]
	fn as_ref(&self) -> &CStr {
		&self.0
	}
}

impl AsRef<CSlice> for CStr {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		CSlice::new(self)
	}
}

impl AsRef<CSlice> for CStrRef {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		self.to_c_str().as_ref()
	}
}

impl AsRef<CSlice> for CString {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		CSlice::new(self)
	}
}

#[cfg(feature = "alloc")]
impl AsRef<CSlice> for CStrBox {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		self.to_c_str().as_ref()
	}
}

impl AsRef<CSlice> for Box<CStr> {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		CSlice::new(self)
	}
}

#[cfg(feature = "alloc")]
impl AsRef<CSlice> for Rc<CStr> {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		CSlice::new(self)
	}
}

#[cfg(feature = "alloc")]
impl AsRef<CSlice> for Arc<CStr> {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		CSlice::new(self)
	}
}

impl AsRef<CSlice> for CStrPtr<'_> {
	#[inline]
	fn as_ref(&self) -> &CSlice {
		CSlice::new(self.to_c_str())
	}
}

impl Deref for CSlice {
	type Target = CStr;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[repr(transparent)]
pub struct CStrRef {
	_cstr: PhantomData<[c_char]>,
	start: c_char,
}

impl CStrRef {
	pub const EMPTY: &'static Self = unsafe {
		Self::new_ref_unchecked(&0)
	};

	#[inline]
	pub fn new<'p, T: Into<CStrPtr<'p>>>(cstr: T) -> &'p Self {
		let cstr = cstr.into();
		Self::with_c_ptr(cstr.into())
	}

	#[inline]
	pub const unsafe fn new_ref_unchecked(cstr: &c_char) -> &Self {
		transmute(cstr)
	}

	#[inline]
	pub const unsafe fn from_ptr<'p>(cstr: *const c_char) -> &'p Self {
		unsafe {
			transmute(cstr)
		}
	}

	#[inline]
	pub const unsafe fn with_ptr<'p>(cstr: *const Self) -> &'p Self {
		Self::from_ptr(cstr as *const c_char)
	}

	#[inline]
	pub const fn with_cstr(cstr: &CStr) -> &Self {
		unsafe {
			Self::from_ptr(cstr.as_ptr())
		}
	}

	#[inline]
	pub const fn with_c_ptr<'a>(cstr: CStrPtr<'a>) -> &'a Self {
		unsafe {
			transmute(cstr)
		}
	}

	#[inline]
	pub const fn as_ptr(&self) -> *const c_char {
		unsafe {
			transmute(self)
		}
	}

	#[inline]
	pub const fn as_c_ptr(&self) -> CStrPtr<'_> {
		unsafe {
			transmute(self)
		}
	}

	#[inline]
	pub fn to_c_str(&self) -> &CStr {
		self.as_c_ptr().to_c_str()
	}

	#[inline]
	pub const fn c_ptr_ref<'r, 'p>(cstr: &'r &'p Self) -> &'r CStrPtr<'p> {
		unsafe {
			transmute(cstr)
		}
	}

	pub fn utf8(&self) -> Utf8Chunks<'_> {
		self.as_c_ptr().utf8()
	}

	#[inline]
	pub const fn is_empty(&self) -> bool {
		self.start == 0
	}
}

impl Default for &'_ CStrRef {
	fn default() -> Self {
		CStrRef::EMPTY
	}
}

impl fmt::Debug for CStrRef {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("CStrRef")
			.field(&self.to_c_str())
			//.field(&format_args!("\"{}\"", self))
			.finish()
	}
}

impl fmt::Display for CStrRef {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for chunk in self.utf8() {
			f.write_str(chunk.valid())?;
			match chunk.invalid() {
				invalid if invalid.is_empty() => (),
				invalid => for _ in invalid {
					fmt::Write::write_char(f, char::REPLACEMENT_CHARACTER)?;
				},
			}
		}
		Ok(())
	}
}

impl PartialOrd for CStrRef {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.to_c_str().partial_cmp(other.to_c_str())
	}
}

impl Ord for CStrRef {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.to_c_str().cmp(other.to_c_str())
	}
}

impl PartialEq for CStrRef {
	fn eq(&self, other: &Self) -> bool {
		self.to_c_str().eq(other.to_c_str())
	}
}

impl Eq for CStrRef {}

impl hash::Hash for CStrRef {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.to_c_str().hash(state)
	}
}

impl ToOwned for CStrRef {
	type Owned = CString;

	fn to_owned(&self) -> Self::Owned {
		self.to_c_str().to_owned()
	}
}

impl Borrow<CStr> for CStrRef {
	#[inline]
	fn borrow(&self) -> &CStr {
		self.to_c_str()
	}
}

impl Borrow<CStrRef> for CStr {
	#[inline]
	fn borrow(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

impl Borrow<CStrRef> for CString {
	#[inline]
	fn borrow(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

impl Borrow<CStrRef> for Box<CStr> {
	#[inline]
	fn borrow(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

#[cfg(feature = "alloc")]
impl Borrow<CStrRef> for Rc<CStr> {
	#[inline]
	fn borrow(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

#[cfg(feature = "alloc")]
impl Borrow<CStrRef> for Arc<CStr> {
	#[inline]
	fn borrow(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

impl Borrow<CStrRef> for CStrPtr<'_> {
	#[inline]
	fn borrow(&self) -> &CStrRef {
		self.as_c_ref()
	}
}

impl AsRef<CStr> for CStrRef {
	#[inline]
	fn as_ref(&self) -> &CStr {
		self.to_c_str()
	}
}

impl AsRef<CStrRef> for CStr {
	#[inline]
	fn as_ref(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

impl AsRef<CStrRef> for CString {
	#[inline]
	fn as_ref(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

impl AsRef<CStrRef> for Box<CStr> {
	#[inline]
	fn as_ref(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

#[cfg(feature = "alloc")]
impl AsRef<CStrRef> for Rc<CStr> {
	#[inline]
	fn as_ref(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

#[cfg(feature = "alloc")]
impl AsRef<CStrRef> for Arc<CStr> {
	#[inline]
	fn as_ref(&self) -> &CStrRef {
		CStrRef::new(self)
	}
}

impl AsRef<CStrRef> for CStrPtr<'_> {
	#[inline]
	fn as_ref(&self) -> &CStrRef {
		self.as_c_ref()
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CStrPtr16<'a> {
	ptr: NonNull<c_wchar>,
	_borrow: PhantomData<&'a [c_wchar]>
}

impl CStrPtr16<'static> {
	pub const EMPTY: Self = unsafe {
		const EMPTY_ZERO: c_wchar = 0;
		Self::new(nonnull_ref(&EMPTY_ZERO))
	};
}

impl<'a> CStrPtr16<'a> {
	pub const unsafe fn new(ptr: NonNull<c_wchar>) -> Self {
		Self {
			ptr,
			_borrow: PhantomData,
		}
	}

	pub const fn as_ptr(self) -> *const c_wchar {
		self.ptr.as_ptr()
	}

	pub const fn as_ptr_ref(self) -> &'a c_wchar {
		unsafe {
			transmute(self.ptr)
		}
	}

	pub const fn ptr(self) -> NonNull<c_wchar> {
		self.ptr
	}

	pub const fn ptr_ref(&self) -> &NonNull<c_wchar> {
		&self.ptr
	}

	pub unsafe fn ptr_mut(&mut self) -> &mut NonNull<c_wchar> {
		&mut self.ptr
	}

	pub const fn as_c_ref(self) -> &'a CStrRef16 {
		CStrRef16::with_c_ptr(self)
	}

	pub fn as_data(self) -> &'a [u16] {
		let mut p = self.as_ptr();
		let mut len = 0;
		unsafe {
			while ptr::read(p) != 0 {
				len += 1;
				p = p.add(1);
			}
			from_raw_parts(self.ptr.as_ptr(), len)
		}
	}

	pub fn as_data_with_nul(self) -> &'a [u16] {
		let data = self.as_data();
		unsafe {
			from_raw_parts(data.as_ptr(), data.len() + 1)
		}
	}

	#[cfg(feature = "std")]
	pub fn to_os_string(self) -> OsString {
		match () {
			#[cfg(windows)]
			() => OsString::from_wide(self.as_data()),
			#[cfg(not(windows))]
			() => OsString::from(self.to_string_lossy()),
		}
	}

	pub fn to_string_lossy(self) -> String {
		String::from_utf16_lossy(self.as_data())
	}

	pub fn utf8(self) -> WideUtf8Reader<'a> {
		WideUtf8Reader::new(self.as_data())
	}

	pub fn utf16(self) -> DecodeUtf16<iter::Copied<slice::Iter<'a, u16>>> {
		decode_utf16(self.as_data().iter().copied())
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		unsafe { *self.ptr.as_ptr() == 0 }
	}

	pub const unsafe fn immortal<'p>(self) -> CStrPtr16<'p> {
		transmute(self)
	}
}

struct CStrPtr16Debug<'a>(CStrPtr16<'a>);
impl fmt::Debug for CStrPtr16Debug<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for c in self.0.utf16() {
			match c {
				Ok(c) =>
					fmt::Write::write_char(f, c),
				Err(e) =>
					write!(f, "\\x{:x}", e.unpaired_surrogate()),
			}?;
		}
		Ok(())
	}
}

impl fmt::Debug for CStrPtr16<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("CStrPtr16")
			.field(&self.ptr)
			.field(&CStrPtr16Debug(*self))
			.finish()
	}
}

impl fmt::Display for CStrPtr16<'_> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self.as_c_ref(), f)
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-03")]
impl From<CStrPtr16<'_>> for windows_strings_03::HSTRING {
	fn from(cstr: CStrPtr16) -> Self {
		windows_strings_03::HSTRING::from_wide(cstr.as_data())
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-04")]
impl From<CStrPtr16<'_>> for windows_strings_04::HSTRING {
	fn from(cstr: CStrPtr16) -> Self {
		windows_strings_04::HSTRING::from_wide(cstr.as_data())
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-03")]
impl From<CStrPtr16<'_>> for windows_strings_03::PCWSTR {
	fn from(cstr: CStrPtr16) -> Self {
		Self::from_raw(cstr.as_ptr())
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-strings-04")]
impl From<CStrPtr16<'_>> for windows_strings_04::PCWSTR {
	fn from(cstr: CStrPtr16) -> Self {
		Self::from_raw(cstr.as_ptr())
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-core-060")]
impl windows_core_060::Param<windows_core_060::PCWSTR> for CStrPtr16<'_> {
	unsafe fn param(self) -> windows_core_060::ParamValue<windows_core_060::PCWSTR> {
		windows_core_060::PCWSTR::from(self).param()
	}
}

#[cfg(windows)]
#[cfg(feature = "windows-core-061")]
impl windows_core_061::Param<windows_core_061::PCWSTR> for CStrPtr16<'_> {
	unsafe fn param(self) -> windows_core_061::ParamValue<windows_core_061::PCWSTR> {
		windows_core_061::PCWSTR::from(self).param()
	}
}

#[repr(transparent)]
pub struct CStrRef16 {
	_cstr: PhantomData<[c_wchar]>,
	start: c_wchar,
}

impl CStrRef16 {
	pub const EMPTY: &'static Self = unsafe {
		Self::new_ref_unchecked(&0)
	};

	#[inline]
	pub fn new<'p, T: Into<CStrPtr16<'p>>>(cstr: T) -> &'p Self {
		let cstr = cstr.into();
		Self::with_c_ptr(cstr.into())
	}

	#[inline]
	pub const unsafe fn new_ref_unchecked(cstr: &c_wchar) -> &Self {
		transmute(cstr)
	}

	#[inline]
	pub const unsafe fn from_ptr<'p>(cstr: *const c_wchar) -> &'p Self {
		unsafe {
			transmute(cstr)
		}
	}

	#[inline]
	pub const unsafe fn with_ptr<'p>(cstr: *const Self) -> &'p Self {
		Self::from_ptr(cstr as *const c_wchar)
	}

	#[inline]
	pub const fn with_c_ptr<'a>(cstr: CStrPtr16<'a>) -> &'a Self {
		unsafe {
			transmute(cstr)
		}
	}

	#[inline]
	pub const fn as_ptr(&self) -> *const c_wchar {
		unsafe {
			transmute(self)
		}
	}

	#[inline]
	pub const fn as_c_ptr(&self) -> CStrPtr16<'_> {
		unsafe {
			transmute(self)
		}
	}

	#[inline]
	pub const fn c_ptr_ref<'r, 'p>(cstr: &'r &'p Self) -> &'r CStrPtr16<'p> {
		unsafe {
			transmute(cstr)
		}
	}

	pub fn to_data(&self) -> &[u16] {
		self.as_c_ptr().as_data()
	}

	pub fn to_data_with_nul(&self) -> &[u16] {
		self.as_c_ptr().as_data_with_nul()
	}

	#[cfg(feature = "std")]
	pub fn to_os_string(&self) -> OsString {
		self.as_c_ptr().to_os_string()
	}

	pub fn to_string_lossy(&self) -> String {
		self.as_c_ptr().to_string_lossy()
	}

	pub fn utf8(&self) -> WideUtf8Reader<'_> {
		self.as_c_ptr().utf8()
	}

	pub fn utf16(&self) -> DecodeUtf16<iter::Copied<slice::Iter<'_, u16>>> {
		self.as_c_ptr().utf16()
	}

	#[inline]
	pub const fn is_empty(&self) -> bool {
		self.start == 0
	}
}

impl Default for &'_ CStrRef16 {
	fn default() -> Self {
		CStrRef16::EMPTY
	}
}

impl fmt::Debug for CStrRef16 {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("CStrRef16")
			.field(&format_args!("\"{}\"", self))
			.finish()
	}
}

impl fmt::Display for CStrRef16 {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for c in self.utf16() {
			let c = c.unwrap_or(char::REPLACEMENT_CHARACTER);
			fmt::Write::write_char(f, c)?;
		}
		Ok(())
	}
}

impl PartialOrd for CStrRef16 {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.to_data().partial_cmp(other.to_data())
	}
}

impl Ord for CStrRef16 {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.to_data().cmp(other.to_data())
	}
}

impl PartialEq for CStrRef16 {
	fn eq(&self, other: &Self) -> bool {
		self.to_data().eq(other.to_data())
	}
}

impl Eq for CStrRef16 {}

impl hash::Hash for CStrRef16 {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.to_data().hash(state)
	}
}

#[cfg(todo)]
impl ToOwned for CStrRef16 {
	type Owned = CString16;

	fn to_owned(&self) -> Self::Owned {
		self.to_data().to_owned()
	}
}

#[cfg(todo)]
impl Borrow<CStrRef16> for HSTRING {
	#[inline]
	fn borrow(&self) -> &CStrRef16 {
		CStrRef16::new(self)
	}
}

impl Borrow<CStrRef16> for CStrPtr16<'_> {
	#[inline]
	fn borrow(&self) -> &CStrRef16 {
		self.as_c_ref()
	}
}

#[cfg(todo)]
impl AsRef<CStrRef16> for HSTRING {
	#[inline]
	fn as_ref(&self) -> &CStrRef16 {
		CStrRef16::new(self)
	}
}

impl AsRef<CStrRef16> for CStrPtr16<'_> {
	#[inline]
	fn as_ref(&self) -> &CStrRef16 {
		self.as_c_ref()
	}
}

// TODO: impl Param for CStrPtr16, and CStrRef/CStrRef16 too

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

#[cfg(feature = "std")]
pub unsafe fn cstring_from_os_unchecked<T: Into<OsString>>(os: T) -> CString {
	let os = os.into();
	CString::from_vec_unchecked(os.into_encoded_bytes())
}

#[cfg(feature = "std")]
pub fn cstring_from_os<T: Into<OsString>>(os: T) -> Result<CString, std::io::Error> {
	let os = os.into();
	let is_ascii = os.as_encoded_bytes()
		.iter().all(|&b| b > 0 && b <= 0x7f);
	match is_ascii {
		true => return Ok(unsafe {
			cstring_from_os_unchecked(os)
		}),
		false => match os.into_string() {
			Ok(s) => CString::new(s)
				.map_err(Into::into),
			Err(os) => return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("CStrings must be non-null ASCII or UTF-8, instead got: {:?}", os),
			)),
		},
	}
}

#[macro_export]
macro_rules! cstr {
	(&$($s:tt)*) => {
		$crate::cstr::cstr!(*$($s)*).as_c_ptr()
	};
	(*$($s:tt)*) => {
		unsafe {
			$crate::cstr::CStrRef::from_ptr(
				concat!($($s)*, "\0").as_bytes().as_ptr() as *const _
			)
		}
	};
	($($s:tt)*) => {
		unsafe {
			::std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($($s)*, "\0").as_bytes())
		}
	};
}
pub use cstr;

use super::{nonnull_ref_cast, nonnull_ref_ref};
