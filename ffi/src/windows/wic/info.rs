use crate::{
	cstr::{c_wchar, CStrPtr16},
	c_bool32,
	windows::{
		adapter::windows_newtype,
		com::{
			imp::{GetProperty, GetMultiple, GetStringW, GetInterface, get_property, get_interface, get_string_w, string_w_os, interface_hierarchy},
			interface::{Interface, InterfacePtr, InterfaceAs, InterfaceTarget},
			unknown::IUnknown_Vtbl,
			InterfaceOwned,
		},
		core::{Result, GUID, HRESULT},
		wic::ColorContext,
	},
};
use core::ptr::NonNull;
#[cfg(windows)]
use std::{
	os::windows::ffi::OsStringExt,
	ffi::OsString,
};

windows_newtype! {
	pub struct Imaging::WICComponentType(pub i32);
}

/// TODO
pub type BitmapDecoderInfo = BitmapCodecInfo;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ComponentInfo {
	pub interface: InterfaceOwned<ComponentInfo>,
}

pub trait ComponentInfoExt {
	fn component_type(&self) -> Result<WICComponentType>;
	fn clsid(&self) -> Result<GUID>;
	fn vendor_guid(&self) -> Result<GUID>;
	fn get_friendly_name(&self) -> Result<Vec<c_wchar>>;
	fn get_author(&self) -> Result<Vec<c_wchar>>;
	fn get_version(&self) -> Result<Vec<c_wchar>>;
	fn get_spec_version(&self) -> Result<Vec<c_wchar>>;

	#[cfg(windows)]
	fn friendly_name(&self) -> Result<OsString> {
		self.get_friendly_name()
			.map(|s| OsString::from_wide(&s))
	}

	#[cfg(windows)]
	fn author(&self) -> Result<OsString> {
		self.get_author()
			.map(string_w_os)
	}

	#[cfg(windows)]
	fn version(&self) -> Result<OsString> {
		self.get_version()
			.map(string_w_os)
	}

	#[cfg(windows)]
	fn spec_version(&self) -> Result<OsString> {
		self.get_spec_version()
			.map(string_w_os)
	}
}

impl<I: Interface + InterfaceAs<ComponentInfo>> ComponentInfoExt for I {
	fn component_type(&self) -> Result<WICComponentType> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetComponentType)
		}
	}

	fn clsid(&self) -> Result<GUID> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetCLSID)
		}
	}

	fn vendor_guid(&self) -> Result<GUID> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetVendorGUID)
		}
	}

	fn get_friendly_name(&self) -> Result<Vec<c_wchar>> {
		unsafe {
			get_string_w(self.get_parent().as_raw(), self.get_parent_vtable().GetFriendlyName)
		}
	}

	fn get_author(&self) -> Result<Vec<c_wchar>> {
		unsafe {
			get_string_w(self.get_parent().as_raw(), self.get_parent_vtable().GetAuthor)
		}
	}

	fn get_version(&self) -> Result<Vec<c_wchar>> {
		unsafe {
			get_string_w(self.get_parent().as_raw(), self.get_parent_vtable().GetVersion)
		}
	}

	fn get_spec_version(&self) -> Result<Vec<c_wchar>> {
		unsafe {
			get_string_w(self.get_parent().as_raw(), self.get_parent_vtable().GetSpecVersion)
		}
	}
}

unsafe impl Interface for ComponentInfo {
	/// {23BC3F0A-698B-4357-886B-F24D50671334}
	const IID: GUID = GUID::from_values(0x23bc3f0a, 0x698b, 0x4357, [0x88, 0x6b,
		0xf2, 0x4d, 0x50, 0x67, 0x13, 0x34,
	]);

	type Owned = Self;
	type Vtable = IWICComponentInfo_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for ComponentInfo {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { ComponentInfo, IUnknown }

#[repr(C)]
pub struct IWICComponentInfo_Vtbl {
	pub base__: IUnknown_Vtbl,
	pub GetComponentType: GetProperty::<WICComponentType>,
	pub GetCLSID: GetProperty::<GUID>,
	pub GetSigningStatus: GetProperty::<u32>,
	pub GetAuthor: GetStringW,
	pub GetVendorGUID: GetProperty::<GUID>,
	pub GetVersion: GetStringW,
	pub GetSpecVersion: GetStringW,
	pub GetFriendlyName: GetStringW,
}

pub trait PixelFormatInfoExt {
	fn format_guid(&self) -> Result<GUID>;
	fn bits_per_pixel(&self) -> Result<u32>;
	fn channel_count(&self) -> Result<u32>;
	fn color_context(&self) -> Result<ColorContext>;
	// TODO: GetChannelMask
}

impl<I: Interface + InterfaceAs<PixelFormatInfo>> PixelFormatInfoExt for I {
	fn format_guid(&self) -> Result<GUID> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetFormatGUID)
		}
	}

	fn bits_per_pixel(&self) -> Result<u32> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetBitsPerPixel)
		}
	}

	fn channel_count(&self) -> Result<u32> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetChannelCount)
		}
	}

	fn color_context(&self) -> Result<ColorContext> {
		unsafe {
			get_interface(self.get_parent().as_raw(), self.get_parent_vtable().GetColorContext)
		}
	}
}

/// https://learn.microsoft.com/en-us/windows/win32/wic/-wic-codec-native-pixel-formats
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PixelFormatInfo {
	pub interface: InterfaceOwned<PixelFormatInfo>,
}

unsafe impl Interface for PixelFormatInfo {
	/// {E8EDA601-3D48-431A-AB44-69059BE88BBE}
	const IID: GUID = GUID::from_values(0xe8eda601, 0x3d48, 0x431a, [0xab, 0x44,
		0x69, 0x05, 0x9b, 0xe8, 0x8b, 0xbe,
	]);

	type Owned = Self;
	type Vtable = IWICPixelFormatInfo_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for PixelFormatInfo {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { PixelFormatInfo, IUnknown, ComponentInfo }
#[repr(C)]
pub struct IWICPixelFormatInfo_Vtbl {
	pub base__: IWICComponentInfo_Vtbl,
	pub GetFormatGUID: GetProperty<GUID>,
	pub GetColorContext: GetInterface<ColorContext>,
	pub GetBitsPerPixel: GetProperty<u32>,
	pub GetChannelCount: GetProperty<u32>,
	pub GetChannelMask: unsafe extern "system" fn(this: *mut InterfaceTarget, channel: u32, buffer_len: u32, buffer: *mut u8, len_actual: *mut u32) -> HRESULT,
}

pub trait BitmapCodecInfoExt {
	fn container_format(&self) -> Result<GUID>;
}

impl<I: InterfaceAs<BitmapCodecInfo>> BitmapCodecInfoExt for I {
	fn container_format(&self) -> Result<GUID> {
		unsafe {
			get_property(self.get_parent().as_raw(), self.get_parent_vtable().GetContainerFormat)
		}
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct BitmapCodecInfo {
	pub interface: InterfaceOwned<BitmapCodecInfo>,
}

unsafe impl Interface for BitmapCodecInfo {
	/// {E87A44C4-B76E-4C47-8B09-298EB12A2714}
	const IID: GUID = GUID::from_values(0xe87a44c4, 0xb76e, 0x4c47, [0x8b, 0x09,
		0x29, 0x8e, 0xb1, 0x2a, 0x27, 0x14,
	]);

	type Owned = Self;
	type Vtable = IWICBitmapCodecInfo_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for BitmapCodecInfo {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { BitmapCodecInfo, IUnknown, ComponentInfo }
#[repr(C)]
pub struct IWICBitmapCodecInfo_Vtbl {
	pub base__: IWICComponentInfo_Vtbl,
	pub GetContainerFormat: GetProperty<GUID>,
	pub GetPixelFormats: GetMultiple<GUID>,
	pub GetColorManagementVersion: GetStringW,
	pub GetDeviceManufacturer: GetStringW,
	pub GetDeviceModels: GetStringW,
	pub GetMimeTypes: GetStringW,
	pub GetFileExtensions: GetStringW,
	pub DoesSupportAnimation: GetProperty<c_bool32>,
	pub DoesSupportChromakey: GetProperty<c_bool32>,
	pub DoesSupportLossless: GetProperty<c_bool32>,
	pub DoesSupportMultiframe: GetProperty<c_bool32>,
	pub MatchesMimeType: unsafe extern "system" fn(*mut InterfaceTarget, CStrPtr16<'static>, *mut c_bool32) -> HRESULT,
}

#[allow(non_upper_case_globals)]
impl ComponentInfo {
	pub const GUID_MetadataFormat8BIMIPTC: GUID = GUID::from_u128(0x0010568c_0852_4e6a_b191_5c33ac5b0430);
	pub const GUID_MetadataFormat8BIMIPTCDigest: GUID = GUID::from_u128(0x1ca32285_9ccd_4786_8bd8_79539db6a006);
	pub const GUID_MetadataFormat8BIMResolutionInfo: GUID = GUID::from_u128(0x739f305d_81db_43cb_ac5e_55013ef9f003);
	pub const GUID_MetadataFormatAPE: GUID = GUID::from_u128(0x2e043dc2_c967_4e05_875e_618bf67e85c3);
	pub const GUID_MetadataFormatApp0: GUID = GUID::from_u128(0x79007028_268d_45d6_a3c2_354e6a504bc9);
	pub const GUID_MetadataFormatApp1: GUID = GUID::from_u128(0x8fd3dfc3_f951_492b_817f_69c2e6d9a5b0);
	pub const GUID_MetadataFormatApp13: GUID = GUID::from_u128(0x326556a2_f502_4354_9cc0_8e3f48eaf6b5);
	pub const GUID_MetadataFormatChunkbKGD: GUID = GUID::from_u128(0xe14d3571_6b47_4dea_b60a_87ce0a78dfb7);
	pub const GUID_MetadataFormatChunkcHRM: GUID = GUID::from_u128(0x9db3655b_2842_44b3_8067_12e9b375556a);
	pub const GUID_MetadataFormatChunkgAMA: GUID = GUID::from_u128(0xf00935a5_1d5d_4cd1_81b2_9324d7eca781);
	pub const GUID_MetadataFormatChunkhIST: GUID = GUID::from_u128(0xc59a82da_db74_48a4_bd6a_b69c4931ef95);
	pub const GUID_MetadataFormatChunkiCCP: GUID = GUID::from_u128(0xeb4349ab_b685_450f_91b5_e802e892536c);
	pub const GUID_MetadataFormatChunkiTXt: GUID = GUID::from_u128(0xc2bec729_0b68_4b77_aa0e_6295a6ac1814);
	pub const GUID_MetadataFormatChunksRGB: GUID = GUID::from_u128(0xc115fd36_cc6f_4e3f_8363_524b87c6b0d9);
	pub const GUID_MetadataFormatChunktEXt: GUID = GUID::from_u128(0x568d8936_c0a9_4923_905d_df2b38238fbc);
	pub const GUID_MetadataFormatChunktIME: GUID = GUID::from_u128(0x6b00ae2d_e24b_460a_98b6_878bd03072fd);
	pub const GUID_MetadataFormatDds: GUID = GUID::from_u128(0x4a064603_8c33_4e60_9c29_136231702d08);
	pub const GUID_MetadataFormatExif: GUID = GUID::from_u128(0x1c3c4f9d_b84a_467d_9493_36cfbd59ea57);
	pub const GUID_MetadataFormatGCE: GUID = GUID::from_u128(0x2a25cad8_deeb_4c69_a788_0ec2266dcafd);
	pub const GUID_MetadataFormatGifComment: GUID = GUID::from_u128(0xc4b6e0e0_cfb4_4ad3_ab33_9aad2355a34a);
	pub const GUID_MetadataFormatGps: GUID = GUID::from_u128(0x7134ab8a_9351_44ad_af62_448db6b502ec);
	pub const GUID_MetadataFormatHeif: GUID = GUID::from_u128(0x817ef3e1_1288_45f4_a852_260d9e7cce83);
	pub const GUID_MetadataFormatHeifHDR: GUID = GUID::from_u128(0x568b8d8a_1e65_438c_8968_d60e1012beb9);
	pub const GUID_MetadataFormatIMD: GUID = GUID::from_u128(0xbd2bb086_4d52_48dd_9677_db483e85ae8f);
	pub const GUID_MetadataFormatIPTC: GUID = GUID::from_u128(0x4fab0914_e129_4087_a1d1_bc812d45a7b5);
	pub const GUID_MetadataFormatIRB: GUID = GUID::from_u128(0x16100d66_8570_4bb9_b92d_fda4b23ece67);
	pub const GUID_MetadataFormatIfd: GUID = GUID::from_u128(0x537396c6_2d8a_4bb6_9bf8_2f0a8e2a3adf);
	pub const GUID_MetadataFormatInterop: GUID = GUID::from_u128(0xed686f8e_681f_4c8b_bd41_a8addbf6b3fc);
	pub const GUID_MetadataFormatJpegChrominance: GUID = GUID::from_u128(0xf73d0dcf_cec6_4f85_9b0e_1c3956b1bef7);
	pub const GUID_MetadataFormatJpegComment: GUID = GUID::from_u128(0x220e5f33_afd3_474e_9d31_7d4fe730f557);
	pub const GUID_MetadataFormatJpegLuminance: GUID = GUID::from_u128(0x86908007_edfc_4860_8d4b_4ee6e83e6058);
	pub const GUID_MetadataFormatLSD: GUID = GUID::from_u128(0xe256031e_6299_4929_b98d_5ac884afba92);
	pub const GUID_MetadataFormatSubIfd: GUID = GUID::from_u128(0x58a2e128_2db9_4e57_bb14_5177891ed331);
	pub const GUID_MetadataFormatThumbnail: GUID = GUID::from_u128(0x243dcee9_8703_40ee_8ef0_22a600b8058c);
	pub const GUID_MetadataFormatUnknown: GUID = GUID::from_u128(0xa45e592f_9078_4a7c_adb5_4edc4fd61b1f);
	pub const GUID_MetadataFormatWebpANIM: GUID = GUID::from_u128(0x6dc4fda6_78e6_4102_ae35_bcfa1edcc78b);
	pub const GUID_MetadataFormatWebpANMF: GUID = GUID::from_u128(0x43c105ee_b93b_4abb_b003_a08c0d870471);
	pub const GUID_MetadataFormatXMP: GUID = GUID::from_u128(0xbb5acc38_f216_4cec_a6c5_5f6e739763a9);
	pub const GUID_MetadataFormatXMPAlt: GUID = GUID::from_u128(0x7b08a675_91aa_481b_a798_4da94908613b);
	pub const GUID_MetadataFormatXMPBag: GUID = GUID::from_u128(0x833cca5f_dcb7_4516_806f_6596ab26dce4);
	pub const GUID_MetadataFormatXMPSeq: GUID = GUID::from_u128(0x63e8df02_eb6c_456c_a224_b25e794fd648);
	pub const GUID_MetadataFormatXMPStruct: GUID = GUID::from_u128(0x22383cf1_ed17_4e2e_af17_d85b8f6b30d0);

	pub const GUID_VendorMicrosoft: GUID = GUID::from_u128(0xf0e749ca_edef_4589_a73a_ee0e626a2a2b);
	pub const GUID_VendorMicrosoftBuiltIn: GUID = GUID::from_u128(0x257a30fd_06b6_462b_aea4_63f70b86e533);
}

#[allow(non_upper_case_globals)]
impl PixelFormatInfo {
	pub const GUID_112bpp6ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc937);
	pub const GUID_112bpp7Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc92a);
	pub const GUID_128bpp7ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc938);
	pub const GUID_128bpp8Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc92b);
	pub const GUID_128bppPRGBAFloat: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc91a);
	pub const GUID_128bppRGBAFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc91e);
	pub const GUID_128bppRGBAFloat: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc919);
	pub const GUID_128bppRGBFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc941);
	pub const GUID_128bppRGBFloat: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc91b);
	pub const GUID_144bpp8ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc939);
	pub const GUID_16bppBGR555: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc909);
	pub const GUID_16bppBGR565: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90a);
	pub const GUID_16bppBGRA5551: GUID = GUID::from_u128(0x05ec7c2b_f1e6_4961_ad46_e1cc810a87d2);
	pub const GUID_16bppCbCr: GUID = GUID::from_u128(0xff95ba6e_11e0_4263_bb45_01721f3460a4);
	pub const GUID_16bppCbQuantizedDctCoefficients: GUID = GUID::from_u128(0xd2c4ff61_56a5_49c2_8b5c_4c1925964837);
	pub const GUID_16bppCrQuantizedDctCoefficients: GUID = GUID::from_u128(0x2fe354f0_1680_42d8_9231_e73c0565bfc1);
	pub const GUID_16bppGray: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90b);
	pub const GUID_16bppGrayFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc913);
	pub const GUID_16bppGrayHalf: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc93e);
	pub const GUID_16bppYQuantizedDctCoefficients: GUID = GUID::from_u128(0xa355f433_48e8_4a42_84d8_e2aa26ca80a4);
	pub const GUID_1bppIndexed: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc901);
	pub const GUID_24bpp3Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc920);
	pub const GUID_24bppBGR: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90c);
	pub const GUID_24bppRGB: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90d);
	pub const GUID_2bppGray: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc906);
	pub const GUID_2bppIndexed: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc902);
	pub const GUID_32bpp3ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc92e);
	pub const GUID_32bpp4Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc921);
	pub const GUID_32bppBGR: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90e);
	pub const GUID_32bppBGR101010: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc914);
	pub const GUID_32bppBGRA: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90f);
	pub const GUID_32bppCMYK: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc91c);
	pub const GUID_32bppGrayFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc93f);
	pub const GUID_32bppGrayFloat: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc911);
	pub const GUID_32bppPBGRA: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc910);
	pub const GUID_32bppPRGBA: GUID = GUID::from_u128(0x3cc4a650_a527_4d37_a916_3142c7ebedba);
	pub const GUID_32bppR10G10B10A2: GUID = GUID::from_u128(0x604e1bb5_8a3c_4b65_b11c_bc0b8dd75b7f);
	pub const GUID_32bppR10G10B10A2HDR10: GUID = GUID::from_u128(0x9c215c5d_1acc_4f0e_a4bc_70fb3ae8fd28);
	pub const GUID_32bppRGB: GUID = GUID::from_u128(0xd98c6b95_3efe_47d6_bb25_eb1748ab0cf1);
	pub const GUID_32bppRGBA: GUID = GUID::from_u128(0xf5c7ad2d_6a8d_43dd_a7a8_a29935261ae9);
	pub const GUID_32bppRGBA1010102: GUID = GUID::from_u128(0x25238d72_fcf9_4522_b514_5578e5ad55e0);
	pub const GUID_32bppRGBA1010102XR: GUID = GUID::from_u128(0x00de6b9a_c101_434b_b502_d0165ee1122c);
	pub const GUID_32bppRGBE: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc93d);
	pub const GUID_40bpp4ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc92f);
	pub const GUID_40bpp5Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc922);
	pub const GUID_40bppCMYKAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc92c);
	pub const GUID_48bpp3Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc926);
	pub const GUID_48bpp5ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc930);
	pub const GUID_48bpp6Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc923);
	pub const GUID_48bppBGR: GUID = GUID::from_u128(0xe605a384_b468_46ce_bb2e_36f180e64313);
	pub const GUID_48bppBGRFixedPoint: GUID = GUID::from_u128(0x49ca140e_cab6_493b_9ddf_60187c37532a);
	pub const GUID_48bppRGB: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc915);
	pub const GUID_48bppRGBFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc912);
	pub const GUID_48bppRGBHalf: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc93b);
	pub const GUID_4bppGray: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc907);
	pub const GUID_4bppIndexed: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc903);
	pub const GUID_56bpp6ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc931);
	pub const GUID_56bpp7Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc924);
	pub const GUID_64bpp3ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc934);
	pub const GUID_64bpp4Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc927);
	pub const GUID_64bpp7ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc932);
	pub const GUID_64bpp8Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc925);
	pub const GUID_64bppBGRA: GUID = GUID::from_u128(0x1562ff7c_d352_46f9_979e_42976b792246);
	pub const GUID_64bppBGRAFixedPoint: GUID = GUID::from_u128(0x356de33c_54d2_4a23_bb04_9b7bf9b1d42d);
	pub const GUID_64bppCMYK: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc91f);
	pub const GUID_64bppPBGRA: GUID = GUID::from_u128(0x8c518e8e_a4ec_468b_ae70_c9a35a9c5530);
	pub const GUID_64bppPRGBA: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc917);
	pub const GUID_64bppPRGBAHalf: GUID = GUID::from_u128(0x58ad26c2_c623_4d9d_b320_387e49f8c442);
	pub const GUID_64bppRGB: GUID = GUID::from_u128(0xa1182111_186d_4d42_bc6a_9c8303a8dff9);
	pub const GUID_64bppRGBA: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc916);
	pub const GUID_64bppRGBAFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc91d);
	pub const GUID_64bppRGBAHalf: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc93a);
	pub const GUID_64bppRGBFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc940);
	pub const GUID_64bppRGBHalf: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc942);
	pub const GUID_72bpp8ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc933);
	pub const GUID_80bpp4ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc935);
	pub const GUID_80bpp5Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc928);
	pub const GUID_80bppCMYKAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc92d);
	pub const GUID_8bppAlpha: GUID = GUID::from_u128(0xe6cd0116_eeba_4161_aa85_27dd9fb3a895);
	pub const GUID_8bppCb: GUID = GUID::from_u128(0x1339f224_6bfe_4c3e_9302_e4f3a6d0ca2a);
	pub const GUID_8bppCr: GUID = GUID::from_u128(0xb8145053_2116_49f0_8835_ed844b205c51);
	pub const GUID_8bppGray: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc908);
	pub const GUID_8bppIndexed: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc904);
	pub const GUID_8bppY: GUID = GUID::from_u128(0x91b4db54_2df9_42f0_b449_2909bb3df88e);
	pub const GUID_96bpp5ChannelsAlpha: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc936);
	pub const GUID_96bpp6Channels: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc929);
	pub const GUID_96bppRGBFixedPoint: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc918);
	pub const GUID_96bppRGBFloat: GUID = GUID::from_u128(0xe3fed78f_e8db_4acf_84c1_e97f6136b327);
	pub const GUID_BlackWhite: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc905);
	pub const GUID_DontCare: GUID = GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc900);
}

#[allow(non_upper_case_globals)]
impl BitmapCodecInfo {
	pub const GUID_ContainerFormatAdng: GUID = GUID::from_u128(0xf3ff6d0d_38c0_41c4_b1fe_1f3824f17b84);
	pub const GUID_ContainerFormatBmp: GUID = GUID::from_u128(0x0af1d87e_fcfe_4188_bdeb_a7906471cbe3);
	pub const GUID_ContainerFormatDds: GUID = GUID::from_u128(0x9967cb95_2e85_4ac8_8ca2_83d7ccd425c9);
	pub const GUID_ContainerFormatGif: GUID = GUID::from_u128(0x1f8a5601_7d4d_4cbd_9c82_1bc8d4eeb9a5);
	pub const GUID_ContainerFormatHeif: GUID = GUID::from_u128(0xe1e62521_6787_405b_a339_500715b5763f);
	pub const GUID_ContainerFormatIco: GUID = GUID::from_u128(0xa3a860c4_338f_4c17_919a_fba4b5628f21);
	pub const GUID_ContainerFormatJpeg: GUID = GUID::from_u128(0x19e4a5aa_5662_4fc5_a0c0_1758028e1057);
	pub const GUID_ContainerFormatPng: GUID = GUID::from_u128(0x1b7cfaf4_713f_473c_bbcd_6137425faeaf);
	pub const GUID_ContainerFormatRaw: GUID = GUID::from_u128(0xfe99ce60_f19c_433c_a3ae_00acefa9ca21);
	pub const GUID_ContainerFormatTiff: GUID = GUID::from_u128(0x163bcc30_e2e9_4f0b_961d_a3e9fdb788a3);
	pub const GUID_ContainerFormatWebp: GUID = GUID::from_u128(0xe094b0e2_67f2_45b3_b0ea_115337ca7cf3);
	pub const GUID_ContainerFormatWmp: GUID = GUID::from_u128(0x57a37caa_367a_4540_916b_f183c5093a4b);
}
