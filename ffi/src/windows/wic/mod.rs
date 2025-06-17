pub mod prelude {
	pub use super::{
		ImagingFactoryExt,
		ComponentInfoExt as _,
		PixelFormatInfoExt as _,
		PaletteExt as _,
		BitmapSourceExt as _,
		BitmapDecoderExt as _,
		FormatConverterExt as _,
		BitmapCodecInfoExt as _,
		StreamExt as _,
	};
}

pub use self::{
	factory::{
		ImagingFactoryExt,
		ImagingFactory, ImagingFactory as IWICImagingFactory,
	},
	info::{
		ComponentInfoExt, PixelFormatInfoExt, BitmapCodecInfoExt,
		ComponentInfo, ComponentInfo as IWICComponentInfo,
		PixelFormatInfo, PixelFormatInfo as IWICPixelFormatInfo,
		BitmapCodecInfo, BitmapCodecInfo as IWICBitmapCodecInfo,
		BitmapDecoderInfo, BitmapDecoderInfo as IWICBitmapDecoderInfo,
		WICComponentType,
	},
	decode::{
		BitmapDecoderExt,
		BitmapDecoder, BitmapDecoder as IWICBitmapDecoder,
		BitmapFrameDecode, BitmapFrameDecode as IWICBitmapFrameDecode,
		MetadataQueryReader, MetadataQueryReader as IWICMetadataQueryReader,
		WICDecodeOptions,
	},
	encode::{
		BitmapEncoder, BitmapEncoder as IWICBitmapEncoder,
	},
	bitmap::{
		BitmapSourceExt,
		BitmapSource, BitmapSource as IWICBitmapSource,
		BitmapScaler, BitmapScaler as IWICBitmapScaler,
		BitmapClipper, BitmapClipper as IWICBitmapClipper,
		BitmapFlipRotator, BitmapFlipRotator as IWICBitmapFlipRotator,
	},
	convert::{
		FormatConverterExt,
		FormatConverter, FormatConverter as IWICFormatConverter,
		ColorTransform, ColorTransform as IWICColorTransform,
		WICBitmapDitherType,
	},
	stream::{
		StreamExt,
		Stream, Stream as IWICStream,
		BitmapFrameBytes,
	},
	palette::{
		PaletteExt,
		Palette, Palette as IWICPalette,
		ColorContext, ColorContext as IWICColorContext,
		WICBitmapPaletteType,
	},
	dds::{
		DdsFrameDecode, DdsFrameDecode as IWICDdsFrameDecode,
		WICDdsFormatInfo,
	},
};

pub mod factory;
pub mod bitmap;
pub mod convert;
pub mod decode;
pub mod encode;
pub mod dds;
#[cfg(feature = "dxgi")]
pub mod dxgi;
pub mod info;
pub mod palette;
pub mod stream;

use crate::windows::adapter::windows_newtype;

#[repr(C)]
pub struct WICRect {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
}

windows_newtype! {
	impl From for Imaging::WICRect(pub Self);
}

#[cfg(feature = "windows-060")]
pub mod wic060 {
	use crate::windows::{
		adapter::windows_newtype,
		core060::InterfaceRef,
		core::Result,
		wic::{self, ImagingFactory},
		Win32_060::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
	};

	pub use crate::windows::Win32_060::Graphics::Imaging::{
		IWICImagingFactory,
		IWICBitmapDecoder, IWICBitmapSource, IWICBitmapFrameDecode,
		IWICComponentInfo,
		IWICPalette,
		IWICStream,
		CLSID_WICImagingFactory,
	};

	pub fn new_factory() -> Result<IWICImagingFactory> {
		unsafe {
			CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
		}.map_err(Into::into)
	}

	impl ImagingFactory {
		pub fn factory_060(&self) -> InterfaceRef<IWICImagingFactory> {
			unsafe {
				InterfaceRef::from_raw(self.interface.raw())
			}
		}
	}

	windows_newtype! {
		impl From@transparent_imp for ImagingFactory{IWICImagingFactory};
	}
	windows_newtype! {
		impl From@transparent_imp for wic::ComponentInfo{IWICComponentInfo};
	}
	windows_newtype! {
		impl From@transparent_imp for wic::Palette{IWICPalette};
	}
	windows_newtype! {
		impl From@transparent_imp for wic::Stream{IWICStream};
	}
	windows_newtype! {
		impl From@transparent_imp for wic::BitmapDecoder{IWICBitmapDecoder};
	}
	windows_newtype! {
		impl From@transparent_imp for wic::BitmapSource{IWICBitmapSource};
	}
	windows_newtype! {
		impl From@transparent_imp for wic::BitmapFrameDecode{IWICBitmapFrameDecode};
	}

	#[cfg(feature = "resources")]
	pub type ImageLoader = super::ImageLoader<IWICImagingFactory>;
	#[cfg(feature = "resources")]
	pub type ImageLoaderRef<'a> = super::ImageLoader<InterfaceRef<'a, IWICImagingFactory>>;

	#[cfg(feature = "resources")]
	impl ResourceLoader for ImageLoaderRef<'_> {
		type Err = Error;
		type Output = IWICBitmapSource;
	}
}

#[cfg(feature = "windows-061")]
pub mod wic061 {
	use crate::windows::{
		Win32_061::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
		Result,
	};

	pub use crate::windows::Win32_061::Graphics::Imaging::{IWICImagingFactory, CLSID_WICImagingFactory};

	pub fn new_factory() -> Result<IWICImagingFactory> {
		unsafe {
			CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
		}
	}
}

#[test]
fn fallback_decode() {
	use crate::windows::{
		com::{com_init_mta, com_uninit},
		wic::{
			prelude::*,
			ImagingFactory, BitmapFrameDecode, BitmapSource,
		},
	};
	fn assert_image(image: &BitmapSource) {
		let (width, height) = image.size().unwrap();
		assert_ne!(width, 0);
		assert_ne!(height, 0);
	}

	let fallback = &include_bytes!("../../../../addonapi/src/host/textures/fallback-texture.bin")[..];

	unsafe {
		com_init_mta().unwrap();
	}
	let loader = ImagingFactory::new().unwrap();
	let image = {
		let decoder = loader.decode_bytes_static(&fallback, None, None).unwrap();
		let frames = decoder.frame_count().unwrap();
		assert_ne!(frames, 0);
		decoder.get_frame(BitmapFrameDecode::DEFAULT_FRAME).unwrap()
	};
	assert_image(&image);

	#[cfg(feature = "dxgi")] {
		let image_dxgi = image.for_dxgi(None, &loader).unwrap();
		assert_image(&image_dxgi);
	}

	drop(image);
	drop(loader);

	unsafe {
		com_uninit();
	}
}
