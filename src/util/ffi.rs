use std::{
	ffi::{CStr, c_char},
	ptr::NonNull,
};

pub unsafe fn cstr_opt<'a>(s: &'a *const c_char) -> Option<&'a CStr> {
	NonNull::new(*s as *mut c_char)
		.map(|p| CStr::from_ptr(p.as_ptr() as *const c_char))
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
