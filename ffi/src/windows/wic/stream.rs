use crate::{
	cstr::CStrPtr16,
	windows::{
		com::{
			imp::interface_hierarchy,
			interface::{Interface, InterfacePtr, InterfaceAs, InterfaceTarget},
			stream::{IStream, ISequentialStream, IStream_Vtbl},
			InterfaceOwned,
		},
		core::{Result, GUID, HRESULT},
		wic::{ImagingFactory, WICDecodeOptions, BitmapFrameDecode, BitmapDecoderExt},
		Win32::Foundation::GENERIC_ACCESS_RIGHTS,
	},
};
use std::{borrow::Cow, ptr::NonNull};

pub trait StreamExt {
	unsafe fn initialize_from_memory<'b>(&self, data: &'b [u8]) -> Result<()>;
	fn initialize_from_istream<S: InterfaceAs<IStream>>(&self, istream: &S) -> Result<()>;
	fn initialize_from_filename(&self, filename: CStrPtr16, access: GENERIC_ACCESS_RIGHTS) -> Result<()>;
}

impl<I: InterfaceAs<Stream>> StreamExt for I {
	unsafe fn initialize_from_memory<'b>(&self, data: &'b [u8]) -> Result<()> {
		(self.get_parent_vtable().InitializeFromMemory)(self.get_parent().as_raw(), data.as_ptr(), data.len() as _)
			.ok()
	}

	fn initialize_from_istream<S: InterfaceAs<IStream>>(&self, istream: &S) -> Result<()> {
		let istream = istream.get_parent();
		unsafe {
			(self.get_parent_vtable().InitializeFromIStream)(self.get_parent().as_raw(), istream.as_raw())
		}.ok()
	}

	fn initialize_from_filename(&self, filename: CStrPtr16, access: GENERIC_ACCESS_RIGHTS) -> Result<()> {
		unsafe {
			let filename = filename.immortal();
			(self.get_parent_vtable().InitializeFromFilename)(self.get_parent().as_raw(), Some(filename), access)
		}.ok()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Stream {
	pub interface: InterfaceOwned<Stream>,
}

unsafe impl Interface for Stream {
	/// {135FF860-22B7-4DDF-B0F6-218F4F299A43}
	const IID: GUID = GUID::from_values(0x135ff860, 0x22b7, 0x4ddf, [0xb0, 0xf6,
		0x21, 0x8f, 0x4f, 0x29, 0x9a, 0x43,
	]);

	type Owned = Self;
	type Vtable = IWICStream_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for Stream {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}

interface_hierarchy! { Stream, IUnknown, IStream, ISequentialStream }

#[repr(C)]
pub struct IWICStream_Vtbl {
	pub base__: IStream_Vtbl,
	pub InitializeFromIStream: unsafe extern "system" fn(this: *mut InterfaceTarget, istream: *mut InterfaceTarget) -> HRESULT,
	pub InitializeFromFilename: unsafe extern "system" fn(this: *mut InterfaceTarget, filename: Option<CStrPtr16<'static>>, access: GENERIC_ACCESS_RIGHTS) -> HRESULT,
	pub InitializeFromMemory: unsafe extern "system" fn(this: *mut InterfaceTarget, bytes: *const u8, len: u32) -> HRESULT,
	pub InitializeFromIStreamRegion: unsafe extern "system" fn(this: *mut InterfaceTarget, istream: *mut InterfaceTarget, offset: u64, len_max: u64) -> HRESULT,
}

#[derive(Debug)]
pub struct BitmapFrameBytes<'d> {
	frame: BitmapFrameDecode,
	data: Cow<'d, [u8]>,
}

impl<'d> BitmapFrameBytes<'d> {
	pub fn decode_frame<B: Into<Cow<'d, [u8]>>>(factory: &ImagingFactory, data: B, vendor: Option<&GUID>, opts: Option<WICDecodeOptions>, frame: u32) -> Result<Self> {
		let data = data.into();
		let decoder = unsafe {
			factory.decode_bytes_unchecked(&data, vendor, opts)?
		};
		let frame = decoder.get_frame(frame)?;

		Ok(unsafe {
			BitmapFrameBytes::new_unchecked(frame, data)
		})
	}

	pub const unsafe fn new_unchecked(frame: BitmapFrameDecode, data: Cow<'d, [u8]>) -> Self {
		Self {
			frame,
			data,
		}
	}

	pub fn clone_ref<'a>(&'a self) -> BitmapFrameBytes<'a> {
		unsafe {
			BitmapFrameBytes::new_unchecked(self.frame.clone(), Cow::Borrowed(&self.data()))
		}
	}

	pub const fn data(&self) -> &Cow<'d, [u8]> {
		&self.data
	}

	pub unsafe fn data_mut(&mut self) -> &mut Cow<'d, [u8]> {
		&mut self.data
	}

	pub const unsafe fn frame(&self) -> &BitmapFrameDecode {
		&self.frame
	}

	pub unsafe fn frame_mut(&mut self) -> &mut BitmapFrameDecode {
		&mut self.frame
	}
}
