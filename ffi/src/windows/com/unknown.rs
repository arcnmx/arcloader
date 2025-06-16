use core::{cmp::Ordering, hash, fmt, mem::transmute, ptr::NonNull};
use crate::{
	nonnull_ref,
	windows::{
		adapter::{windows_adapter, windows_newtype},
		com::{
			interface::{Interface, InterfacePtr, InterfaceBase, InterfaceTarget},
			InterfaceOwned, InterfaceRef,
			E_POINTER, E_NOINTERFACE,
		},
		core::{Result, GUID, HRESULT},
	},
};

pub trait IUnknownImpl: Interface + InterfaceBase<IUnknown> {
	fn AddRef(&self) -> u32 {
		self.as_parent().AddRef()
	}

	unsafe fn Release(&self) -> u32 {
		self.as_parent().Release()
	}

	fn QueryInterface(&self, iid: &GUID, interface: &mut Option<NonNull<InterfaceTarget>>) -> HRESULT {
		self.as_parent().QueryInterface(iid, interface)
	}

	fn query<I: InterfaceBase<IUnknown>>(&self) -> Result<InterfaceOwned<I>> where
		I: InterfacePtr,
	{
		let mut interface: Option<InterfaceOwned<I>> = None;
		self.QueryInterface(&I::IID, unsafe { transmute(&mut interface) }).ok()
			.and_then(|()| interface.ok_or(E_POINTER.into()))
	}

	fn ptr_eq(&self, rhs: &Self) -> bool {
		self.as_raw() == rhs.as_raw() ||
			self.cast::<IUnknown>().ok().map(|l| l.as_raw()) == rhs.cast::<IUnknown>().ok().map(|r| r.as_raw())
	}
}

impl IUnknown {
	#[inline(always)]
	pub const unsafe fn from_raw(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			unknown: InterfaceOwned::from_raw(ptr)
		}
	}
}

unsafe impl Interface for IUnknown {
	const IID: GUID = GUID::from_values(0, 0, 0, [0xc0, 0, 0, 0, 0, 0, 0, 0x46]);
	type Vtable = IUnknown_Vtbl;
	type Owned = Self;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.unknown.as_raw()
	}
}

unsafe impl InterfacePtr for IUnknown {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self::from_raw(ptr)
	}
}

impl IUnknownImpl for IUnknown {
	fn AddRef(&self) -> u32 {
		unsafe {
			(self.vtable().AddRef)(self.as_raw())
		}
	}

	unsafe fn Release(&self) -> u32 {
		(self.vtable().Release)(self.as_raw())
	}

	fn QueryInterface(&self, iid: &GUID, interface: &mut Option<NonNull<InterfaceTarget>>) -> HRESULT {
		unsafe {
			(self.vtable().QueryInterface)(self.as_raw(), iid, interface as *mut _ as *mut *mut _)
		}
	}
}

unsafe impl InterfaceBase<IUnknown> for IUnknown {
}

pub type IUnknown_Vtbl_QueryInterface = unsafe extern "system" fn(this: *mut InterfaceTarget, iid: *const GUID, interface: *mut *mut InterfaceTarget) -> HRESULT;
pub type IUnknown_Vtbl_AddRef = unsafe extern "system" fn(this: *mut InterfaceTarget) -> u32;
pub type IUnknown_Vtbl_Release = unsafe extern "system" fn(this: *mut InterfaceTarget) -> u32;
#[repr(C)]
pub struct IUnknown_Vtbl {
	pub QueryInterface: IUnknown_Vtbl_QueryInterface,
	pub AddRef: IUnknown_Vtbl_AddRef,
	pub Release: IUnknown_Vtbl_Release,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct IUnknown {
	pub unknown: InterfaceOwned<IUnknown>,
}

impl IUnknown {
	pub const UNIMPLEMENTED_REF: InterfaceRef<'static, IUnknown> = unsafe {
		InterfaceRef::from_raw(nonnull_ref(&&Self::UNIMPLEMENTED_VTBL).cast())
	};
	pub const UNIMPLEMENTED: IUnknown = IUnknown {
		unknown: unsafe {
			InterfaceOwned::from_raw(Self::UNIMPLEMENTED_REF.raw())
		},
	};
	pub const UNIMPLEMENTED_VTBL: IUnknown_Vtbl = IUnknown_Vtbl {
		QueryInterface: unimplemented_query,
		AddRef: unimplemented_add_ref,
		Release: unimplemented_release,
	};

	pub fn query_raw(&self) -> *mut InterfaceTarget {
		self.cast::<IUnknown>().ok()
			.map(|unk| unk.as_raw())
			.unwrap_or(self.unknown.as_raw())
	}

	pub fn ptr_eq(&self, rhs: &Self) -> bool {
		self.as_raw() == rhs.as_raw() ||
			self.query_raw() == rhs.query_raw()
	}
}

impl fmt::Debug for IUnknown {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("IUnknown")
			.field(&self.unknown.as_raw())
			.finish()
	}
}

impl PartialEq for IUnknown {
	fn eq(&self, rhs: &Self) -> bool {
		self.ptr_eq(rhs)
	}
}
impl Eq for IUnknown {}

impl PartialOrd for IUnknown {
	fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
		Some(self.cmp(rhs))
	}
}
impl Ord for IUnknown {
	fn cmp(&self, rhs: &Self) -> Ordering {
		self.query_raw().cmp(&rhs.query_raw())
	}
}
impl hash::Hash for IUnknown {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		hash::Hash::hash(&self.query_raw(), state)
	}
}

pub unsafe extern "system" fn unimplemented_add_ref(_this: *mut InterfaceTarget) -> u32 {
	// you are not special...
	2
}

pub  unsafe extern "system" fn unimplemented_release(_this: *mut InterfaceTarget) -> u32 {
	// ... and you never will be
	2
}

pub unsafe extern "system" fn unimplemented_query(_this: *mut InterfaceTarget, _iid: *const GUID, _interface: *mut *mut InterfaceTarget) -> HRESULT {
	E_NOINTERFACE
}

windows_newtype! {
	impl From for core::IUnknown(pub Self);
}
windows_newtype! {
	impl From for core::IUnknown_Vtbl(pub Self);
}

windows_adapter! { pub(crate) mod core as core0xx =>
	use core0xx::{IUnknown, imp::CanInto};
	impl CanInto<IUnknown> for super::IUnknown {
	}
	impl CanInto<super::IUnknown> for IUnknown {
		const QUERY: bool = false;
	}
}
