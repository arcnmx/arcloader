use crate::{
	cstr::c_wchar,
	windows::{
		core::{Result, HRESULT},
		com::{
			interface::{InterfaceTarget, InterfacePtr},
			E_POINTER, E_INVALIDARG,
		},
	},
};
use ::core::{convert::Infallible, mem::{transmute, MaybeUninit}, ptr::NonNull};
#[cfg(all(windows, feature = "std"))]
use std::{ffi::OsString, os::windows::ffi::OsStringExt};

pub type Unimplemented = unsafe extern "system" fn(this: *mut InterfaceTarget, _: Infallible) -> HRESULT;
pub type Routine = unsafe extern "system" fn(this: *mut InterfaceTarget) -> HRESULT;
pub type GetStringW = GetMultiple<c_wchar>;
pub type SetProperty<T> = unsafe extern "system" fn(this: *mut InterfaceTarget, value: T) -> HRESULT;
pub type GetProperty<T> = unsafe extern "system" fn(this: *mut InterfaceTarget, out: *mut T) -> HRESULT;
pub type GetProperty2<T0, T1> = unsafe extern "system" fn(this: *mut InterfaceTarget, out0: *mut T0, out1: *mut T1) -> HRESULT;
pub type GetPropertyBy<T, K> = unsafe extern "system" fn(this: *mut InterfaceTarget, key: K, out: *mut T) -> HRESULT;
pub type GetMultiple<T> = unsafe extern "system" fn(this: *mut InterfaceTarget, len: u32, values: *mut T, len_actual: *mut u32) -> HRESULT;
pub type GetInterface<I/*: InterfacePtr*/> = GetProperty<Option<I>>;

pub unsafe fn get_string_w(this: *mut InterfaceTarget, f: GetStringW) -> Result<Vec<c_wchar>> {
	let mut buf = [MaybeUninit::<c_wchar>::uninit(); 48];
	let mut len = 0;
	f(this, buf.len() as u32, buf.as_mut_ptr() as *mut c_wchar, &mut len).ok()?;
	let lensz = len as usize;
	if let Some(b) = buf.get(..lensz) {
		return Ok(match b.is_empty() {
			true => Default::default(),
			false => unsafe {
				&*(b as *const [MaybeUninit<c_wchar>] as *const [c_wchar])
			}.to_owned(),
		})
	}

	let mut buf = Vec::with_capacity(lensz);
	f(this, buf.capacity() as u32, buf.as_mut_ptr(), &mut len).ok()?;
	if len as usize != lensz {
		return Err(E_INVALIDARG.into())
	}
	buf.set_len(len as usize);
	Ok(buf)
}

#[cfg(all(windows, feature = "std"))]
pub fn string_w_os<S: AsRef<[u16]>>(s: S) -> OsString {
	let s = s.as_ref();
	let s = s.strip_suffix(&[0u16]).unwrap_or(s);
	OsString::from_wide(s)
}

pub unsafe fn get_property<T>(this: *mut InterfaceTarget, f: GetProperty<T>) -> Result<T> {
	let mut prop = MaybeUninit::<T>::uninit();
	f(this, prop.as_mut_ptr()).ok()
		.map(move |()| unsafe {
			prop.assume_init()
		})
}

pub unsafe fn get_property_opt<T>(this: *mut InterfaceTarget, f: GetProperty<Option<T>>) -> Result<T> {
	let mut prop = None;
	let res = f(this, &mut prop);
	match (res.ok(), prop) {
		(Ok(()), Some(i)) => Ok(i),
		(Ok(()), None) => Err(E_POINTER.into()),
		(Err(e), _) => Err(e),
	}
}

#[inline(always)]
pub unsafe fn get_interface<T: InterfacePtr>(this: *mut InterfaceTarget, f: GetInterface<T>) -> Result<T> {
	get_interface_inner(this, transmute(f))
		.map(|i| T::from_nn(i))
}

pub unsafe fn get_interface_inner(this: *mut InterfaceTarget, f: GetProperty<Option<NonNull<InterfaceTarget>>>) -> Result<NonNull<InterfaceTarget>> {
	get_property_opt(this, f)
}

#[inline(always)]
pub fn interface_res<I: InterfacePtr>(res: HRESULT, interface: Option<I>) -> Result<I> {
	unsafe {
		let interface = interface.map(|i| i.into_nn());
		let res = interface_res_inner(res, interface);
		res.map(|i| I::from_nn(i))
	}
}

#[inline(never)]
fn interface_res_inner(res: HRESULT, interface: Option<NonNull<InterfaceTarget>>) -> Result<NonNull<InterfaceTarget>> {
	match (res.ok(), interface) {
		(Ok(()), Some(i)) => Ok(i),
		(Ok(()), None) => Err(E_POINTER.into()),
		(Err(e), _) => Err(e),
	}
}

#[macro_export]
macro_rules! interface_hierarchy {
	($ty:ident, IUnknown $($parents:tt)*) => {
		$crate::windows::com::imp::interface_hierarchy! {
			$ty, $crate::windows::com::IUnknown $($parents)*
		}
		#[doc(hidden)]
		#[allow(unused)]
		impl $ty {
			fn __impl__interface_h() {
				$crate::windows::adapter::windows_adapter! { pub(crate) mod core as core0xx_hier =>
					impl core0xx_hier::imp::CanInto<core0xx_hier::IUnknown> for super::$ty {}
				}
			}
		}
	};
	($ty:ty $(, $parent:ty)*$(,)?) => {
		unsafe impl $crate::windows::com::interface::InterfaceBase<$ty> for $ty {}
		$(
			unsafe impl $crate::windows::com::interface::InterfaceBase<$parent> for $ty {}
			impl AsRef<$parent> for $ty where
				Self: $crate::windows::com::interface::InterfacePtr,
				$parent: $crate::windows::com::interface::InterfacePtr,
			{
				#[inline]
				fn as_ref(&self) -> &$parent {
					$crate::windows::com::interface::InterfaceBase::as_parent_ref(self)
				}
			}
			impl From<$ty> for $parent where
				$parent: $crate::windows::com::InterfacePtr,
			{
				#[inline]
				fn from(i: $ty) -> Self {
					unsafe {
						$crate::windows::com::InterfacePtr::from_nn(
							$crate::windows::com::Interface::into_nn(i)
						)
					}
				}
			}
		)*

		$crate::windows::com::imp::interface_hierarchy! {
			// why reverse order...
			@parents($ty) $($parent),*
		}
	};
	(@parents($ty:ty)) => {
		// base or orphaned class like IUnknown
	};
	(@parents($ty:ty) $base:ty) => {
		// singular base parent, likely IUnknown
		impl ::core::ops::Deref for $ty where
			Self: $crate::windows::com::InterfacePtr,
		{
			type Target = $base;
			#[inline]
			fn deref(&self) -> &$base {
				$crate::windows::com::interface::InterfaceBase::as_parent_ref(self)
			}
		}
	};
	(@parents($ty:ty) $base:ty $(, $parents:ty)+) => {
		$crate::windows::com::imp::interface_hierarchy! {
			@parent($ty: $base) $($parents),+
		}
	};
	(@parent($ty:ty: $base:ty) $parent:ty) => {
		impl ::core::ops::Deref for $ty {
			type Target = $parent;
			#[inline]
			fn deref(&self) -> &$parent {
				$crate::windows::com::interface::InterfaceBase::as_parent_ref(self)
			}
		}
	};
	(@parent($ty:ty: $base:ty) $sub:ty $(, $parents:ty)+) => {
		$crate::windows::com::imp::interface_hierarchy! {
			@parent($ty: $base) $($parents),+
		}
	};
}
pub use interface_hierarchy;
