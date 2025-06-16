use crate::windows::{
	adapter::windows_newtype,
	com::{
		imp::{GetProperty, GetPropertyBy, GetInterface, get_property, get_interface, interface_res, interface_hierarchy},
		interface::{Interface, InterfacePtr, InterfaceAs, InterfaceTarget},
		unknown::{IUnknown, IUnknown_Vtbl},
		InterfaceOwned,
	},
	core::{Result, GUID, HRESULT},
	wic::{
		bitmap::{BitmapSource, IWICBitmapSource_Vtbl},
		palette::Palette,
		BitmapDecoderInfo,
		ColorContext,
	},
};
use core::ptr::NonNull;

windows_newtype! {
	pub struct Imaging::WICDecodeOptions(pub i32);
}
impl WICDecodeOptions {
	pub const METADATA_CACHE_ON_DEMAND: WICDecodeOptions = Self(0);
	pub const METADATA_CACHE_ON_LOAD: WICDecodeOptions = Self(1);
}

/// TODO
pub type MetadataQueryReader = IUnknown;

pub trait BitmapDecoderExt {
	fn get_frame(&self, frame: u32) -> Result<BitmapFrameDecode>;
	fn container_format(&self) -> Result<GUID>;
	fn decoder_info(&self) -> Result<BitmapDecoderInfo>;
	fn copy_palette<P: InterfaceAs<Palette>>(&self, palette: &P) -> Result<()>;
	fn metadata_query_reader(&self) -> Result<MetadataQueryReader>;
	fn get_preview(&self) -> Result<BitmapSource>;
	fn get_thumbnail(&self) -> Result<BitmapSource>;
	fn frame_count(&self) -> Result<u32>;
	// TODO: GetColorContexts
}

impl<I: InterfaceAs<BitmapDecoder>> BitmapDecoderExt for I {
	fn get_frame(&self, frame: u32) -> Result<BitmapFrameDecode> {
		let mut out = None;
		let res = unsafe {
			(self.get_parent_vtable().GetFrame)(self.get_parent().as_raw(), frame, &mut out)
		};
		interface_res(res, out)
	}

	fn container_format(&self) -> Result<GUID> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetContainerFormat)
		}
	}

	fn decoder_info(&self) -> Result<BitmapDecoderInfo> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().GetDecoderInfo)
		}
	}

	fn copy_palette<P: InterfaceAs<Palette>>(&self, palette: &P) -> Result<()> {
		unsafe {
			(self.get_parent_vtable().CopyPalette)(self.get_parent().as_raw(), palette.get_parent().as_raw())
		}.ok()
	}

	fn metadata_query_reader(&self) -> Result<MetadataQueryReader> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().GetMetadataQueryReader)
		}
	}

	fn get_preview(&self) -> Result<BitmapSource> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().GetPreview)
		}
	}

	fn get_thumbnail(&self) -> Result<BitmapSource> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().GetThumbnail)
		}
	}

	fn frame_count(&self) -> Result<u32> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetFrameCount)
		}
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct BitmapDecoder {
	pub interface: InterfaceOwned<BitmapDecoder>,
}

unsafe impl Interface for BitmapDecoder {
	/// {9EDDE9E7-8DEE-47EA-99DF-E6FAF2ED44BF}
	const IID: GUID = GUID::from_values(0x9edde9e7, 0x8dee, 0x47ea, [0x99, 0xdf,
		0xe6, 0xfa, 0xf2, 0xed, 0x44, 0xbf,
	]);

	type Owned = Self;
	type Vtable = IWICBitmapDecoder_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for BitmapDecoder {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { BitmapDecoder, IUnknown }

#[repr(C)]
pub struct IWICBitmapDecoder_Vtbl {
	pub base__: IUnknown_Vtbl,
	pub QueryCapability: unsafe extern "system" fn(this: *mut InterfaceTarget, istream: *mut InterfaceTarget, out: *mut u32) -> HRESULT,
	pub Initialize: unsafe extern "system" fn(this: *mut InterfaceTarget, istream: *mut InterfaceTarget, cacheoptions: WICDecodeOptions) -> HRESULT,
	pub GetContainerFormat: GetProperty<GUID>,
	pub GetDecoderInfo: GetInterface<BitmapDecoderInfo>,
	pub CopyPalette: unsafe extern "system" fn(this: *mut InterfaceTarget, palette: *mut InterfaceTarget) -> HRESULT,
	pub GetMetadataQueryReader: GetInterface<MetadataQueryReader>,
	pub GetPreview: GetInterface<BitmapSource>,
	pub GetColorContexts: unsafe extern "system" fn(this: *mut InterfaceTarget, len: u32, contexts: *mut Option<ColorContext>, len_actual: *mut u32) -> HRESULT,
	pub GetThumbnail: GetInterface<BitmapSource>,
	pub GetFrameCount: GetProperty<u32>,
	pub GetFrame: GetPropertyBy<Option<BitmapFrameDecode>, u32>,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct BitmapFrameDecode {
	pub interface: InterfaceOwned<BitmapFrameDecode>,
}

impl BitmapFrameDecode {
	pub const DEFAULT_FRAME: u32 = 0;
}

unsafe impl Interface for BitmapFrameDecode {
	/// {3B16811B-6A43-4EC9-A813-3D930C13B940}
	const IID: GUID = GUID::from_values(0x3b16811b, 0x6a43, 0x4ec9, [0xa8, 0x13,
		0x3d, 0x93, 0x0c, 0x13, 0xb9, 0x40,
	]);

	type Owned = Self;
	type Vtable = IWICBitmapFrameDecode_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for BitmapFrameDecode {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { BitmapFrameDecode, IUnknown, BitmapSource }

#[repr(C)]
pub struct IWICBitmapFrameDecode_Vtbl {
	pub base__: IWICBitmapSource_Vtbl,
	pub GetMetadataQueryReader: GetInterface<MetadataQueryReader>,
	pub GetColorContexts: unsafe extern "system" fn(this: *mut InterfaceTarget, len: u32, contexts: *mut Option<ColorContext>, len_actual: *mut u32) -> HRESULT,
	pub GetThumbnail: GetInterface<BitmapSource>,
}
