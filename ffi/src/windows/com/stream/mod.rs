use crate::windows::{
	adapter::windows_newtype,
	com::{
		imp::{GetInterface, Routine, SetProperty, interface_res},
		interface::{Interface, InterfacePtr, InterfaceAs, InterfaceBase, InterfaceTarget},
		unknown::{IUnknown, IUnknown_Vtbl},
		InterfaceOwned,
	},
	core::{GUID, HRESULT, PWSTR, Result},
	Win32::Foundation::FILETIME,
};
use core::{mem::{transmute, MaybeUninit}, ops, ptr::NonNull};

windows_newtype! {
	pub struct Com::STREAM_SEEK(pub u32);
}

windows_newtype! {
	pub struct Com::STATFLAG(pub i32);
}

windows_newtype! {
	pub struct Com::STGM(pub u32);
}
windows_newtype! {
	pub struct Com::STGC(pub i32);
}

#[cfg(todo)]
windows_newtype! {
	impl From for Com::STATSTG(Self);
}

#[repr(C)]
pub struct STATSTG {
	pub pwcsName: PWSTR,
	pub kind: u32,
	pub cbSize: u64,
	pub mtime: FILETIME,
	pub ctime: FILETIME,
	pub atime: FILETIME,
	pub grfMode: STGM,
	pub grfLocksSupported: u32,
	pub clsid: GUID,
	pub grfStateBits: u32,
	pub reserved: u32,
}

pub trait IStreamExt {
	fn clone_new(&self) -> Result<IStream>;
	fn seek(&self, amount: i64, base: STREAM_SEEK) -> Result<u64>;
	fn set_size(&self, size: u64) -> Result<()>;
	fn commit(&self, flags: STGC) -> Result<()>;
	fn revert(&self) -> Result<()>;
	fn stat(&self, flags: STATFLAG) -> Result<STATSTG>;
}

impl<I: InterfaceAs<IStream>> IStreamExt for I {
	fn clone_new(&self) -> Result<IStream> {
		let mut out = None;
		let res = unsafe {
			(self.get_parent_vtable().Clone)(self.get_parent().as_raw(), &mut out)
		};
		interface_res(res, out)
	}

	fn seek(&self, amount: i64, base: STREAM_SEEK) -> Result<u64> {
		let mut out = 0u64;
		let res = unsafe {
			(self.get_parent_vtable().Seek)(self.get_parent().as_raw(), amount, base, &mut out)
		};
		res.ok().map(move |()| out)
	}

	fn set_size(&self, size: u64) -> Result<()> {
		unsafe {
			(self.get_parent_vtable().SetSize)(self.get_parent().as_raw(), size)
		}.ok()
	}

	fn commit(&self, flags: STGC) -> Result<()> {
		unsafe {
			(self.get_parent_vtable().Commit)(self.get_parent().as_raw(), flags.0 as _)
		}.ok()
	}

	fn revert(&self) -> Result<()> {
		unsafe {
			(self.get_parent_vtable().Revert)(self.get_parent().as_raw())
		}.ok()
	}

	fn stat(&self, flags: STATFLAG) -> Result<STATSTG> {
		let mut out = MaybeUninit::<STATSTG>::uninit();
		let res = unsafe {
			(self.get_parent_vtable().Stat)(self.get_parent().as_raw(), out.as_mut_ptr(), flags.0 as _)
		};
		res.ok().map(move |()| unsafe {
			out.assume_init()
		})
	}

	/* TODO: CopyTo, LockRegion, UnlockRegion */
}

pub trait ISequentialStreamExt {
	#[inline]
	fn read(&self, buf: &mut [u8]) -> Result<usize> {
		self.read_uninit(unsafe {
			transmute(buf)
		})
	}

	fn read_uninit(&self, buf: &mut [MaybeUninit<u8>]) -> Result<usize>;

	fn write(&self, buf: &[u8]) -> Result<usize>;
}

impl<I: InterfaceAs<ISequentialStream>> ISequentialStreamExt for I {
	fn read_uninit(&self, buf: &mut [MaybeUninit<u8>]) -> Result<usize> {
		let mut out = 0u32;
		let res = unsafe {
			(self.get_parent_vtable().Read)(self.get_parent().as_raw(), buf.as_mut_ptr() as *mut MaybeUninit<u8> as *mut _, buf.len() as u32, &mut out)
		};
		res.ok().map(move |()| out as usize)
	}

	fn write(&self, buf: &[u8]) -> Result<usize> {
		let mut out = 0u32;
		let res = unsafe {
			(self.get_parent_vtable().Write)(self.get_parent().as_raw(), buf.as_ptr() as *const _, buf.len() as u32, &mut out)
		};
		res.ok().map(move |()| out as usize)
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct IStream {
	pub interface: InterfaceOwned<IStream>,
}

unsafe impl Interface for IStream {
	/// {0000000C-0000-0000-C000-000000000046}
	const IID: GUID = GUID::from_values(0x0000000c, 0x0000, 0x0000, [0xc0, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
	]);

	type Owned = Self;
	type Vtable = IStream_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for IStream {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
unsafe impl InterfaceBase<IUnknown> for IStream {}
unsafe impl InterfaceBase<ISequentialStream> for IStream {}
unsafe impl InterfaceBase<IStream> for IStream {}

#[repr(C)]
pub struct IStream_Vtbl {
	pub base__: ISequentialStream_Vtbl,
	pub Seek: unsafe extern "system" fn(this: *mut InterfaceTarget, position: i64, base: STREAM_SEEK, actual: *mut u64) -> HRESULT,
	pub SetSize: SetProperty<u64>,
	pub CopyTo: unsafe extern "system" fn(this: *mut InterfaceTarget, stm: *mut InterfaceTarget, amt: u64, read: *mut u64, written: *mut u64) -> HRESULT,
	pub Commit: SetProperty<u32>,
	pub Revert: Routine,
	pub LockRegion: unsafe extern "system" fn(this: *mut InterfaceTarget, offset: u64, cb: u64, locktype: u32) -> HRESULT,
	pub UnlockRegion: unsafe extern "system" fn(this: *mut InterfaceTarget, offset: u64, cb: u64, locktype: u32) -> HRESULT,
	pub Stat: unsafe extern "system" fn(this: *mut InterfaceTarget, out: *mut STATSTG, flags: /*STATFLAG*/u32) -> HRESULT,
	pub Clone: GetInterface<IStream>,
}

impl AsRef<ISequentialStream_Vtbl> for IStream_Vtbl {
	fn as_ref(&self) -> &ISequentialStream_Vtbl {
		&self.base__
	}
}
impl AsRef<IUnknown_Vtbl> for IStream_Vtbl {
	fn as_ref(&self) -> &IUnknown_Vtbl {
		self.base__.as_ref()
	}
}
impl ops::Deref for IStream {
	type Target = ISequentialStream;
	fn deref(&self) -> &Self::Target {
		self.as_parent_ref()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ISequentialStream {
	pub interface: InterfaceOwned<ISequentialStream>,
}

unsafe impl Interface for ISequentialStream {
	/// {0C733A30-2A1C-11CE-ADE5-00AA0044773D}
	const IID: GUID = GUID::from_values(0x0c733a30, 0x2a1c, 0x11ce, [0xad, 0xe5,
		0x00, 0xaa, 0x00, 0x44, 0x77, 0x3d,
	]);

	type Owned = Self;
	type Vtable = ISequentialStream_Vtbl;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}
unsafe impl InterfacePtr for ISequentialStream {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfacePtr::from_nn(ptr),
		}
	}
}
unsafe impl InterfaceBase<IUnknown> for ISequentialStream {}
unsafe impl InterfaceBase<ISequentialStream> for ISequentialStream {}

#[repr(C)]
pub struct ISequentialStream_Vtbl {
	pub base__: IUnknown_Vtbl,
	pub Read: unsafe extern "system" fn(this: *mut InterfaceTarget, buffer: *mut u8, size: u32, read: *mut u32) -> HRESULT,
	pub Write: unsafe extern "system" fn(this: *mut InterfaceTarget, buffer: *const u8, size: u32, written: *mut u32) -> HRESULT,
}

impl AsRef<IUnknown_Vtbl> for ISequentialStream_Vtbl {
	fn as_ref(&self) -> &IUnknown_Vtbl {
		&self.base__
	}
}
