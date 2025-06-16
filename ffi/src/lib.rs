#![allow(non_camel_case_types)]

pub mod wide;
pub mod windows;

mod nn;
pub use self::nn::*;

mod ptr;
pub use self::ptr::*;

#[macro_use]
pub mod cstr;

#[doc(hidden)]
#[macro_use]
pub mod externs;

pub type UserMallocFn = unsafe extern "C" fn(size: usize, user_data: *mut c_void) -> *mut c_void;
pub type UserFreeFn = unsafe extern "C" fn(p: *mut c_void, user_data: *mut c_void);

pub use core::{
	ffi::{c_void, c_int, c_uint, c_long, c_ulong, c_char, c_uchar, c_schar},
	ptr::NonNull,
};
pub use core::ffi::{c_int as c_senum, c_uint as c_uenum};
pub use crate::cstr::c_wchar;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct c_bool {
	pub value: u8,
}

impl c_bool {
	pub const FALSE: Self = Self::with(0);
	pub const TRUE: Self = Self::with(1);

	#[inline(always)]
	pub const fn with(value: u8) -> Self {
		Self { value }
	}

	#[inline(always)]
	pub const fn new(value: bool) -> Self {
		Self::with(value as _)
	}

	/// Is true?
	#[inline(always)]
	pub const fn get(self) -> bool {
		self.value != Self::FALSE.value
	}

	#[inline(always)]
	pub const fn get_lsb(self) -> bool {
		self.value & 1 != 0
	}

	#[inline(always)]
	pub const fn try_get(self) -> Result<bool, u8> {
		match self {
			Self::TRUE => Ok(true),
			Self::FALSE => Ok(false),
			_ => Err(self.value),
		}
	}

	#[inline(always)]
	pub const fn is_false(self) -> bool {
		self.value == Self::FALSE.value
	}
}

impl From<bool> for c_bool {
	#[inline(always)]
	fn from(value: bool) -> c_bool {
		c_bool::new(value)
	}
}

impl From<c_bool> for bool {
	#[inline(always)]
	fn from(value: c_bool) -> bool {
		value.get()
	}
}

impl From<u8> for c_bool {
	#[inline(always)]
	fn from(value: u8) -> c_bool {
		c_bool::with(value)
	}
}

impl From<c_bool> for u8 {
	#[inline(always)]
	fn from(value: c_bool) -> u8 {
		value.value
	}
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct c_bool32 {
	pub value: u32,
}

impl c_bool32 {
	pub const FALSE: Self = Self::with(0);
	pub const TRUE: Self = Self::with(1);

	#[inline(always)]
	pub const fn with(value: u32) -> Self {
		Self { value }
	}

	#[inline(always)]
	pub const fn new(value: bool) -> Self {
		Self::with(value as _)
	}

	/// Is true?
	#[inline(always)]
	pub const fn get(self) -> bool {
		self.value != Self::FALSE.value
	}

	#[inline(always)]
	pub const fn get_lsb(self) -> bool {
		self.value & 1 != 0
	}

	/// truncate to 8-bit value without losing non-zero truthiness
	#[inline(always)]
	pub const fn get8(self) -> c_bool {
		match self.value {
			value @ 0..=0xff => c_bool::with(value as u8),
			_ => c_bool::TRUE,
		}
	}

	#[inline(always)]
	pub const fn try_get(self) -> Result<bool, u32> {
		match self {
			Self::TRUE => Ok(true),
			Self::FALSE => Ok(false),
			_ => Err(self.value),
		}
	}

	#[inline(always)]
	pub const fn is_false(self) -> bool {
		self.value == Self::FALSE.value
	}
}

impl From<bool> for c_bool32 {
	#[inline(always)]
	fn from(value: bool) -> c_bool32 {
		c_bool32::new(value)
	}
}

impl From<c_bool32> for bool {
	#[inline(always)]
	fn from(value: c_bool32) -> bool {
		value.get()
	}
}

impl From<u32> for c_bool32 {
	#[inline(always)]
	fn from(value: u32) -> c_bool32 {
		c_bool32::with(value)
	}
}

impl From<c_bool32> for u32 {
	#[inline(always)]
	fn from(value: c_bool32) -> u32 {
		value.value
	}
}

impl From<c_bool> for c_bool32 {
	#[inline(always)]
	fn from(value: c_bool) -> c_bool32 {
		c_bool32::with(value.value.into())
	}
}

impl From<c_bool32> for c_bool {
	#[inline(always)]
	fn from(value: c_bool32) -> c_bool {
		value.get8()
	}
}

#[test]
fn enum_repr_c() {
	#![allow(dead_code)]
	use core::{hint::black_box, mem::transmute};

	#[repr(C)]
	enum ReprCu {
		X = 1,
		Y,
		Z = 9,
	}
	#[repr(C)]
	enum ReprCs {
		X = 1,
		Y,
		Z = -1,
	}

	let repr_c_u: c_uenum = unsafe {
		transmute(ReprCu::Z)
	};
	assert_eq!(repr_c_u, ReprCu::Z as c_uenum);

	let repr_c_s: c_senum = unsafe {
		transmute(ReprCs::Z)
	};
	assert_eq!(repr_c_s, ReprCs::Z as c_senum);

	let repr_c_u_0: c_uenum = black_box(unsafe {
		transmute(None::<ReprCu>)
	});
	assert_eq!(repr_c_u_0, 0);
	let repr_c_u_1: c_uenum = black_box(unsafe {
		transmute(Err::<(), ReprCu>(ReprCu::X))
	});
	assert_eq!(repr_c_u_1, 1);

	let repr_c_s_0: c_senum = black_box(unsafe {
		transmute(None::<ReprCs>)
	});
	if repr_c_s_0 == 0 {
		panic!("enum repr(C) changed for signed integers?");
	}
	let repr_c_s_1: c_senum = black_box(unsafe {
		transmute(Err::<(), ReprCs>(ReprCs::Z))
	});
	assert_eq!(repr_c_s_1, -1);
}
