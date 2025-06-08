use core::{mem::transmute, ptr::{self, NonNull}};

#[inline]
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

#[inline]
pub fn nonnull_unwrap_unchecked<T: ?Sized>(p: Option<NonNull<T>>) -> NonNull<T> {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub fn nonnull_unwrap<T: ?Sized>(p: Option<NonNull<T>>) -> *const T {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub fn nonnull_unwrap_mut<T: ?Sized>(p: Option<NonNull<T>>) -> *mut T {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub const fn nonnull_opt_cast<U, T: ?Sized>(p: Option<NonNull<T>>) -> Option<NonNull<U>> {
	match () {
		#[cfg(feature = "unstable")]
		() => unsafe { core::intrinsics::transmute_unchecked(src) },
		#[cfg(not(feature = "unstable"))]
		() => *nonnull_opt_ref_cast(&p),
	}
}

#[inline]
pub const fn nonnull_ref<P: ?Sized>(p: &P) -> NonNull<P> {
	unsafe {
		nonnull_ref_unchecked(p)
	}
}

#[inline]
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

#[inline]
pub const fn nonnull_ref_ref<'p, T: ?Sized>(p: &'p &'_ T) -> &'p NonNull<T> {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub const fn nonnull_ref_ptr<T: ?Sized>(p: &*const T) -> &Option<NonNull<T>> {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub const fn nonnull_ref_ptr_mut<T: ?Sized>(p: &*mut T) -> &Option<NonNull<T>> {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub const fn nonnull_unwrap_ref_unchecked<T: ?Sized>(p: &Option<NonNull<T>>) -> &NonNull<T> {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub const fn nonnull_ref_cast<U, T: ?Sized>(p: &NonNull<T>) -> &NonNull<U> {
	unsafe {
		let p: *const NonNull<U> = p as *const NonNull<T> as *const _;
		transmute(p)
	}
}

#[inline]
pub const fn nonnull_opt_ref_cast<U, T: ?Sized>(p: &Option<NonNull<T>>) -> &Option<NonNull<U>> {
	unsafe {
		let p: *const Option<NonNull<U>> = p as *const Option<NonNull<T>> as *const _;
		transmute(p)
	}
}

#[inline]
pub unsafe fn nonnull_ref_cast_unsized<T: ?Sized, U: ?Sized>(p: &NonNull<T>) -> &NonNull<U> {
	#[cfg(debug_assertions)]
	use core::mem::size_of;
	match () {
		#[cfg(debug_assertions)]
		_ if size_of::<*const T>() < size_of::<*const U>() =>
			panic!("&NonNull cast size mismatch"),
		_ => {},
	}
	unsafe {
		let p: *const NonNull<U> = p as *const NonNull<T> as *const _;
		transmute(p)
	}
}

#[inline]
pub fn nonnull_mut_ref<T: ?Sized>(p: &mut *const T) -> &mut Option<NonNull<T>> {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub fn nonnull_mut_mut<T: ?Sized>(p: &mut *mut T) -> &mut Option<NonNull<T>> {
	unsafe {
		transmute(p)
	}
}

#[inline]
pub fn nonnull_unwrap_mut_unchecked<T: ?Sized>(p: &mut Option<NonNull<T>>) -> &mut NonNull<T> {
	unsafe {
		transmute(p)
	}
}

pub fn nonnull_mut_cast<T: ?Sized, U>(p: &mut NonNull<T>) -> &mut NonNull<U> {
	unsafe {
		let p: *mut NonNull<U> = p as *mut NonNull<T> as *mut _;
		transmute(p)
	}
}

#[inline]
pub unsafe fn opt_unwrap_unchecked<'a, T>(opt: Option<T>) -> T {
	#[allow(unreachable_patterns)]
	match opt {
		#[cfg(debug_assertions)]
		opt => opt.unwrap(),
		#[cfg(not(debug_assertions))]
		None => unsafe {
			core::hint::unreachable_unchecked()
		},
		#[cfg(not(debug_assertions))]
		Some(some) => some,
	}
}

#[inline]
pub unsafe fn opt_ref_unwrap_unchecked<'a, T>(opt: &Option<T>) -> &T {
	match opt {
		#[cfg(debug_assertions)]
		opt => opt.as_ref().unwrap(),
		#[cfg(not(debug_assertions))]
		opt => unsafe {
			transmute(opt)
		},
	}
}

#[inline]
pub const fn opt_ref_some<T>(some: &T) -> &Option<T> {
	unsafe {
		transmute(some)
	}
}

trait OptionNoneRef<'a>: Sized + 'a {
	const NONE: &'a Option<Self>;
}

impl<'a, T: 'a> OptionNoneRef<'a> for T {
	const NONE: &'a Option<T> = &None;
}

#[inline]
pub fn opt_ref_none<'a, T>() -> &'a Option<T> {
	<T as OptionNoneRef>::NONE
}

#[inline]
pub fn opt_ref_from<T>(opt: Option<&T>) -> &Option<T> {
	match opt {
		Some(some) => opt_ref_some(some),
		None => <T as OptionNoneRef>::NONE,
	}
}

#[inline]
pub fn opt_ref_map<R, T: ?Sized, F: FnOnce(&T) -> &R>(opt: Option<&T>, f: F) -> &Option<R> {
	match opt {
		Some(some) => opt_ref_some(f(some)),
		None => &None,
	}
}

#[inline]
pub fn opt_ref_then<R, T: ?Sized, F: FnOnce(&T) -> Option<&R>>(opt: Option<&T>, f: F) -> &Option<R> {
	match opt {
		Some(some) => opt_ref_from(f(some)),
		None => &None,
	}
}
