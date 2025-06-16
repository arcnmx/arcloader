use crate::{
	cstr::{c_wchar, CStrPtr16},
	windows::{
		com::{
			imp::{interface_res, get_interface, interface_hierarchy, GetInterface, GetPropertyBy, Unimplemented},
			interface::{Interface, InterfacePtr, InterfaceAs, InterfaceTarget},
			stream::IStream,
			unknown::IUnknown_Vtbl,
			InterfaceOwned,
		},
		core::{Result, GUID, HRESULT, PCWSTR},
		wic::{
			self,
			BitmapFrameBytes,
			BitmapDecoder, BitmapEncoder,
			BitmapClipper, BitmapScaler, BitmapFlipRotator,
			Stream, StreamExt,
			Palette, FormatConverter,
			ColorContext, ColorTransform,
			ComponentInfo, PixelFormatInfo,
			WICDecodeOptions,
		},
		winerror,
		Win32::Foundation::{GENERIC_ACCESS_RIGHTS, GENERIC_READ},
	},
};
use std::{borrow::Cow, os::windows::ffi::OsStrExt, path::Path, ptr::{self, NonNull}};

pub trait ImagingFactoryExt {
	fn component_info(&self, id: &GUID) -> Result<ComponentInfo>;

	fn pixel_format_info<G: AsRef<GUID>>(&self, id: &G) -> Result<PixelFormatInfo> {
		self.component_info(id.as_ref())
			.and_then(|info| info.cast::<PixelFormatInfo>())
			.map(Interface::to_canon)
	}

	fn create_stream(&self) -> Result<Stream>;
	fn create_palette(&self) -> Result<Palette>;
	fn create_format_converter(&self) -> Result<FormatConverter>;

	fn create_decoder_from_stream<S: InterfaceAs<IStream>>(&self, stream: &S, vendor: Option<&GUID>, metadata: WICDecodeOptions) -> Result<BitmapDecoder>;
	fn create_decoder_from_filename(&self, filename: CStrPtr16, vendor: Option<&GUID>, access: GENERIC_ACCESS_RIGHTS, metadata: WICDecodeOptions) -> Result<BitmapDecoder>;
	unsafe fn create_decoder_from_file_handle(&self, handle: usize, vendor: Option<&GUID>, metadata: WICDecodeOptions) -> Result<BitmapDecoder>;
}

impl<I: InterfaceAs<ImagingFactory>> ImagingFactoryExt for I {
	fn component_info(&self, id: &GUID) -> Result<ComponentInfo> {
		let mut out = None;
		let res = unsafe {
			(self.get_parent_vtable().CreateComponentInfo)(self.as_raw(), id, &mut out)
		};
		interface_res(res, out)
	}

	fn create_stream(&self) -> Result<Stream> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().CreateStream)
		}
	}

	fn create_palette(&self) -> Result<Palette> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().CreatePalette)
		}
	}

	fn create_format_converter(&self) -> Result<FormatConverter> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().CreateFormatConverter)
		}
	}

	fn create_decoder_from_stream<S: InterfaceAs<IStream>>(&self, stream: &S, vendor: Option<&GUID>, metadata: WICDecodeOptions) -> Result<BitmapDecoder> {
		let vendor = vendor.map(|v| v as *const GUID).unwrap_or(ptr::null());
		let stream = stream.get_parent();

		let mut out = None;
		let res = unsafe {
			(self.get_parent_vtable().CreateDecoderFromStream)(self.get_parent().as_raw(), stream.as_raw(), vendor, metadata, &mut out)
		};
		interface_res(res, out)
	}

	fn create_decoder_from_filename(&self, filename: CStrPtr16, vendor: Option<&GUID>, access: GENERIC_ACCESS_RIGHTS, metadata: WICDecodeOptions) -> Result<BitmapDecoder> {
		let vendor = vendor.map(|v| v as *const GUID).unwrap_or(ptr::null());

		let mut out = None;
		let res = unsafe {
			let filename = filename.immortal();
			(self.get_parent_vtable().CreateDecoderFromFilename)(self.get_parent().as_raw(), Some(filename), vendor, access, metadata, &mut out)
		};
		interface_res(res, out)
	}

	unsafe fn create_decoder_from_file_handle(&self, handle: usize, vendor: Option<&GUID>, metadata: WICDecodeOptions) -> Result<BitmapDecoder> {
		let vendor = vendor.map(|v| v as *const GUID).unwrap_or(ptr::null());

		let mut out = None;
		let res = unsafe {
			(self.get_parent_vtable().CreateDecoderFromFileHandle)(self.get_parent().as_raw(), handle, vendor, metadata, &mut out)
		};
		interface_res(res, out)
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ImagingFactory {
	pub interface: InterfaceOwned<ImagingFactory>,
}

impl ImagingFactory {
	/// {CACAF262-9370-4615-A13B-9F5539DA4C0A}
	pub const CLSID: GUID = GUID::from_values(0xCACAF262, 0x9370, 0x4615, [0xA1, 0x3B,
		0x9F, 0x55, 0x39, 0xDA, 0x4C, 0x0A,
	]);

	pub fn new() -> Result<Self> {
		#[allow(unreachable_patterns)]
		match () {
			#[cfg(feature = "windows-061")]
			_ => wic::wic061::new_factory().map_err(Into::into).map(Into::into),
			#[cfg(feature = "windows-060")]
			_ => wic::wic060::new_factory().map_err(Into::into).map(Into::into),
			_ => Err(winerror!()),
		}
	}

	pub const DECODE_OPTIONS_DEFAULT: WICDecodeOptions = WICDecodeOptions::METADATA_CACHE_ON_DEMAND;

	pub fn decode_stream<S: InterfaceAs<IStream>>(&self, stream: &S, vendor: Option<&GUID>, opts: Option<WICDecodeOptions>) -> Result<BitmapDecoder> {
		let opts = opts.unwrap_or(Self::DECODE_OPTIONS_DEFAULT);
		self.create_decoder_from_stream(stream, vendor, opts)
	}

	pub fn decode_filename(&self, filename: CStrPtr16, vendor: Option<&GUID>, opts: Option<WICDecodeOptions>) -> Result<BitmapDecoder> {
		let opts = opts.unwrap_or(Self::DECODE_OPTIONS_DEFAULT);
		self.create_decoder_from_filename(filename, vendor, GENERIC_READ, opts)
	}

	pub fn decode_path<P: AsRef<Path>>(&self, path: P, vendor: Option<&GUID>, opts: Option<WICDecodeOptions>) -> Result<BitmapDecoder> {
		let path = path.as_ref();
		let mut fname: Vec<c_wchar> = path.as_os_str().encode_wide().collect();
		fname.push(0);
		let filename = unsafe {
			CStrPtr16::new(NonNull::new_unchecked(fname.as_ptr() as *mut _))
		};
		let opts = opts.unwrap_or(Self::DECODE_OPTIONS_DEFAULT);
		let res = self.create_decoder_from_filename(filename, vendor, GENERIC_READ, opts);
		drop(fname);
		res
	}

	pub unsafe fn stream_for_bytes<'b>(&self, data: &'b [u8]) -> Result<Stream> {
		let stream = self.create_stream()?;
		stream.initialize_from_memory(data)?;
		Ok(stream)
	}

	pub fn decode_bytes_frame<'b, B: Into<Cow<'b, [u8]>>>(&self, data: B, vendor: Option<&GUID>, opts: Option<WICDecodeOptions>, frame: u32) -> Result<BitmapFrameBytes<'b>> {
		BitmapFrameBytes::decode_frame(self, data, vendor, opts, frame)
	}

	pub unsafe fn decode_bytes_unchecked<'b>(&self, data: &'b [u8], vendor: Option<&GUID>, opts: Option<WICDecodeOptions>) -> Result<BitmapDecoder> {
		let stream = self.create_stream()?;
		stream.initialize_from_memory(data)?;
		self.decode_stream(&stream, vendor, opts)
	}

	pub fn decode_bytes_static(&self, data: &'static [u8], vendor: Option<&GUID>, opts: Option<WICDecodeOptions>) -> Result<BitmapDecoder> {
		unsafe {
			self.decode_bytes_unchecked(data, vendor, opts)
		}
	}
}

unsafe impl Interface for ImagingFactory {
	/// {EC5EC8A9-C395-4314-9C77-54D7A935FF70}
	const IID: GUID = GUID::from_values(0xec5ec8a9, 0xc395, 0x4314, [0x9c, 0x77,
		0x54, 0xd7, 0xa9, 0x35, 0xff, 0x70,
	]);

	type Owned = Self;
	type Vtable = IWICImagingFactory_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for ImagingFactory {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { ImagingFactory, IUnknown }

pub type IWICImagingFactory_CreateComponentInfo = GetPropertyBy<Option<ComponentInfo>, *const GUID>;
pub type IWICImagingFactory_CreateDecoderFromStream = unsafe extern "system" fn(this: *mut InterfaceTarget, stream: *mut InterfaceTarget, vendor: *const GUID, metadataoptions: WICDecodeOptions, out: *mut Option<BitmapDecoder>) -> HRESULT;
pub type IWICImagingFactory_CreateDecoderFromFileHandle = unsafe extern "system" fn(this: *mut InterfaceTarget, handle: usize, vendor: *const GUID, metadataoptions: WICDecodeOptions, out: *mut Option<BitmapDecoder>) -> HRESULT;
pub type IWICImagingFactory_CreateDecoderFromFilename = unsafe extern "system" fn(this: *mut InterfaceTarget, filename: PCWSTR, vendor: *const GUID, access: GENERIC_ACCESS_RIGHTS, metadataoptions: WICDecodeOptions, out: *mut Option<BitmapDecoder>) -> HRESULT;
pub type IWICImagingFactory_CreateCodec<I> = unsafe extern "system" fn(this: *mut InterfaceTarget, format: *const GUID, vendor: *const GUID, out: *mut Option<I>) -> HRESULT;

#[repr(C)]
pub struct IWICImagingFactory_Vtbl {
	pub base__: IUnknown_Vtbl,
	pub CreateDecoderFromFilename: IWICImagingFactory_CreateDecoderFromFilename,
	pub CreateDecoderFromStream: IWICImagingFactory_CreateDecoderFromStream,
	pub CreateDecoderFromFileHandle: IWICImagingFactory_CreateDecoderFromFileHandle,
	pub CreateComponentInfo: IWICImagingFactory_CreateComponentInfo,
	pub CreateDecoder: IWICImagingFactory_CreateCodec<BitmapDecoder>,
	pub CreateEncoder: IWICImagingFactory_CreateCodec<BitmapEncoder>,
	pub CreatePalette: GetInterface<Palette>,
	pub CreateFormatConverter: GetInterface<FormatConverter>,
	pub CreateBitmapScaler: GetInterface<BitmapScaler>,
	pub CreateBitmapClipper: GetInterface<BitmapClipper>,
	pub CreateBitmapFlipRotator: GetInterface<BitmapFlipRotator>,
	pub CreateStream: GetInterface<Stream>,
	pub CreateColorContext: GetInterface<ColorContext>,
	pub CreateColorTransformer: GetInterface<ColorTransform>,
	pub CreateBitmap: Unimplemented,
	pub CreateBitmapFromSource: Unimplemented,
	pub CreateBitmapFromSourceRect: Unimplemented,
	pub CreateBitmapFromMemory: Unimplemented,
	pub CreateBitmapFromHBITMAP: usize,
	pub CreateBitmapFromHICON: Unimplemented,
	pub CreateComponentEnumerator: Unimplemented,
	pub CreateFastMetadataEncoderFromDecoder: Unimplemented,
	pub CreateFastMetadataEncoderFromFrameDecode: Unimplemented,
	pub CreateQueryWriter: Unimplemented,
	pub CreateQueryWriterFromReader: Unimplemented,
}
