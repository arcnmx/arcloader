use crate::windows::{
	core::{Result, GUID},
	Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
	wic::{
		ImagingFactory,
		BitmapSource, BitmapSourceExt,
		PixelFormatInfo, PixelFormatInfoExt, FormatConverterExt, ImagingFactoryExt,
		WICBitmapDitherType, WICBitmapPaletteType,
	},
	winerror,
};
use std::{borrow::Cow, iter, result::Result as StdResult};

impl BitmapSource {
	pub const DXGI_FORMAT_FALLBACK: DXGI_FORMAT = DXGI_FORMAT::R8G8B8A8_UNORM;
	pub const DXGI_PIXFMT_FALLBACK: &'static GUID = &PixelFormatInfo::GUID_32bppRGBA;
	pub fn for_dxgi(&self, dxgifmt: Option<DXGI_FORMAT>, factory: &ImagingFactory) -> Result<Cow<Self>> {
		let pixfmt_id = self.pixel_format()?;
		let pixfmt_dest = match dxgifmt {
			Some(f) => match PixelFormatInfo::guid_from_dxgi(f) {
				Some(dest) => {
					debug_assert_eq!(Some(f), PixelFormatInfo::guid_to_dxgi(dest));
					Some((dest, f))
				},
				None => {
					None
				},
			},
			None => match PixelFormatInfo::guid_to_dxgi(&pixfmt_id) {
				Some(_dxgifmt) => {
					#[cfg(debug_assertions)]
					let pixfmt = factory.pixel_format_info(&pixfmt_id)?;
					#[cfg(debug_assertions)]
					let bpp = pixfmt.bits_per_pixel()? as usize;
					debug_assert_eq!(_dxgifmt.bits_per_pixel(), Some(bpp));
					return Ok(Cow::Borrowed(self))
				},
				None => None,
			},
		};

		let palette = factory.create_palette()?;
		self.copy_palette_to(&palette)?;
		let pixfmts = pixfmt_dest.into_iter()
			.chain(iter::once((Self::DXGI_PIXFMT_FALLBACK, Self::DXGI_FORMAT_FALLBACK)));
		//palette.InitializeCustom(&[])?;
		//let palette = ManuallyDrop::new(IWICPalette::from_raw(ptr::null_mut()));
		let mut conv_res = None;
		for (pixfmt_dest, _dxgifmt) in pixfmts {
			let conv = factory.create_format_converter()?;
			match conv.can_convert(&pixfmt_id, pixfmt_dest) {
				Err(e) => {
					conv_res.get_or_insert(Err(e));
					continue
				}
				Ok(false) => continue,
				Ok(true) => (),
			}
			let res = conv.initialize(self, pixfmt_dest, WICBitmapDitherType::ERROR_DIFFUSION, &palette, 0.0, WICBitmapPaletteType::CUSTOM);
			match res {
				Ok(()) => {
					conv_res = Some(Ok(conv));
					break
				},
				Err(e) => {
					conv_res.get_or_insert(Err(e));
				},
			}
		}
		let conv = conv_res.unwrap_or_else(|| Err(winerror!("no conversion attempted?")))?;

		Ok(Cow::Owned(conv.into()))
	}

	pub fn dxgi_format(&self) -> Result<StdResult<DXGI_FORMAT, GUID>> {
		self.pixel_format ()
			.map(|id| PixelFormatInfo::guid_to_dxgi(&id)
				.ok_or(id)
			)
	}
}

impl PixelFormatInfo {
	pub fn guid_to_dxgi(wic: &GUID) -> Option<DXGI_FORMAT> {
		Some(match *wic {
			PixelFormatInfo::GUID_128bppRGBAFloat => DXGI_FORMAT::R32G32B32A32_FLOAT,
			PixelFormatInfo::GUID_64bppRGBAHalf => DXGI_FORMAT::R16G16B16A16_FLOAT,
			PixelFormatInfo::GUID_32bppRGBA => DXGI_FORMAT::R8G8B8A8_UNORM,
			PixelFormatInfo::GUID_64bppRGBA => DXGI_FORMAT::R16G16B16A16_UNORM,
			PixelFormatInfo::GUID_32bppRGBA1010102 => DXGI_FORMAT::R10G10B10A2_UNORM,
			PixelFormatInfo::GUID_32bppRGBE => 	DXGI_FORMAT::R9G9B9E5_SHAREDEXP,
			PixelFormatInfo::GUID_8bppAlpha => DXGI_FORMAT::A8_UNORM,
			// cfg?
			PixelFormatInfo::GUID_32bppGrayFloat => DXGI_FORMAT::R32_FLOAT,
			PixelFormatInfo::GUID_16bppGrayHalf => DXGI_FORMAT::R16_FLOAT,
			PixelFormatInfo::GUID_16bppGray => DXGI_FORMAT::R16_UNORM,
			PixelFormatInfo::GUID_8bppGray => DXGI_FORMAT::R8_UNORM,
			// cfg(dxgi = "1.1"):
			PixelFormatInfo::GUID_32bppBGRA => DXGI_FORMAT::B8G8R8A8_UNORM,
			PixelFormatInfo::GUID_32bppBGR => DXGI_FORMAT::B8G8R8X8_UNORM,
			PixelFormatInfo::GUID_32bppRGBA1010102XR => DXGI_FORMAT::R10G10B10_XR_BIAS_A2_UNORM,
			// cfg(dxgi = "1.2"):
			PixelFormatInfo::GUID_16bppBGRA5551 => DXGI_FORMAT::B5G5R5A1_UNORM,
			PixelFormatInfo::GUID_16bppBGR565 => DXGI_FORMAT::B5G6R5_UNORM,
			// cfg(windows 8?)
			PixelFormatInfo::GUID_96bppRGBFloat => DXGI_FORMAT::R32G32B32_FLOAT,
			_ => return None,
		})
	}

	pub fn guid_from_dxgi(format: DXGI_FORMAT) -> Option<&'static GUID> {
		Some(match format {
			DXGI_FORMAT::R32G32B32A32_FLOAT => &PixelFormatInfo::GUID_128bppRGBAFloat,
			DXGI_FORMAT::R16G16B16A16_FLOAT => &PixelFormatInfo::GUID_64bppRGBAHalf,
			DXGI_FORMAT::R8G8B8A8_UNORM => &PixelFormatInfo::GUID_32bppRGBA,
			DXGI_FORMAT::R16G16B16A16_UNORM => &PixelFormatInfo::GUID_64bppRGBA,
			DXGI_FORMAT::R10G10B10A2_UNORM => &PixelFormatInfo::GUID_32bppRGBA1010102,
			DXGI_FORMAT::R9G9B9E5_SHAREDEXP => &PixelFormatInfo::GUID_32bppRGBE,
			DXGI_FORMAT::A8_UNORM => &PixelFormatInfo::GUID_8bppAlpha,
			DXGI_FORMAT::R32_FLOAT => &PixelFormatInfo::GUID_32bppGrayFloat,
			DXGI_FORMAT::R16_FLOAT => &PixelFormatInfo::GUID_16bppGrayHalf,
			DXGI_FORMAT::R16_UNORM => &PixelFormatInfo::GUID_16bppGray,
			DXGI_FORMAT::R8_UNORM => &PixelFormatInfo::GUID_8bppGray,
			DXGI_FORMAT::B8G8R8A8_UNORM => &PixelFormatInfo::GUID_32bppBGRA,
			DXGI_FORMAT::B8G8R8X8_UNORM => &PixelFormatInfo::GUID_32bppBGR,
			DXGI_FORMAT::R10G10B10_XR_BIAS_A2_UNORM => &PixelFormatInfo::GUID_32bppRGBA1010102XR,
			DXGI_FORMAT::B5G5R5A1_UNORM => &PixelFormatInfo::GUID_16bppBGRA5551,
			DXGI_FORMAT::B5G6R5_UNORM => &PixelFormatInfo::GUID_16bppBGR565,
			DXGI_FORMAT::R32G32B32_FLOAT => &PixelFormatInfo::GUID_96bppRGBFloat,
			_ => return None,
		})
	}
}
