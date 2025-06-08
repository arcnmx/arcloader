use std::{mem::{transmute, MaybeUninit}, ptr};

pub fn new_uninit_slice_box<T>(len: usize) -> Box<[MaybeUninit<T>]> {
	match () {
		#[cfg(msrv = "1.82.0")]
		_ => Box::<[T]>::new_uninit_slice(heap_len),
		#[cfg(not(msrv = "1.82.0"))]
		_ => {
			let mut v = Vec::<MaybeUninit<T>>::with_capacity(len);
			unsafe {
				v.set_len(len);
				v.into_boxed_slice()
			}
		},
	}
}

pub fn write_clone_of_slice<'d, T: Clone>(dst: &'d mut [MaybeUninit<T>], src: &[T]) -> &'d mut [T] {
	match () {
		#[cfg(feature = "unstable")]
		_ => dst.write_clone_of_slice(src),
		#[cfg(not(feature = "unstable"))]
		_ => {
			for (dst, src) in dst.iter_mut().zip(src) {
				dst.write(src.clone());
			}
			unsafe {
				transmute(dst)
			}
		},
	}
}

pub fn write_copy_of_slice<'d, T: Copy>(dst: &'d mut [MaybeUninit<T>], src: &[T]) -> &'d mut [T] {
	match () {
		#[cfg(feature = "unstable")]
		_ => dst.write_copy_of_slice(src),
		#[cfg(not(feature = "unstable"))]
		_ => {
			unsafe {
				ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr().cast::<T>(), dst.len());
				transmute(dst)
			}
		},
	}
}

#[inline(always)]
//#[cfg_attr(feature = "unstable", const)]
pub unsafe fn transmute_unchecked<S, D>(src: S) -> D {
	match () {
		#[cfg(feature = "unstable")]
		() => unsafe { core::intrinsics::transmute_unchecked(src) },
		#[cfg(not(feature = "unstable"))]
		() => unsafe {
			core::mem::transmute_copy(&core::mem::ManuallyDrop::new(src))
		},
	}
}
