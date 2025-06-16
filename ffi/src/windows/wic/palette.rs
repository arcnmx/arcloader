use crate::{
	c_bool32,
	windows::{
		adapter::windows_newtype,
		com::{
			imp::{get_property, interface_hierarchy, GetProperty, GetMultiple, Unimplemented},
			interface::{Interface, InterfacePtr, InterfaceAs, InterfaceTarget},
			unknown::{IUnknown, IUnknown_Vtbl},
			InterfaceOwned,
		},
		core::{GUID, Result},
	},
};
use std::ptr::NonNull;

windows_newtype! {
	pub struct Imaging::WICBitmapPaletteType(pub i32);
}
impl WICBitmapPaletteType {
	pub const CUSTOM: Self = Self(0);
	pub const FIXED_BW: Self = Self(2);
	pub const FIXED_GRAY16: Self = Self(11);
	pub const FIXED_GRAY256: Self = Self(12);
	pub const FIXED_GRAY4: Self = Self(10);
	pub const FIXED_HALFTONE125: Self = Self(6);
	pub const FIXED_HALFTONE216: Self = Self(7);
	pub const FIXED_HALFTONE252: Self = Self(8);
	pub const FIXED_HALFTONE256: Self = Self(9);
	pub const FIXED_HALFTONE27: Self = Self(4);
	pub const FIXED_HALFTONE64: Self = Self(5);
	pub const FIXED_HALFTONE8: Self = Self(3);
	pub const FIXED_WEB_PALETTE: Self = Self(7);
	pub const MEDIAN_CUT: Self = Self(1);
}

/// TODO
pub type ColorContext = IUnknown;

pub trait PaletteExt {
	fn kind(&self) -> Result<WICBitmapPaletteType>;
	fn color_count(&self) -> Result<u32>;
	fn is_black_white(&self) -> Result<bool>;
	fn is_grayscale(&self) -> Result<bool>;
	fn has_alpha(&self) -> Result<bool>;
	fn get_colors(&self, colours: &mut [u32]) -> Result<usize>;
}

impl<I: InterfaceAs<Palette>> PaletteExt for I {
	fn kind(&self) -> Result<WICBitmapPaletteType> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetType)
		}
	}

	fn color_count(&self) -> Result<u32> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetColorCount)
		}
	}

	fn is_black_white(&self) -> Result<bool> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().IsBlackWhite)
		}.map(Into::into)
	}

	fn is_grayscale(&self) -> Result<bool> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().IsGrayscale)
		}.map(Into::into)
	}

	fn has_alpha(&self) -> Result<bool> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().HasAlpha)
		}.map(Into::into)
	}

	fn get_colors(&self, colours: &mut [u32]) -> Result<usize> {
		let mut count = 0;
		unsafe {
			(self.get_parent_vtable().GetColors)(self.get_parent().as_raw(), colours.len() as _, colours.as_mut_ptr(), &mut count)
		}.ok().map(move |()| count as usize)
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Palette {
	pub interface: InterfaceOwned<Palette>,
}
unsafe impl Interface for Palette {
	/// {00000040-A8F2-4877-BA0A-FD2B6645FB94}
	const IID: GUID = GUID::from_values(0x00000040, 0xa8f2, 0x4877, [0xba, 0x0a,
		0xfd, 0x2b, 0x66, 0x45, 0xfb, 0x94,
	]);

	type Owned = Self;
	type Vtable = IWICPalette_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for Palette {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { Palette, IUnknown }

#[repr(C)]
pub struct IWICPalette_Vtbl {
	pub base__: IUnknown_Vtbl,
	pub InitializePredefined: Unimplemented,
	pub InitializeCustom: Unimplemented,
	pub InitializeFromBitmap: Unimplemented,
	pub InitializeFromPalette: Unimplemented,
	pub GetType: GetProperty<WICBitmapPaletteType>,
	pub GetColorCount: GetProperty<u32>,
	pub GetColors: GetMultiple<u32>,
	pub IsBlackWhite: GetProperty<c_bool32>,
	pub IsGrayscale: GetProperty<c_bool32>,
	pub HasAlpha: GetProperty<c_bool32>,
}
