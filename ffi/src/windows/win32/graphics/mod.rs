pub mod Imaging {
	#[cfg(feature = "wic")]
	pub use crate::windows::wic::{
		WICRect, WICComponentType, WICDdsFormatInfo, WICDecodeOptions, WICBitmapDitherType, WICBitmapPaletteType,
		IWICStream,
		IWICPalette,
		IWICBitmapScaler,
		IWICBitmapSource,
		IWICColorContext,
		IWICBitmapClipper,
		IWICBitmapDecoder,
		IWICBitmapEncoder,
		IWICComponentInfo,
		IWICColorTransform,
		IWICImagingFactory,
		IWICBitmapCodecInfo,
		IWICFormatConverter,
		IWICBitmapDecoderInfo,
		IWICBitmapFlipRotator,
		IWICBitmapFrameDecode,
		IWICMetadataQueryReader,
		stream::IWICStream_Vtbl,
		palette::IWICPalette_Vtbl,
		bitmap::IWICBitmapSource_Vtbl,
		factory::IWICImagingFactory_Vtbl,
		convert::IWICFormatConverter_Vtbl,
		info::{IWICPixelFormatInfo_Vtbl, IWICComponentInfo_Vtbl},
		decode::{IWICBitmapDecoder_Vtbl, IWICBitmapFrameDecode_Vtbl},
	};
}

#[cfg(feature = "dxgi")]
#[path = "dxgi.rs"]
pub mod Dxgi;
