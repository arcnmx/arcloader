use crate::{
	extensions::nexus::NexusHost,
	util::{ffi::{cstr_opt, nonnull_ref}, win::{find_resource, WinError, WinResult, MAKERESOURCEA}},
};
use nexus::texture::{RawTextureReceiveCallback, Texture};
use windows::{core::{Interface, Param, GUID}, Win32::{Foundation::{ERROR_CREATE_FAILED, ERROR_INVALID_HANDLE, ERROR_INVALID_PIXEL_FORMAT, GENERIC_READ, HMODULE}, Graphics::{Direct3D::D3D11_SRV_DIMENSION_TEXTURE2D, Direct3D11::{ID3D11DeviceContext, ID3D11ShaderResourceView, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_RESOURCE_MISC_GENERATE_MIPS, D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_SHADER_RESOURCE_VIEW_DESC_0, D3D11_SUBRESOURCE_DATA, D3D11_TEX2D_SRV, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT}, Dxgi::Common::{DXGI_FORMAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC}, Imaging::{self as wic, CLSID_WICImagingFactory, IWICBitmapFrameDecode, IWICBitmapSource, IWICImagingFactory, IWICPixelFormatInfo, WICBitmapDitherTypeErrorDiffusion, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand, WICRect}}, System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER}}};
use windows_strings::{HSTRING, PCWSTR};
use std::{borrow::Cow, collections::{hash_map::Entry, HashMap}, ffi::{c_char, c_void, CStr, CString}, mem::transmute, ptr::{self, NonNull}, sync::{Arc, LazyLock, Once, RwLock, TryLockError}};

#[derive(Clone, Debug)]
pub struct TextureLoaderWic {
	pub factory: IWICImagingFactory,
}

unsafe impl Sync for TextureLoaderWic {
}

// https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-resources-textures-how-to
impl TextureLoaderWic {
	pub fn new() -> WinResult<Self> {
		let factory = unsafe {
			CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
		}?;
		Ok(Self {
			factory,
		})
	}

	/// TODO: move this into a field on host
	pub fn loader() -> WinResult<&'static Self> {
		thread_local! {
			static TEXTURE_LOADER_WIC: WinResult<TextureLoaderWic> = TextureLoaderWic::new();
		}

		TEXTURE_LOADER_WIC
			.with(|loader| match loader {
				// XXX: kinda okay because it's not Send? .-.
				Ok(l) => Ok(unsafe { transmute(l) }),
				Err(e) => Err(e.clone()),
			})
	}


	pub fn pixfmt(&self, wicfmt: &GUID) -> WinResult<IWICPixelFormatInfo> {
		unsafe {
			let info = self.factory.CreateComponentInfo(wicfmt)?;
			match info.GetComponentType()? {
				wic::WICPixelFormat => info.cast(),
				_ => Err(WinError::new(ERROR_INVALID_PIXEL_FORMAT.to_hresult(), format!("{wicfmt:?} is not a WICPixelFormat"))),
			}
		}
	}

	pub fn pixfmt_name(pixfmt: &IWICPixelFormatInfo) -> WinResult<HSTRING> {
		let mut name = [0u16; 64];
		let mut len: u32 = name.len() as u32;
		unsafe {
			pixfmt.GetFriendlyName(&mut name, &mut len)
		}?;

		let len = (len as usize).min(name.len());
		let name = &name[..len];
		Ok(HSTRING::from_wide(name))
	}

	pub fn decode_file_frame<P: Param<PCWSTR>>(&self, path: P, frame_index: Option<u32>) -> WinResult<IWICBitmapFrameDecode> {
		unsafe {
			let decoder = self.factory.CreateDecoderFromFilename(path, None, GENERIC_READ, WICDecodeMetadataCacheOnDemand)?;
			decoder.GetFrame(frame_index.unwrap_or(0))
		}
	}

	pub fn decode_memory_frame(&self, data: &[u8], frame_index: Option<u32>) -> WinResult<IWICBitmapFrameDecode> {
		unsafe {
			let stream = self.factory.CreateStream()?;
			stream.InitializeFromMemory(data)?;
			let decoder = self.factory.CreateDecoderFromStream(&stream, ptr::null(), WICDecodeMetadataCacheOnDemand)?;
			decoder.GetFrame(frame_index.unwrap_or(0))
		}
	}

	pub fn decode_resource_frame(&self, module: HMODULE, resource: u32, frame_index: Option<u32>) -> WinResult<IWICBitmapFrameDecode> {
		let resource_id = resource.try_into()
			.map_err(|e| WinError::new(ERROR_INVALID_HANDLE.to_hresult(), format!("resource ID {resource} out of range: {e}")))?;
		let data = unsafe {
			find_resource(&module, MAKERESOURCEA(resource_id), windows_strings::s!("PNG"))
		}?;
		self.decode_memory_frame(data, frame_index)
	}

	pub fn pixfmt_to_dxgi(wic: &GUID) -> Option<DXGI_FORMAT> {
		use windows::Win32::Graphics::Dxgi::Common::*;

		Some(match *wic {
			wic::GUID_WICPixelFormat128bppRGBAFloat => DXGI_FORMAT_R32G32B32A32_FLOAT,
			wic::GUID_WICPixelFormat64bppRGBAHalf => DXGI_FORMAT_R16G16B16A16_FLOAT,
			wic::GUID_WICPixelFormat32bppRGBA => DXGI_FORMAT_R8G8B8A8_UNORM,
			wic::GUID_WICPixelFormat64bppRGBA => DXGI_FORMAT_R16G16B16A16_UNORM,
			wic::GUID_WICPixelFormat32bppRGBA1010102 => DXGI_FORMAT_R10G10B10A2_UNORM,
			wic::GUID_WICPixelFormat32bppRGBE => 	DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
			wic::GUID_WICPixelFormat8bppAlpha => DXGI_FORMAT_A8_UNORM,
			// cfg?
			wic::GUID_WICPixelFormat32bppGrayFloat => DXGI_FORMAT_R32_FLOAT,
			wic::GUID_WICPixelFormat16bppGrayHalf => DXGI_FORMAT_R16_FLOAT,
			wic::GUID_WICPixelFormat16bppGray => DXGI_FORMAT_R16_UNORM,
			wic::GUID_WICPixelFormat8bppGray => DXGI_FORMAT_R8_UNORM,
			// cfg(dxgi = "1.1"):
			wic::GUID_WICPixelFormat32bppBGRA => DXGI_FORMAT_B8G8R8A8_UNORM,
			wic::GUID_WICPixelFormat32bppBGR => DXGI_FORMAT_B8G8R8X8_UNORM,
			wic::GUID_WICPixelFormat32bppRGBA1010102XR => DXGI_FORMAT_R10G10B10_XR_BIAS_A2_UNORM,
			// cfg(dxgi = "1.2"):
			wic::GUID_WICPixelFormat16bppBGRA5551 => DXGI_FORMAT_B5G5R5A1_UNORM,
			wic::GUID_WICPixelFormat16bppBGR565 => DXGI_FORMAT_B5G6R5_UNORM,
			// cfg(windows 8?)
			wic::GUID_WICPixelFormat96bppRGBFloat => DXGI_FORMAT_R32G32B32_FLOAT,
			_ => return None,
		})
	}

	pub fn dxgi_pixfmt_bpp(pixfmt: DXGI_FORMAT) -> WinResult<usize> {
		use windows::Win32::Graphics::Dxgi::Common::*;

		Ok(match pixfmt {
			DXGI_FORMAT_R32G32B32A32_FLOAT | DXGI_FORMAT_R32G32B32A32_SINT | DXGI_FORMAT_R32G32B32A32_UINT | DXGI_FORMAT_R32G32B32A32_TYPELESS
				=> 128,
			DXGI_FORMAT_R32G32B32_FLOAT | DXGI_FORMAT_R32G32B32_SINT | DXGI_FORMAT_R32G32B32_UINT | DXGI_FORMAT_R32G32B32_TYPELESS
				=> 96,
			DXGI_FORMAT_R16G16B16A16_FLOAT | DXGI_FORMAT_R16G16B16A16_SINT | DXGI_FORMAT_R16G16B16A16_UINT | DXGI_FORMAT_R16G16B16A16_SNORM | DXGI_FORMAT_R16G16B16A16_UNORM | DXGI_FORMAT_R16G16B16A16_TYPELESS
			| DXGI_FORMAT_R32G32_FLOAT | DXGI_FORMAT_R32G32_SINT | DXGI_FORMAT_R32G32_UINT | DXGI_FORMAT_R32G32_TYPELESS
			| DXGI_FORMAT_R32G8X24_TYPELESS | DXGI_FORMAT_D32_FLOAT_S8X24_UINT | DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS | DXGI_FORMAT_X32_TYPELESS_G8X24_UINT
			| DXGI_FORMAT_Y416 | DXGI_FORMAT_Y210 | DXGI_FORMAT_Y216
				=> 64,
			DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB | DXGI_FORMAT_R8G8B8A8_SINT | DXGI_FORMAT_R8G8B8A8_UINT | DXGI_FORMAT_R8G8B8A8_TYPELESS
			| DXGI_FORMAT_B8G8R8A8_UNORM | DXGI_FORMAT_R8G8B8A8_SNORM | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB | DXGI_FORMAT_B8G8R8A8_TYPELESS
			| DXGI_FORMAT_B8G8R8X8_UNORM | DXGI_FORMAT_B8G8R8X8_UNORM_SRGB
			| DXGI_FORMAT_R32_SINT | DXGI_FORMAT_R32_UINT | DXGI_FORMAT_R32_FLOAT | DXGI_FORMAT_R32_TYPELESS
			| DXGI_FORMAT_R24G8_TYPELESS | DXGI_FORMAT_D24_UNORM_S8_UINT | DXGI_FORMAT_R24_UNORM_X8_TYPELESS | DXGI_FORMAT_X24_TYPELESS_G8_UINT
			| DXGI_FORMAT_R16G16_FLOAT | DXGI_FORMAT_R16G16_UNORM | DXGI_FORMAT_R16G16_SNORM | DXGI_FORMAT_R16G16_SINT | DXGI_FORMAT_R16G16_UINT | DXGI_FORMAT_R16G16_TYPELESS
			| DXGI_FORMAT_R10G10B10A2_TYPELESS | DXGI_FORMAT_R10G10B10A2_UNORM | DXGI_FORMAT_R10G10B10A2_UINT
			| DXGI_FORMAT_R11G11B10_FLOAT
			| DXGI_FORMAT_R9G9B9E5_SHAREDEXP
			| DXGI_FORMAT_R10G10B10_XR_BIAS_A2_UNORM
			| DXGI_FORMAT_AYUV | DXGI_FORMAT_Y410 | DXGI_FORMAT_YUY2
				=> 32,
			DXGI_FORMAT_R16_SINT | DXGI_FORMAT_R16_UINT | DXGI_FORMAT_R16_UNORM | DXGI_FORMAT_R16_SNORM | DXGI_FORMAT_R16_FLOAT | DXGI_FORMAT_R16_TYPELESS
			| DXGI_FORMAT_R8G8_SINT | DXGI_FORMAT_R8G8_UINT | DXGI_FORMAT_R8G8_SNORM | DXGI_FORMAT_R8G8_UNORM
			| DXGI_FORMAT_R8G8_B8G8_UNORM | DXGI_FORMAT_G8R8_G8B8_UNORM
			| DXGI_FORMAT_B5G6R5_UNORM | DXGI_FORMAT_B5G5R5A1_UNORM
			| DXGI_FORMAT_B4G4R4A4_UNORM | DXGI_FORMAT_A4B4G4R4_UNORM
				=> 16,
			DXGI_FORMAT_R8_SINT | DXGI_FORMAT_R8_UINT | DXGI_FORMAT_R8_UNORM | DXGI_FORMAT_R8_SNORM | DXGI_FORMAT_R8_TYPELESS
			| DXGI_FORMAT_A8_UNORM
				=> 8,
			| DXGI_FORMAT_R1_UNORM
				=> 1,
			// block compression: DXGI_FORMAT_BC*
			// planar and mpeg bs:
			// 16 bpc:  DXGI_FORMAT_P016
			// 10bpc: DXGI_FORMAT_P010
			//.8bpc:  DXGI_FORMAT_420_OPAQUE
			//DXGI_FORMAT_P208
			//DXGI_FORMAT_V208 | DXGI_FORMAT_V408
			//DXGI_FORMAT_SAMPLER_FEEDBACK_MIN_MIP_OPAQUE | DXGI_FORMAT_SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE
			// DXGI_FORMAT_NV12 | DXGI_FORMAT_NV11
			// DXGI_FORMAT_AI44 | DXGI_FORMAT_IA44 | DXGI_FORMAT_P8 | DXGI_FORMAT_A8P8
			_ => return Err(WinError::new(ERROR_INVALID_PIXEL_FORMAT.to_hresult(), format!("unknown bpp for {pixfmt:?}"))),
		})
	}

	pub fn dxgi_pixfmt_supports_mipmaps(fmt: DXGI_FORMAT) -> bool {
		use windows::Win32::Graphics::Dxgi::Common::*;

		match fmt {
			// feature level 9.1 to 9.3:
			DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB | DXGI_FORMAT_B5G6R5_UNORM | DXGI_FORMAT_B8G8R8A8_UNORM | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB | DXGI_FORMAT_B8G8R8X8_UNORM | DXGI_FORMAT_B8G8R8X8_UNORM_SRGB
				=> true,
			// feature level 9.2:
			DXGI_FORMAT_R16G16B16A16_FLOAT | DXGI_FORMAT_R16G16B16A16_UNORM | DXGI_FORMAT_R16G16_FLOAT | DXGI_FORMAT_R16G16_UNORM | DXGI_FORMAT_R32_FLOAT
				=> true,
			// feature level 9.3:
			DXGI_FORMAT_R32G32B32A32_FLOAT
			// optional? | DXGI_FORMAT_B4G4R4A4
				=> true,
			// feature level 10:
			DXGI_FORMAT_R16G16B16A16_SNORM | DXGI_FORMAT_R32G32_FLOAT | DXGI_FORMAT_R10G10B10A2_UNORM | DXGI_FORMAT_R11G11B10_FLOAT | DXGI_FORMAT_R8G8B8A8_SNORM | DXGI_FORMAT_R16G16_SNORM | DXGI_FORMAT_R8G8_UNORM | DXGI_FORMAT_R8G8_SNORM | DXGI_FORMAT_R16_FLOAT | DXGI_FORMAT_R16_UNORM | DXGI_FORMAT_R16_SNORM | DXGI_FORMAT_R8_UNORM | DXGI_FORMAT_R8_SNORM | DXGI_FORMAT_A8_UNORM
			// optional? | DXGI_FORMAT_R32G32B32_FLOAT | DXGI_FORMAT_B5G5R5A1_UNORM
				=> true,
			_ => false,
		}
	}
}

#[derive(Debug, Clone)]
pub struct WicImage {
	pub image: IWICBitmapSource,
}

impl WicImage {
	pub fn new<I: Into<IWICBitmapSource>>(image: I) -> Self {
		let image = image.into();
		Self {
			image,
		}
	}

	pub fn pixfmt(&self) -> WinResult<IWICPixelFormatInfo> {
		let fmt = unsafe {
			self.image.GetPixelFormat()
		}?;
		TextureLoaderWic::loader()?.pixfmt(&fmt)
	}

	pub fn rect(&self) -> WinResult<WICRect> {
		let (mut w, mut h) = (0, 0);
		unsafe {
			self.image.GetSize(&mut w, &mut h)
		}.map(|()| WICRect {
			X: 0,
			Y: 0,
			Width: w as _,
			Height: h as _,
		})
	}

	pub fn size_info(&self) -> WinResult<(usize, u32)> {
		let WICRect { Width: w, Height: h, .. } = self.rect()?;
		let bpp = unsafe {
			self.pixfmt()?.GetBitsPerPixel()
		}?;
		let stride = (w as u32 * bpp + 7) / 8;
		Ok((
			stride as usize * h as usize,
			stride,
		))
	}

	pub fn to_data(&self) -> WinResult<Box<[u8]>> {
		trace!("WicImage::to_data()");
		let (size, stride) = self.size_info()?;
		Ok(unsafe {
			let mut buffer: Vec<u8> = Vec::with_capacity(size);
			let dest = buffer.spare_capacity_mut() as *mut [_] as *mut [u8];
			self.image.CopyPixels(ptr::null(), stride, &mut *dest)?;
			buffer.set_len(size);
			buffer.into_boxed_slice()
		})
	}

	pub const DXGI_FORMAT_FALLBACK: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;
	pub fn for_dxgi(&self, pixfmt11: Option<DXGI_FORMAT>) -> WinResult<Cow<Self>> {
		let pixfmt_id = unsafe { self.image.GetPixelFormat()? };
		let pixfmt11 = match pixfmt11 {
			Some(f) => f,
			None if TextureLoaderWic::pixfmt_to_dxgi(&pixfmt_id).is_some() =>
				return Ok(Cow::Borrowed(self)),
			None => Self::DXGI_FORMAT_FALLBACK,
		};

		debug!("Converting from {pixfmt_id:?} to {pixfmt11:?}");
		let loader = TextureLoaderWic::loader()?;
		let conv = unsafe {
			loader.factory.CreateFormatConverter()
		}?;
		let palette = unsafe {
			loader.factory.CreatePalette()
		}?;
		unsafe {
			self.image.CopyPalette(&palette)
		}?;
		//palette.InitializeCustom(&[])?;
		//let palette = ManuallyDrop::new(IWICPalette::from_raw(ptr::null_mut()));
		unsafe {
			conv.Initialize(&self.image, &wic::GUID_WICPixelFormat32bppRGBA, WICBitmapDitherTypeErrorDiffusion, &palette, 0.0, WICBitmapPaletteTypeCustom)
		}?;

		Ok(Cow::Owned(Self::new(conv)))
	}

	pub fn describe_d3d11(&self) -> WinResult<D3D11_TEXTURE2D_DESC> {
		let pixfmt = self.pixfmt()?;
		// let pixfmt_id = unsafe { pixfmt.GetFormatGUID()? };
		let pixfmt_id = unsafe { self.image.GetPixelFormat()? };
		let pixfmt11 = TextureLoaderWic::pixfmt_to_dxgi(&pixfmt_id)
			.ok_or_else(|| {
				let name = TextureLoaderWic::pixfmt_name(&pixfmt)
					.ok();
				let name = name.as_ref()
					.map(|name| PCWSTR::from_raw(name.as_ptr()));
				let msg = match name {
					Some(name) => format!("unknown DXGI format for {}", unsafe { name.display() }),
					None => format!("unknown DXGI format for {pixfmt_id:?}"),
				};
				WinError::new(ERROR_INVALID_PIXEL_FORMAT.to_hresult(), msg)
			})?;
		debug_assert_eq!(TextureLoaderWic::dxgi_pixfmt_bpp(pixfmt11).ok(), unsafe { pixfmt.GetBitsPerPixel() }.ok().map(|bpp| bpp as usize));

		let gen_mips = match TextureLoaderWic::dxgi_pixfmt_supports_mipmaps(pixfmt11) {
			#[cfg(todo)]
			true => true,
			_ => false,
		};

		let WICRect { Width: w, Height: h, .. } = self.rect()?;
		let desc = D3D11_TEXTURE2D_DESC {
			Width: w as u32,
			Height: h as u32,
			MipLevels: if gen_mips { 0 } else { 1 },
			ArraySize: 1,
			Format: pixfmt11,
			SampleDesc: DXGI_SAMPLE_DESC {
				Count: 1,
				Quality: 0,
			},
			Usage: D3D11_USAGE_DEFAULT,
			BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32 | if gen_mips { D3D11_BIND_RENDER_TARGET.0 as u32 } else { 0u32 },
			CPUAccessFlags: 0,
			MiscFlags: if gen_mips { D3D11_RESOURCE_MISC_GENERATE_MIPS.0 as u32 } else { 0u32 },
		};

		Ok(desc)
	}
}

#[derive(Debug, Clone)]
pub struct TextureUpload {
	pub data: Box<[u8]>,
	pub stride: u32,
	pub desc: D3D11_TEXTURE2D_DESC,
}

impl TextureUpload {
	pub fn with_image(image: &WicImage) -> WinResult<Self> {
		let image = image.for_dxgi(None)?;
		let desc = image.describe_d3d11()?;
		let (_size, stride) = image.size_info()?;
		let data = image.to_data()?;
		debug_assert_eq!(data.len(), _size);

		Ok(Self {
			data,
			stride,
			desc,
		})
	}

	pub fn describe_d3d11_subresource(&self) -> D3D11_SUBRESOURCE_DATA {
		D3D11_SUBRESOURCE_DATA {
			pSysMem: self.data.as_ptr() as *const _,
			SysMemPitch: self.stride,
			SysMemSlicePitch: self.data.len() as u32,
		}
	}

	pub fn describe_d3d11_srv(&self) -> D3D11_SHADER_RESOURCE_VIEW_DESC {
		let gen_mips = self.desc.MiscFlags & D3D11_RESOURCE_MISC_GENERATE_MIPS.0 as u32 != 0;
		let inner = D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
			Texture2D: D3D11_TEX2D_SRV {
				MipLevels: if gen_mips { u32::MAX } else { 1 },
				.. D3D11_TEX2D_SRV::default()
			},
		};
		D3D11_SHADER_RESOURCE_VIEW_DESC {
			Format: self.desc.Format,
			ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
			Anonymous: inner,
		}
	}

	pub fn create_texture(&self, context: &ID3D11DeviceContext) -> WinResult<ID3D11Texture2D> {
		let device = unsafe { context.GetDevice()? };

		let data = self.describe_d3d11_subresource();
		//data.SysMemSlicePitch = 0;

		debug!("TRACE CreateTexture2D({:#?}, {data:#?}", self.desc);
		let mut texture = None;
		let texture = unsafe {
			device.CreateTexture2D(&self.desc, Some(&data)/*None*/, Some(&mut texture))
		}.and_then(|()| texture.ok_or_else(||
			WinError::new(ERROR_CREATE_FAILED.to_hresult(), "CreateTexture2D produced no output")
		))?;

		Ok(texture)
	}

	pub fn create_srv(&self, context: &ID3D11DeviceContext, texture: &ID3D11Texture2D) -> WinResult<ID3D11ShaderResourceView> {
		let desc = self.describe_d3d11_srv();
		let mut view = None;
		let view = unsafe {
			context.GetDevice()?.CreateShaderResourceView(texture, Some(&desc), Some(&mut view))
		}.and_then(|()| view.ok_or_else(||
			WinError::new(ERROR_CREATE_FAILED.to_hresult(), "CreateShaderResourceView produced no output")
		))?;

		let gen_mips = unsafe {
			desc.Anonymous.Texture2D.MipLevels == u32::MAX
		};
		if gen_mips {
			unsafe {
				context.GenerateMips(&view);
			}
		}
		Ok(view)
	}

	pub fn create_texture_addonapi(&self, context: &ID3D11DeviceContext) -> WinResult<Texture> {
		let texture = self.create_texture(context)?;
		let resource = self.create_srv(context, &texture)?;
		Ok(Texture {
			width: self.desc.Width,
			height: self.desc.Height,
			resource: Some(resource),
		})
	}
}

#[derive(Debug, Clone)]
pub enum TextureEntry {
	Loaded(Box<Texture>),
	Failed,
	Upload {
		callback: Option<RawTextureReceiveCallback>,
		upload: Option<TextureUpload>,
	},
	#[cfg(todo)]
	Decode {
		callback: Option<RawTextureReceiveCallback>,
		image: Option<WicImage>,
	},
}

#[derive(Debug, Clone, Default)]
pub struct TextureCache {
	pub textures: HashMap<Arc<CStr>, TextureEntry>,
	pub upload_count: usize,
	pub fallback: Option<WinResult<Texture>>,
}

static POISON_WARNING: Once = Once::new();
static TEXTURE_CACHE: LazyLock<RwLock<TextureCache>> = LazyLock::new(|| Default::default());

impl TextureCache {
	pub fn next_upload(&mut self) -> Option<(Arc<CStr>, TextureUpload)> {
		if self.upload_count == 0 {
			return None
		}

		for (id, entry) in &mut self.textures {
			let upload = match entry {
				TextureEntry::Upload { upload, .. } =>
					upload,
				_ => continue,
			};

			if let Some(upload) = upload.take() {
				self.upload_count = self.upload_count.saturating_sub(1);
				return Some((id.clone(), upload))
			}
		}
		None
	}

	pub fn queue_upload(&mut self, id: CString, upload: TextureUpload, callback: Option<RawTextureReceiveCallback>) {
		if !self.textures.contains_key(id.as_c_str()) {
			error!("TODO: duplicate texture upload for {id:?}");
			return
		}

		self.upload_count = self.upload_count.saturating_add(1);
		self.textures.insert(id.into(), TextureEntry::Upload {
			callback,
			upload: Some(upload),
		});
	}

	pub fn report_upload(&mut self, id: &Arc<CStr>, texture: WinResult<Texture>) -> Option<(RawTextureReceiveCallback, *const Texture)> {
		let (texture, ptr) = match texture {
			Ok(texture) => {
				let texture = Box::new(texture);
				let ptr: *const Texture = &*texture;
				(TextureEntry::Loaded(texture), ptr)
			},
			Err(err) => {
				error!("Texture {id:?} failed to load: {err}");
				(TextureEntry::Failed, ptr::null())
			},
		};
		match self.textures.entry(id.clone()) {
			Entry::Vacant(_) => {
				error!("TODO: texture {id:?} loaded but wasn't requested");
				None
			},
			Entry::Occupied(mut e) => match e.get_mut() {
				e @ &mut TextureEntry::Upload { callback, upload: None } => {
					*e = texture;
					callback.map(|cb| (cb, ptr))
				},
				e @ &mut TextureEntry::Failed => {
					error!("TODO: did we ask for texture {id:?} reload?");
					*e = texture;
					None
				},
				TextureEntry::Upload { upload: Some(..), .. } => {
					error!("TODO: texture {id:?} wasn't queued for upload");
					None
				},
				TextureEntry::Loaded(..) => {
					error!("TODO: texture {id:?} was already loaded");
					None
				},
			},
		}
	}

	pub fn texture_uploads() {
		let context = match NexusHost::dxgi_device_context() {
			Ok(ctx) => ctx,
			_ => return,
		};

		loop {
			let upload = {
				let mut cache = match TEXTURE_CACHE.try_write() {
					Ok(cache) => cache,
					_ => break,
				};
				cache.init_fallback(&context);
				cache.next_upload()
			};
			let (id, upload) = match upload {
				Some(u) => u,
				None => break,
			};

			let texture = upload.create_texture_addonapi(&context);

			let report = TEXTURE_CACHE.write()
				.unwrap_or_else(|e| e.into_inner())
				.report_upload(&id, texture);

			match report {
				Some((cb, texture)) => cb(id.as_ptr(), texture),
				None => (),
			};
		}
	}

	pub fn schedule_upload(&mut self, id: CString, image: WicImage, callback: Option<RawTextureReceiveCallback>) -> WinResult<()> {
		match self.textures.get(id.as_c_str()) {
			None | Some(TextureEntry::Failed) => (),
			Some(&TextureEntry::Upload { callback: ucb, .. }) if ucb == callback => {
				error!("texture {id:?} already scheduled, stop asking!");
				return Ok(())
			},
			_ => {
				error!("texture {id:?} already scheduled");
				return Ok(())
			},
		}

		let upload = TextureUpload::with_image(&image)?;

		self.textures.insert(id.into(), TextureEntry::Upload {
			upload: Some(upload),
			callback,
		});

		Ok(())
	}

	pub fn get(&self, id: &CStr) -> Option<NonNull<Texture>> {
		match self.textures.get(id)? {
			TextureEntry::Loaded(texture) => Some(nonnull_ref(&*texture)),
			TextureEntry::Failed => self.fallback(),
			TextureEntry::Upload { .. }  => {
				None
			},
		}
	}

	const TEXTURE_FALLBACK_ID: &'static CStr = cstr!(" :3");

	pub fn load_fallback(context: &ID3D11DeviceContext) -> WinResult<Texture> {
		let fallback_data = include_bytes!("fallback-texture.bin");
		let texture = TextureLoaderWic::loader()
			.and_then(|loader| loader.decode_memory_frame(fallback_data, None))
			.map(WicImage::new)
			.and_then(|i| TextureUpload::with_image(&i))
			.and_then(|i| i.create_texture_addonapi(context));
		texture
	}

	pub fn init_fallback(&mut self, context: &ID3D11DeviceContext) {
		if self.fallback.is_none() {
			self.fallback = Some(Self::load_fallback(context));
			if let Some(Err(e)) = &self.fallback {
				debug!("how could you! {e}");
			}
		}
	}

	pub fn fallback(&self) -> Option<NonNull<Texture>> {
		match &self.fallback {
			Some(Ok(t)) => Some(nonnull_ref(t)),
			_ => None,
		}
	}

	pub fn addonapi_ptr_nn(texture: NonNull<Texture>) -> *const Texture {
		texture.as_ptr() as *const Texture
	}

	pub fn addonapi_ptr_nn_opt(texture: Option<NonNull<Texture>>) -> *const Texture {
		texture.map(Self::addonapi_ptr_nn)
			.unwrap_or(ptr::null())
	}

	pub fn addonapi_fallback() -> *const Texture {
		let fallback = TEXTURE_CACHE.try_read()
			.or_else(|e| match e {
				TryLockError::Poisoned(e) => Ok(e.into_inner()),
				TryLockError::WouldBlock => Err(()),
			}).ok()
			.and_then(|c| c.fallback());

		TextureCache::addonapi_ptr_nn_opt(fallback)
	}

}

impl NexusHost {
	fn texture_lookup_with<R, F>(id: Option<&CStr>, f: F) -> Result<R, &CStr> where
		F: FnOnce(&TextureCache, &TextureEntry, &CStr) -> Option<R>,
		R: From<*const Texture>,
	{
		let cache = match TEXTURE_CACHE.read() {
			Ok(c) => c,
			Err(e) => return {
				POISON_WARNING.call_once(|| {
					error!("texture cache poisoned");
				});
				Ok(TextureCache::addonapi_ptr_nn_opt(e.into_inner().fallback()).into())
			},
		};

		let id = match id {
			Some(id) => id,
			None => return {
				error!("texture identifier required");
				Ok(TextureCache::addonapi_ptr_nn_opt(cache.fallback()).into())
			},
		};

		match cache.textures.get(id).and_then(|e| f(&cache, e, id)) {
			Some(texture) => Ok(texture),
			None => Err(id),
		}
	}

	fn texture_lookup_create(id: Option<&CStr>) -> Result<*const Texture, &CStr> {
		Self::texture_lookup_with(id, |_cache, entry, _id| Some(match entry {
			TextureEntry::Loaded(t) => &**t,
			TextureEntry::Failed => return None,
			#[cfg(todo)]
			TextureEntry::Upload { storage, .. } => &*storage,
			TextureEntry::Upload { .. } => ptr::null(),
		}))
	}

	fn texture_lookup_load(id: Option<&CStr>, callback: RawTextureReceiveCallback) -> Result<(), &CStr> {
		let mut id_ptr = ptr::null();
		let texture = Self::texture_lookup_with(id, |_cache, entry, id| {
			id_ptr = id.as_ptr();
			Some(match entry {
				TextureEntry::Loaded(t) => Some(&**t as *const Texture),
				TextureEntry::Failed => return None,
				#[cfg(todo)]
				TextureEntry::Upload { storage, .. } => Some(&*storage),
				TextureEntry::Upload { callback: cb, .. } => {
					match cb {
						&Some(cb) if cb == callback =>
							error!("another caller wants to load {cb:?}'s texture {id:?}, ignoring {callback:?} (you got here late bud)"),
						_ => (),
					}
					None
				},
			})
		})?;

		// if we haven't returned Err(id) to continue with load by now, we either
		// found a texture (Some) or are still loading (None)
		if let Some(texture) = texture {
			callback(id_ptr, texture)
		}

		Ok(())
	}

	fn texture_schedule_load(req: WinResult<TextureUpload>, id: &CStr, callback: RawTextureReceiveCallback) {
		let texture = {
			let mut cache = match TEXTURE_CACHE.write() {
				Ok(cache) => cache,
				Err(e) => {
					// uh oh...
					drop(e);
					POISON_WARNING.call_once(|| {
						error!("texture cache poisoned");
					});
					return callback(id.as_ptr(), TextureCache::addonapi_fallback())
				},
			};
			let prev;
			let res = match req {
				Err(..) => {
					// failed to load, inform callback immediately
					prev = cache.textures.insert(id.into(), TextureEntry::Failed);
					//Some(TextureCache::addonapi_ptr_nn_opt(cache.fallback()))
					Some(ptr::null())
				},
				Ok(upload) => {
					prev = cache.textures.insert(id.into(), TextureEntry::Upload {
						upload: Some(upload),
						callback: Some(callback),
					});
					None
				},
			};
			match prev {
				None | Some(TextureEntry::Failed) => (),
				Some(t) => {
					error!("texture {id:?} replacing {t:?}, this is bad");
				},
			}
			res
		};

		if let Some(texture) = texture {
			callback(id.as_ptr(), texture);
		}
	}

	fn texture_create(req: TextureUpload, id: &CStr) -> WinResult<*const Texture> {
		let context = Self::dxgi_device_context()?;
		let texture = req.create_texture_addonapi(&context)?;

		let mut cache = match TEXTURE_CACHE.write() {
			Ok(cache) => cache,
			Err(e) => {
				// uh oh...
				drop(e);
				POISON_WARNING.call_once(|| {
					error!("texture cache poisoned");
				});
				return Ok(TextureCache::addonapi_fallback())
			},
		};

		let texture = Box::new(texture);
		let ptr: *const Texture = &*texture;
		let prev = cache.textures.insert(id.into(), TextureEntry::Loaded(texture));
		match prev {
			None | Some(TextureEntry::Failed) => (),
			Some(t) => {
				error!("texture {id:?} replacing {t:?}, this is bad");
			},
		}

		Ok(ptr)
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get(identifier: *const c_char) -> *const Texture {
		let id = cstr_opt(&identifier);

		addonapi_stub!(texture::get("{:?}", id));

		let id = Self::texture_lookup_with(id, |cache, entry, _id| Some(match entry {
			TextureEntry::Loaded(t) => &**t,
			TextureEntry::Failed => {
				warn!("failed texture {_id:?} requested");
				cache.fallback().map(TextureCache::addonapi_ptr_nn)?
			},
			#[cfg(todo)]
			TextureEntry::Upload { storage, .. } => &*storage,
			TextureEntry::Upload { .. } => ptr::null(),
		}));

		match id {
			Ok(res) => res,
			Err(..) => {
				// not found
				ptr::null()
			},
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_file(identifier: *const c_char, filename: *const c_char, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);
		let path = cstr_opt(&filename);
		addonapi_stub!(texture::load_from_file("{:?}, {:?}, {:?}", id, path, callback));

		let id = match Self::texture_lookup_load(id, callback) {
			Ok(()) => return,
			Err(id) => id,
		};

		let path = match path.map(CStr::to_string_lossy) {
			Some(Cow::Borrowed(s)) => HSTRING::from(s),
			Some(Cow::Owned(s)) => HSTRING::from(s),
			None => {
				error!("texture filename required");
				return callback(identifier, TextureCache::addonapi_fallback())
			},
		};

		let request = TextureLoaderWic::loader()
			.and_then(|loader| loader.decode_file_frame(&path, None))
			.map(WicImage::new)
			.and_then(|i| TextureUpload::with_image(&i));

		if let Err(e) = &request {
			error!("Failed to load texture {id:?} from {path:?}: {e}");
		}

		Self::texture_schedule_load(request, id, callback)
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_url(identifier: *const c_char, remote: *const c_char, endpoint: *const c_char, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);
		let host = cstr_opt(&remote);
		let path = cstr_opt(&endpoint);

		let id = match Self::texture_lookup_load(id, callback) {
			Ok(()) => return,
			Err(id) => id,
		};

		let t = addonapi_stub!(texture::load_from_url("{:?}, {:?}, {:?}, {:?}", id, host, path, callback) => TextureCache::addonapi_fallback());
		callback(identifier, t)
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_memory(identifier: *const c_char, ptr: *const c_void, size: usize, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);
		let data = ptr::slice_from_raw_parts(ptr as *const u8, size);
		addonapi_stub!(texture::load_from_memory("{:?}, {:?}, {:?}", id, data, callback));

		let id = match Self::texture_lookup_load(id, callback) {
			Ok(()) => return,
			Err(id) => id,
		};

		let request = TextureLoaderWic::loader()
			.and_then(|loader| loader.decode_memory_frame(&*data, None))
			.map(WicImage::new)
			.and_then(|i| TextureUpload::with_image(&i));

		if let Err(e) = &request {
				error!("Failed to load texture {id:?} from {data:?}: {e}");
		}

		Self::texture_schedule_load(request, id, callback)
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_load_from_resource(identifier: *const c_char, resource_id: u32, module: HMODULE, callback: RawTextureReceiveCallback) {
		let id = cstr_opt(&identifier);
		addonapi_stub!(texture::load_from_resource("{:?}, {:?}, {:?}, {:?}", id, resource_id, module, callback));

		let id = match Self::texture_lookup_load(id, callback) {
			Ok(()) => return,
			Err(id) => id,
		};

		let request = TextureLoaderWic::loader()
			.and_then(|loader| loader.decode_resource_frame(module, resource_id, None))
			.map(WicImage::new)
			.and_then(|i| TextureUpload::with_image(&i));

		if let Err(e) = &request {
			error!("Failed to load texture {id:?} from {module:?}:{resource_id}: {e}");
		}

		Self::texture_schedule_load(request, id, callback)
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_file(identifier: *const c_char, filename: *const c_char) -> *const Texture {
		let id = cstr_opt(&identifier);
		let path = cstr_opt(&filename);
		addonapi_stub!(texture::get_or_create_from_file("{:?}, {:?}", id, path));

		let id = match Self::texture_lookup_create(id) {
			Ok(texture) => return texture,
			Err(id) => id,
		};

		let path = match path.map(CStr::to_string_lossy) {
			Some(Cow::Borrowed(s)) => HSTRING::from(s),
			Some(Cow::Owned(s)) => HSTRING::from(s),
			None => {
				error!("texture filename required");
				return TextureCache::addonapi_fallback()
			},
		};

		let texture = TextureLoaderWic::loader()
			.and_then(|loader| loader.decode_file_frame(&path, None))
			.map(WicImage::new)
			.and_then(|i| TextureUpload::with_image(&i))
			.and_then(|i| Self::texture_create(i, id));

		match texture {
			Ok(texture) => texture,
			Err(e) => {
				error!("CreateTexture2D failed to create {id:?} for {path:?}: {e}");
				ptr::null()
			},
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_resource(identifier: *const c_char, resource_id: u32, module: HMODULE) -> *const Texture {
		let id = cstr_opt(&identifier);
		addonapi_stub!(texture::get_or_create_from_resource("{:?}, {:?}, {:?}", id, resource_id, module));

		let id = match Self::texture_lookup_create(id) {
			Ok(texture) => return texture,
			Err(id) => id,
		};

		let texture = TextureLoaderWic::loader()
			.and_then(|loader| loader.decode_resource_frame(module, resource_id, None))
			.map(WicImage::new)
			.and_then(|i| TextureUpload::with_image(&i))
			.and_then(|i| Self::texture_create(i, id));

		match texture {
			Ok(texture) => texture,
			Err(e) => {
				error!("CreateTexture2D failed to create {id:?} for {module:?}/{resource_id}: {e}");
				ptr::null()
			},
		}
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_url(identifier: *const c_char, remote: *const c_char, endpoint: *const c_char) -> *const Texture {
		let id = cstr_opt(&identifier);
		let remote = cstr_opt(&remote);
		let endpoint = cstr_opt(&endpoint);

		addonapi_stub!(texture::get_or_create_from_url("{:?}, {:?}, {:?}", id, remote, endpoint) => TextureCache::addonapi_fallback())
	}

	pub unsafe extern "C-unwind" fn addonapi_texture_get_or_create_from_memory(identifier: *const c_char, ptr: *const c_void, size: usize) -> *const Texture {
		let id = cstr_opt(&identifier);
		let data = ptr::slice_from_raw_parts(ptr as *const u8, size);
		addonapi_stub!(texture::get_or_create_from_memory("{:?}, {:?}", id, data));

		let id = match Self::texture_lookup_create(id) {
			Ok(texture) => return texture,
			Err(id) => id,
		};

		let texture = TextureLoaderWic::loader()
			.and_then(|loader| loader.decode_memory_frame(&*data, None))
			.map(WicImage::new)
			.and_then(|i| TextureUpload::with_image(&i))
			.and_then(|i| Self::texture_create(i, id));

		match texture {
			Ok(texture) => texture,
			Err(e) => {
				error!("CreateTexture2D failed to create {id:?} for {data:?}: {e}");
				ptr::null()
			},
		}
	}
}
