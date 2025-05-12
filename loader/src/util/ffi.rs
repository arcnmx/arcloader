use std::{
	ffi::{CStr, c_char},
	ptr::{self, NonNull},
};

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

pub fn nonnull_const<P: ?Sized>(p: *const P) -> Option<NonNull<P>> {
	NonNull::new(p as *mut P)
}

pub fn nonnull_ref<P: ?Sized>(p: &P) -> NonNull<P> {
	unsafe {
		nonnull_ref_unchecked(p)
	}
}

pub unsafe fn nonnull_ref_unchecked<P: ?Sized>(p: *const P) -> NonNull<P> {
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
