pub mod Common {
	use crate::windows::{
		adapter::windows_newtype,
		core::Result,
		winerror,
	};

	windows_newtype! {
		pub struct Dxgi_Common::DXGI_FORMAT(pub i32);
	}

	impl DXGI_FORMAT {
		pub fn bits_per_pixel(self) -> Option<usize> {
			Some(match self {
				DXGI_FORMAT::R32G32B32A32_FLOAT | DXGI_FORMAT::R32G32B32A32_SINT | DXGI_FORMAT::R32G32B32A32_UINT | DXGI_FORMAT::R32G32B32A32_TYPELESS
					=> 128,
				DXGI_FORMAT::R32G32B32_FLOAT | DXGI_FORMAT::R32G32B32_SINT | DXGI_FORMAT::R32G32B32_UINT | DXGI_FORMAT::R32G32B32_TYPELESS
					=> 96,
				DXGI_FORMAT::R16G16B16A16_FLOAT | DXGI_FORMAT::R16G16B16A16_SINT | DXGI_FORMAT::R16G16B16A16_UINT | DXGI_FORMAT::R16G16B16A16_SNORM | DXGI_FORMAT::R16G16B16A16_UNORM | DXGI_FORMAT::R16G16B16A16_TYPELESS
				| DXGI_FORMAT::R32G32_FLOAT | DXGI_FORMAT::R32G32_SINT | DXGI_FORMAT::R32G32_UINT | DXGI_FORMAT::R32G32_TYPELESS
				| DXGI_FORMAT::R32G8X24_TYPELESS | DXGI_FORMAT::D32_FLOAT_S8X24_UINT | DXGI_FORMAT::R32_FLOAT_X8X24_TYPELESS | DXGI_FORMAT::X32_TYPELESS_G8X24_UINT
				| DXGI_FORMAT::Y416 | DXGI_FORMAT::Y210 | DXGI_FORMAT::Y216
					=> 64,
				DXGI_FORMAT::R8G8B8A8_UNORM | DXGI_FORMAT::R8G8B8A8_UNORM_SRGB | DXGI_FORMAT::R8G8B8A8_SINT | DXGI_FORMAT::R8G8B8A8_UINT | DXGI_FORMAT::R8G8B8A8_TYPELESS
				| DXGI_FORMAT::B8G8R8A8_UNORM | DXGI_FORMAT::R8G8B8A8_SNORM | DXGI_FORMAT::B8G8R8A8_UNORM_SRGB | DXGI_FORMAT::B8G8R8A8_TYPELESS
				| DXGI_FORMAT::B8G8R8X8_UNORM | DXGI_FORMAT::B8G8R8X8_UNORM_SRGB
				| DXGI_FORMAT::R32_SINT | DXGI_FORMAT::R32_UINT | DXGI_FORMAT::R32_FLOAT | DXGI_FORMAT::R32_TYPELESS
				| DXGI_FORMAT::R24G8_TYPELESS | DXGI_FORMAT::D24_UNORM_S8_UINT | DXGI_FORMAT::R24_UNORM_X8_TYPELESS | DXGI_FORMAT::X24_TYPELESS_G8_UINT
				| DXGI_FORMAT::R16G16_FLOAT | DXGI_FORMAT::R16G16_UNORM | DXGI_FORMAT::R16G16_SNORM | DXGI_FORMAT::R16G16_SINT | DXGI_FORMAT::R16G16_UINT | DXGI_FORMAT::R16G16_TYPELESS
				| DXGI_FORMAT::R10G10B10A2_TYPELESS | DXGI_FORMAT::R10G10B10A2_UNORM | DXGI_FORMAT::R10G10B10A2_UINT
				| DXGI_FORMAT::R11G11B10_FLOAT
				| DXGI_FORMAT::R9G9B9E5_SHAREDEXP
				| DXGI_FORMAT::R10G10B10_XR_BIAS_A2_UNORM
				| DXGI_FORMAT::AYUV | DXGI_FORMAT::Y410 | DXGI_FORMAT::YUY2
					=> 32,
				DXGI_FORMAT::R16_SINT | DXGI_FORMAT::R16_UINT | DXGI_FORMAT::R16_UNORM | DXGI_FORMAT::R16_SNORM | DXGI_FORMAT::R16_FLOAT | DXGI_FORMAT::R16_TYPELESS
				| DXGI_FORMAT::R8G8_SINT | DXGI_FORMAT::R8G8_UINT | DXGI_FORMAT::R8G8_SNORM | DXGI_FORMAT::R8G8_UNORM
				| DXGI_FORMAT::R8G8_B8G8_UNORM | DXGI_FORMAT::G8R8_G8B8_UNORM
				| DXGI_FORMAT::B5G6R5_UNORM | DXGI_FORMAT::B5G5R5A1_UNORM
				| DXGI_FORMAT::B4G4R4A4_UNORM | DXGI_FORMAT::A4B4G4R4_UNORM
					=> 16,
				DXGI_FORMAT::R8_SINT | DXGI_FORMAT::R8_UINT | DXGI_FORMAT::R8_UNORM | DXGI_FORMAT::R8_SNORM | DXGI_FORMAT::R8_TYPELESS
				| DXGI_FORMAT::A8_UNORM
					=> 8,
				| DXGI_FORMAT::R1_UNORM
					=> 1,
				// block compression: DXGI_FORMAT::BC*
				// planar and mpeg bs:
				// 16 bpc:  DXGI_FORMAT::P016
				// 10bpc: DXGI_FORMAT::P010
				//.8bpc:  DXGI_FORMAT::420_OPAQUE
				//DXGI_FORMAT::P208
				//DXGI_FORMAT::V208 | DXGI_FORMAT::V408
				//DXGI_FORMAT::SAMPLER_FEEDBACK_MIN_MIP_OPAQUE | DXGI_FORMAT::SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE
				// DXGI_FORMAT::NV12 | DXGI_FORMAT::NV11
				// DXGI_FORMAT::AI44 | DXGI_FORMAT::IA44 | DXGI_FORMAT::P8 | DXGI_FORMAT::A8P8
				_ => return None,
			})
		}

		pub fn try_bits_per_pixel(self) -> Result<usize> {
			self.bits_per_pixel()
				.ok_or_else(|| winerror!(ERROR_INVALID_PIXEL_FORMAT, fmt:"unknown bpp for {self:?}"))
		}

		pub fn supports_mipmaps(self) -> bool {
			match self {
				// feature level 9.1 to 9.3:
				DXGI_FORMAT::R8G8B8A8_UNORM | DXGI_FORMAT::R8G8B8A8_UNORM_SRGB | DXGI_FORMAT::B5G6R5_UNORM | DXGI_FORMAT::B8G8R8A8_UNORM | DXGI_FORMAT::B8G8R8A8_UNORM_SRGB | DXGI_FORMAT::B8G8R8X8_UNORM | DXGI_FORMAT::B8G8R8X8_UNORM_SRGB
					=> true,
				// feature level 9.2:
				DXGI_FORMAT::R16G16B16A16_FLOAT | DXGI_FORMAT::R16G16B16A16_UNORM | DXGI_FORMAT::R16G16_FLOAT | DXGI_FORMAT::R16G16_UNORM | DXGI_FORMAT::R32_FLOAT
					=> true,
				// feature level 9.3:
				DXGI_FORMAT::R32G32B32A32_FLOAT
				// optional? | DXGI_FORMAT::B4G4R4A4
					=> true,
				// feature level 10:
				DXGI_FORMAT::R16G16B16A16_SNORM | DXGI_FORMAT::R32G32_FLOAT | DXGI_FORMAT::R10G10B10A2_UNORM | DXGI_FORMAT::R11G11B10_FLOAT | DXGI_FORMAT::R8G8B8A8_SNORM | DXGI_FORMAT::R16G16_SNORM | DXGI_FORMAT::R8G8_UNORM | DXGI_FORMAT::R8G8_SNORM | DXGI_FORMAT::R16_FLOAT | DXGI_FORMAT::R16_UNORM | DXGI_FORMAT::R16_SNORM | DXGI_FORMAT::R8_UNORM | DXGI_FORMAT::R8_SNORM | DXGI_FORMAT::A8_UNORM
				// optional? | DXGI_FORMAT::R32G32B32_FLOAT | DXGI_FORMAT::B5G5R5A1_UNORM
					=> true,
				_ => false,
			}
		}
	}
	#[allow(non_upper_case_globals)]
	impl DXGI_FORMAT {
		pub const _420_OPAQUE: Self = Self(106);
		pub const A4B4G4R4_UNORM: Self = Self(191);
		pub const A8P8: Self = Self(114);
		pub const A8_UNORM: Self = Self(65);
		pub const AI44: Self = Self(111);
		pub const AYUV: Self = Self(100);
		pub const B4G4R4A4_UNORM: Self = Self(115);
		pub const B5G5R5A1_UNORM: Self = Self(86);
		pub const B5G6R5_UNORM: Self = Self(85);
		pub const B8G8R8A8_TYPELESS: Self = Self(90);
		pub const B8G8R8A8_UNORM: Self = Self(87);
		pub const B8G8R8A8_UNORM_SRGB: Self = Self(91);
		pub const B8G8R8X8_TYPELESS: Self = Self(92);
		pub const B8G8R8X8_UNORM: Self = Self(88);
		pub const B8G8R8X8_UNORM_SRGB: Self = Self(93);
		pub const BC1_TYPELESS: Self = Self(70);
		pub const BC1_UNORM: Self = Self(71);
		pub const BC1_UNORM_SRGB: Self = Self(72);
		pub const BC2_TYPELESS: Self = Self(73);
		pub const BC2_UNORM: Self = Self(74);
		pub const BC2_UNORM_SRGB: Self = Self(75);
		pub const BC3_TYPELESS: Self = Self(76);
		pub const BC3_UNORM: Self = Self(77);
		pub const BC3_UNORM_SRGB: Self = Self(78);
		pub const BC4_SNORM: Self = Self(81);
		pub const BC4_TYPELESS: Self = Self(79);
		pub const BC4_UNORM: Self = Self(80);
		pub const BC5_SNORM: Self = Self(84);
		pub const BC5_TYPELESS: Self = Self(82);
		pub const BC5_UNORM: Self = Self(83);
		pub const BC6H_SF16: Self = Self(96);
		pub const BC6H_TYPELESS: Self = Self(94);
		pub const BC6H_UF16: Self = Self(95);
		pub const BC7_TYPELESS: Self = Self(97);
		pub const BC7_UNORM: Self = Self(98);
		pub const BC7_UNORM_SRGB: Self = Self(99);
		pub const D16_UNORM: Self = Self(55);
		pub const D24_UNORM_S8_UINT: Self = Self(45);
		pub const D32_FLOAT: Self = Self(40);
		pub const D32_FLOAT_S8X24_UINT: Self = Self(20);
		pub const G8R8_G8B8_UNORM: Self = Self(69);
		pub const IA44: Self = Self(112);
		pub const NV11: Self = Self(110);
		pub const NV12: Self = Self(103);
		pub const P010: Self = Self(104);
		pub const P016: Self = Self(105);
		pub const P208: Self = Self(130);
		pub const P8: Self = Self(113);
		pub const R10G10B10A2_TYPELESS: Self = Self(23);
		pub const R10G10B10A2_UINT: Self = Self(25);
		pub const R10G10B10A2_UNORM: Self = Self(24);
		pub const R10G10B10_XR_BIAS_A2_UNORM: Self = Self(89);
		pub const R11G11B10_FLOAT: Self = Self(26);
		pub const R16G16B16A16_FLOAT: Self = Self(10);
		pub const R16G16B16A16_SINT: Self = Self(14);
		pub const R16G16B16A16_SNORM: Self = Self(13);
		pub const R16G16B16A16_TYPELESS: Self = Self(9);
		pub const R16G16B16A16_UINT: Self = Self(12);
		pub const R16G16B16A16_UNORM: Self = Self(11);
		pub const R16G16_FLOAT: Self = Self(34);
		pub const R16G16_SINT: Self = Self(38);
		pub const R16G16_SNORM: Self = Self(37);
		pub const R16G16_TYPELESS: Self = Self(33);
		pub const R16G16_UINT: Self = Self(36);
		pub const R16G16_UNORM: Self = Self(35);
		pub const R16_FLOAT: Self = Self(54);
		pub const R16_SINT: Self = Self(59);
		pub const R16_SNORM: Self = Self(58);
		pub const R16_TYPELESS: Self = Self(53);
		pub const R16_UINT: Self = Self(57);
		pub const R16_UNORM: Self = Self(56);
		pub const R1_UNORM: Self = Self(66);
		pub const R24G8_TYPELESS: Self = Self(44);
		pub const R24_UNORM_X8_TYPELESS: Self = Self(46);
		pub const R32G32B32A32_FLOAT: Self = Self(2);
		pub const R32G32B32A32_SINT: Self = Self(4);
		pub const R32G32B32A32_TYPELESS: Self = Self(1);
		pub const R32G32B32A32_UINT: Self = Self(3);
		pub const R32G32B32_FLOAT: Self = Self(6);
		pub const R32G32B32_SINT: Self = Self(8);
		pub const R32G32B32_TYPELESS: Self = Self(5);
		pub const R32G32B32_UINT: Self = Self(7);
		pub const R32G32_FLOAT: Self = Self(16);
		pub const R32G32_SINT: Self = Self(18);
		pub const R32G32_TYPELESS: Self = Self(15);
		pub const R32G32_UINT: Self = Self(17);
		pub const R32G8X24_TYPELESS: Self = Self(19);
		pub const R32_FLOAT: Self = Self(41);
		pub const R32_FLOAT_X8X24_TYPELESS: Self = Self(21);
		pub const R32_SINT: Self = Self(43);
		pub const R32_TYPELESS: Self = Self(39);
		pub const R32_UINT: Self = Self(42);
		pub const R8G8B8A8_SINT: Self = Self(32);
		pub const R8G8B8A8_SNORM: Self = Self(31);
		pub const R8G8B8A8_TYPELESS: Self = Self(27);
		pub const R8G8B8A8_UINT: Self = Self(30);
		pub const R8G8B8A8_UNORM: Self = Self(28);
		pub const R8G8B8A8_UNORM_SRGB: Self = Self(29);
		pub const R8G8_B8G8_UNORM: Self = Self(68);
		pub const R8G8_SINT: Self = Self(52);
		pub const R8G8_SNORM: Self = Self(51);
		pub const R8G8_TYPELESS: Self = Self(48);
		pub const R8G8_UINT: Self = Self(50);
		pub const R8G8_UNORM: Self = Self(49);
		pub const R8_SINT: Self = Self(64);
		pub const R8_SNORM: Self = Self(63);
		pub const R8_TYPELESS: Self = Self(60);
		pub const R8_UINT: Self = Self(62);
		pub const R8_UNORM: Self = Self(61);
		pub const R9G9B9E5_SHAREDEXP: Self = Self(67);
		pub const SAMPLER_FEEDBACK_MIN_MIP_OPAQUE: Self = Self(189);
		pub const SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE: Self = Self(190);
		pub const UNKNOWN: Self = Self(0);
		pub const V208: Self = Self(131);
		pub const V408: Self = Self(132);
		pub const X24_TYPELESS_G8_UINT: Self = Self(47);
		pub const X32_TYPELESS_G8X24_UINT: Self = Self(22);
		pub const Y210: Self = Self(108);
		pub const Y216: Self = Self(109);
		pub const Y410: Self = Self(101);
		pub const Y416: Self = Self(102);
		pub const YUY2: Self = Self(107);
	}
}
