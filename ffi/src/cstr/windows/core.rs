use crate::cstr::{CStrPtr, CStrPtr16, CStrRef, CStrRef16};
#[cfg(feature = "alloc")]
use crate::cstr::CStrBox;
use core0xx::{Param, ParamValue, TypeKind, CopyType, PCSTR, PCWSTR};

impl Param<PCSTR> for CStrPtr<'_> {
	unsafe fn param(self) -> ParamValue<PCSTR> {
		PCSTR::from(self).param()
	}
}
impl Param<PCSTR> for &'_ CStrPtr<'_> {
	unsafe fn param(self) -> ParamValue<PCSTR> {
		Param::param(*self)
	}
}
impl Param<PCSTR> for &'_ CStrRef {
	unsafe fn param(self) -> ParamValue<PCSTR> {
		self.as_c_ptr().param()
	}
}
#[cfg(feature = "alloc")]
impl Param<PCSTR> for &'_ CStrBox {
	unsafe fn param(self) -> ParamValue<PCSTR> {
		self.as_c_ptr().param()
	}
}

impl Param<PCWSTR> for CStrPtr16<'_> {
	unsafe fn param(self) -> ParamValue<PCWSTR> {
		PCWSTR::from(self).param()
	}
}
impl Param<PCWSTR> for &'_ CStrPtr16<'_> {
	unsafe fn param(self) -> ParamValue<PCWSTR> {
		Param::param(*self)
	}
}
impl Param<PCWSTR> for &'_ CStrRef16 {
	unsafe fn param(self) -> ParamValue<PCWSTR> {
		self.as_c_ptr().param()
	}
}
#[cfg(todo)]
#[cfg(feature = "alloc")]
impl Param<PCWSTR> for &'_ CStrBox16 {
	unsafe fn param(self) -> ParamValue<PCWSTR> {
		self.as_c_ptr().param()
	}
}

impl TypeKind for CStrPtr<'_> {
	type TypeKind = CopyType;
}
impl TypeKind for &'_ CStrRef {
	type TypeKind = CopyType;
}
#[cfg(feature = "alloc")]
impl TypeKind for CStrBox {
	type TypeKind = core0xx::CloneType;
}

impl TypeKind for CStrPtr16<'_> {
	type TypeKind = CopyType;
}
impl TypeKind for &'_ CStrRef16 {
	type TypeKind = CopyType;
}
