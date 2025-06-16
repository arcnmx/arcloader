use crate::windows::{
	com::{
		imp::{interface_hierarchy, get_property, GetProperty, GetProperty2},
		interface::{Interface, InterfacePtr, InterfaceAs, InterfaceTarget},
		unknown::IUnknown_Vtbl,
		InterfaceOwned, InterfaceRef,
	},
	core::{Result, GUID, HRESULT},
	wic::{Palette, WICRect},
};
use std::ptr::NonNull;

/// TODO
pub type BitmapScaler = BitmapSource;
/// TODO
pub type BitmapClipper = BitmapSource;
/// TODO
pub type BitmapFlipRotator = BitmapSource;

pub trait BitmapSourceExt {
	fn size(&self) -> Result<(u32, u32)>;
	fn resolution(&self) -> Result<(f64, f64)>;
	fn rect(&self) -> Result<WICRect> {
		self.size().map(|(w, h)| WICRect {
			x: 0,
			y: 0,
			width: w as _,
			height: h as _,
		})
	}

	fn pixel_format(&self) -> Result<GUID>;
	fn copy_palette(&self, dest: InterfaceRef<Palette>) -> Result<()>;
	fn copy_palette_to<P: InterfaceAs<Palette>>(&self, dest: &P) -> Result<()> {
		self.copy_palette(dest.get_parent())
	}
	fn copy_pixels(&self, bounds: &WICRect, stride: u32, dest: &mut [u8]) -> Result<()>;
}
impl<I: InterfaceAs<BitmapSource>> BitmapSourceExt for I {
	fn size(&self) -> Result<(u32, u32)> {
		let mut size = (0, 0);
		unsafe {
			let (w, h) = &mut size;
			(self.get_parent_vtable().GetSize)(self.get_parent().as_raw(), w, h).ok()
		}.map(move |()| size)
	}

	fn resolution(&self) -> Result<(f64, f64)> {
		let mut res = (0.0f64, 0.0f64);
		unsafe {
			let (x, y) = &mut res;
			(self.get_parent_vtable().GetResolution)(self.get_parent().as_raw(), x, y).ok()
		}.map(move |()| res)
	}

	fn pixel_format(&self) -> Result<GUID> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetPixelFormat)
		}
	}

	fn copy_palette(&self, dest: InterfaceRef<Palette>) -> Result<()> {
		unsafe {
			(self.get_parent_vtable().CopyPalette)(self.get_parent().as_raw(), dest.as_raw())
		}.ok()
	}

	fn copy_pixels(&self, bounds: &WICRect, stride: u32, dest: &mut [u8]) -> Result<()> {
		unsafe {
			(self.get_parent_vtable().CopyPixels)(self.get_parent().as_raw(), bounds, stride, dest.len() as _, dest.as_mut_ptr())
		}.ok()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct BitmapSource {
	pub interface: InterfaceOwned<BitmapSource>,
}

unsafe impl Interface for BitmapSource {
	/// {00000120-A8F2-4877-BA0A-FD2B6645FB94}
	const IID: GUID = GUID::from_values(0x00000120, 0xa8f2, 0x4877, [0xba, 0x0a,
		0xfd, 0x2b, 0x66, 0x45, 0xfb, 0x94,
	]);

	type Owned = Self;
	type Vtable = IWICBitmapSource_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for BitmapSource {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { BitmapSource, IUnknown }

#[repr(C)]
pub struct IWICBitmapSource_Vtbl {
	pub base__: IUnknown_Vtbl,
	pub GetSize: GetProperty2<u32, u32>,
	pub GetPixelFormat: GetProperty<GUID>,
	pub GetResolution: GetProperty2<f64, f64>,
	pub CopyPalette: unsafe extern "system" fn(this: *mut InterfaceTarget, dst: *mut InterfaceTarget) -> HRESULT,
	pub CopyPixels: unsafe extern "system" fn(this: *mut InterfaceTarget, bounds: *const WICRect, stride: u32, size: u32, buffer: *mut u8) -> HRESULT,
}
