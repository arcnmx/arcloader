use core::{borrow::Borrow, mem::transmute, ptr::NonNull};
use crate::{
	nn::nonnull_ref_unchecked,
	cstr::{CStrPtr, CStrPtr16, CStrRef, CStrRef16},
};
#[cfg(feature = "alloc")]
use crate::cstr::CStrBox;
use strings0xx::{HSTRING, PCSTR, PCWSTR};

#[cfg(feature = "alloc")]
impl From<CStrPtr<'_>> for HSTRING {
	fn from(cstr: CStrPtr) -> Self {
		match cstr {
			#[cfg(feature = "std")]
			cstr => (&*cstr.to_os_str()).into(),
			#[cfg(not(feature = "std"))]
			cstr => cstr.as_c_str().to_string_lossy()[..].into(),
		}
	}
}

#[cfg(feature = "alloc")]
impl From<&'_ CStrRef> for HSTRING {
	#[inline]
	fn from(cstr: &'_ CStrRef) -> Self {
		cstr.as_c_ptr().into()
	}
}

#[cfg(feature = "alloc")]
impl From<&'_ CStrBox> for HSTRING {
	#[inline]
	fn from(cstr: &'_ CStrBox) -> Self {
		cstr.as_c_ptr().into()
	}
}

#[cfg(feature = "alloc")]
impl From<CStrBox> for HSTRING {
	#[inline]
	fn from(cstr: CStrBox) -> Self {
		cstr.as_c_ptr().into()
	}
}

impl From<CStrPtr16<'_>> for HSTRING {
	fn from(cstr: CStrPtr16) -> Self {
		HSTRING::from_wide(cstr.as_data())
	}
}

impl From<&'_ CStrRef16> for HSTRING {
	#[inline]
	fn from(cstr: &'_ CStrRef16) -> Self {
		cstr.as_c_ptr().into()
	}
}

impl<'a> From<&'a HSTRING> for &'a CStrPtr16<'a> {
	fn from(hstring: &'a HSTRING) -> Self {
		match hstring_data_ref(hstring) {
			Some(data) if !data.is_null() => unsafe {
				let data: &'a &'a crate::c_wchar = transmute(data);
				CStrPtr16::new_ref_ref(data)
			},
			_ => &CStrPtr16::EMPTY,
		}
	}
}

impl<'a> AsRef<CStrPtr16<'a>> for &'a HSTRING {
	fn as_ref(&self) -> &CStrPtr16<'a> {
		From::from(*self)
	}
}

impl<'a> Borrow<CStrPtr16<'a>> for &'a HSTRING {
	fn borrow(&self) -> &CStrPtr16<'a> {
		self.as_ref()
	}
}

fn hstring_data_ref(s: &HSTRING) -> Option<&*mut u16> {
	unsafe {
		let header = *(s as *const HSTRING as *const *mut u32);
		let header = NonNull::new(header)?;
		let data = header.add(4).cast::<*mut u16>();
		Some(&*data.as_ptr())
	}
}

impl<'a> From<&'a HSTRING> for CStrPtr16<'a> {
	#[inline]
	fn from(cstr: &'a HSTRING) -> Self {
		unsafe {
			CStrPtr16::new(nonnull_ref_unchecked(cstr.as_ptr() as *mut _))
		}
	}
}

impl<'a> From<&'a HSTRING> for &'a CStrRef16 {
	#[inline]
	fn from(cstr: &'a HSTRING) -> Self {
		CStrPtr16::from(cstr).as_c_ref()
	}
}

impl Borrow<CStrRef16> for HSTRING {
	#[inline]
	fn borrow(&self) -> &CStrRef16 {
		From::from(self)
	}
}

impl AsRef<CStrRef16> for HSTRING {
	#[inline]
	fn as_ref(&self) -> &CStrRef16 {
		From::from(self)
	}
}

// TODO: CStrBox16 HSTRING conversions

impl From<CStrPtr<'_>> for PCSTR {
	#[inline]
	fn from(cstr: CStrPtr) -> Self {
		Self::from_raw(cstr.as_uptr())
	}
}

impl From<&'_ CStrRef> for PCSTR {
	#[inline]
	fn from(cstr: &'_ CStrRef) -> Self {
		cstr.as_c_ptr().into()
	}
}

#[cfg(feature = "alloc")]
impl From<&'_ CStrBox> for PCSTR {
	#[inline]
	fn from(cstr: &'_ CStrBox) -> Self {
		cstr.as_c_ptr().into()
	}
}

impl From<CStrPtr16<'_>> for PCWSTR {
	#[inline]
	fn from(cstr: CStrPtr16<'_>) -> Self {
		Self::from_raw(cstr.as_ptr())
	}
}

impl From<&'_ CStrRef16> for PCWSTR {
	#[inline]
	fn from(cstr: &'_ CStrRef16) -> Self {
		cstr.as_c_ptr().into()
	}
}

#[cfg(todo)]
#[cfg(feature = "alloc")]
impl From<&'_ CStrBox16> for PCWSTR {
	#[inline]
	fn from(cstr: &'_ CStrBox16) -> Self {
		cstr.as_c_ptr().into()
	}
}
