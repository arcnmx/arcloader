use crate::windows::{
	adapter::windows_newtype,
	com::{
		imp::{interface_hierarchy, GetProperty2, GetInterface},
		interface::{Interface, InterfacePtr, InterfaceTarget},
		InterfaceOwned,
	},
	core::{GUID, HRESULT},
	wic::{
		WICRect,
		bitmap::{BitmapSource, IWICBitmapSource_Vtbl},
	},
};
use core::ptr::NonNull;

#[repr(C)]
pub struct WICDdsFormatInfo {
	#[cfg(feature = "dxgi")]
	pub dxgi_format: crate::windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
	#[cfg(not(feature = "dxgi"))]
	pub dxgi_format: i32,
	pub bytes_per_block: u32,
	pub block_width: u32,
	pub block_height: u32,
}
#[cfg(feature = "dxgi")]
windows_newtype! {
	impl From for Imaging::WICDdsFormatInfo(pub Self);
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct DdsFrameDecode {
	pub interface: InterfaceOwned<DdsFrameDecode>,
}

impl DdsFrameDecode {
	pub const DEFAULT_FRAME: u32 = 0;
}

unsafe impl Interface for DdsFrameDecode {
	/// {3D4C0C61-18A4-41E4-BD80-481A4FC9F464}
	const IID: GUID = GUID::from_values(0x3d4c0c61, 0x18a4, 0x41e4, [0xbd, 0x80,
		0x48, 0x1a, 0x4f, 0xc9, 0xf4, 0x64,
	]);

	type Owned = Self;
	type Vtable = IWICDdsFrameDecode_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for DdsFrameDecode {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
interface_hierarchy! { DdsFrameDecode, IUnknown, BitmapSource }

#[repr(C)]
pub struct IWICDdsFrameDecode_Vtbl {
	pub base__: IWICBitmapSource_Vtbl,
	pub GetSizeInBlocks: GetProperty2<u32, u32>,
	pub GetFormatInfo: GetInterface<WICDdsFormatInfo>,
	pub CopyBlocks: unsafe extern "system" fn(this: *mut InterfaceTarget, bounds_in_blocks: *const WICRect, stride: u32, size: u32, buffer: *mut u8) -> HRESULT,
}
