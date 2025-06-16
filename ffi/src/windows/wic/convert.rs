use crate::{
	c_bool32,
	windows::{
		adapter::windows_newtype,
		com::{
			imp::interface_hierarchy,
			interface::{Interface, InterfacePtr, InterfaceAs, InterfaceTarget},
			InterfaceOwned,
		},
		core::{Result, GUID, HRESULT},
		wic::{
			bitmap::{BitmapSource, IWICBitmapSource_Vtbl},
			Palette,
			WICBitmapPaletteType,
		},
	},
};
use core::ptr::NonNull;

/// TODO
pub type ColorTransform = BitmapSource;

windows_newtype! {
	pub struct Imaging::WICBitmapDitherType(pub i32);
}

impl WICBitmapDitherType {
	pub const DUAL_SPIRAL_4X4: Self = Self(6);
	pub const DUAL_SPIRAL_8X8: Self = Self(7);
	pub const ERROR_DIFFUSION: Self = Self(8);
	pub const ORDERED_16X16: Self = Self(3);
	pub const ORDERED_4X4: Self = Self(1);
	pub const ORDERED_8X8: Self = Self(2);
	pub const SOLID: Self = Self(0);
	pub const SPIRAL_4X4: Self = Self(4);
	pub const SPIRAL_8X8: Self = Self(5);
}

pub trait FormatConverterExt {
	fn initialize<S: InterfaceAs<BitmapSource>, P: InterfaceAs<Palette>>(&self, src: &S, pixel_format: &GUID, dither: WICBitmapDitherType, palette: &P, alpha_threshold: f64, translate: WICBitmapPaletteType) -> Result<()>;

	fn can_convert(&self, pixel_format_src: &GUID, pixel_format_dest: &GUID) -> Result<bool>;
}

impl<I: InterfaceAs<FormatConverter>> FormatConverterExt for I {
	fn initialize<S: InterfaceAs<BitmapSource>, P: InterfaceAs<Palette>>(&self, src: &S, pixel_format: &GUID, dither: WICBitmapDitherType, palette: &P, alpha_threshold: f64, translate: WICBitmapPaletteType) -> Result<()> {
		let src = src.get_parent();
		let palette = palette.get_parent();
		unsafe {
			(self.get_parent_vtable().Initialize)(self.get_parent().as_raw(), src.as_raw(), pixel_format, dither, palette.as_raw(), alpha_threshold, translate)
		}.ok()
	}

	fn can_convert(&self, pixel_format_src: &GUID, pixel_format_dest: &GUID) -> Result<bool> {
		let mut out = c_bool32::FALSE;
		unsafe {
			(self.get_parent_vtable().CanConvert)(self.get_parent().as_raw(), pixel_format_src, pixel_format_dest, &mut out)
		}.ok().map(move |()| out.get())
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct FormatConverter {
	pub interface: InterfaceOwned<FormatConverter>,
}
unsafe impl Interface for FormatConverter {
	/// {00000301-A8F2-4877-BA0A-FD2B6645FB94}
	const IID: GUID = GUID::from_values(0x00000301, 0xa8f2, 0x4877, [0xba, 0x0a,
		0xfd, 0x2b, 0x66, 0x45, 0xfb, 0x94,
	]);

	type Owned = Self;
	type Vtable = IWICFormatConverter_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for FormatConverter {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { FormatConverter, IUnknown, BitmapSource }

#[repr(C)]
pub struct IWICFormatConverter_Vtbl {
	pub base__: IWICBitmapSource_Vtbl,
	pub Initialize: unsafe extern "system" fn(this: *mut InterfaceTarget, bitmap_src: *mut InterfaceTarget, pixfmt_dst: *const GUID, dither: WICBitmapDitherType, palette: *mut InterfaceTarget, alpha_threshold_percent: f64, palette_translate: WICBitmapPaletteType) -> HRESULT,
	pub CanConvert: unsafe extern "system" fn(this: *mut InterfaceTarget, pixfmt_src: *const GUID, pixfmt_dst: *const GUID, out: *mut c_bool32) -> HRESULT,
}
