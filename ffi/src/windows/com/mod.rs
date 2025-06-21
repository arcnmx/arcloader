use crate::windows::{
	adapter::{windows_adapter, windows_newtype},
	core::{Result, HRESULT, GUID},
};
use ::core::ops;

pub const CO_E_NOTINITIALIZED: HRESULT = HRESULT::CO_NOTINITIALIZED;
pub const E_INVALIDARG: HRESULT = HRESULT::INVALIDARG;
pub const E_NOINTERFACE: HRESULT = HRESULT::NOINTERFACE;
pub const E_POINTER: HRESULT = HRESULT::POINTER;

pub mod interface;
pub mod imp;
pub mod owned;
mod r#ref;
pub mod unknown;
pub mod stream;

pub use self::{
	interface::{Interface, InterfacePtr, InterfaceTarget},
	owned::{InterfaceOwned, InterfaceRc},
	r#ref::InterfaceRef,
	unknown::IUnknown,
};

pub type Known<I> = InterfaceOwned<I>;

windows_adapter! { pub mod core as core0xx =>
	use core0xx::Interface;
	use crate::windows::{
		com,
		core::{self, Result}
	};
	use ::core::{mem::transmute, ptr::NonNull};

	pub fn create_guid() -> Result<core::GUID> {
		unsafe {
			core0xx::imp::CoCreateGuid()
				.map(Into::into)
				.map_err(Into::into)
		}
	}

	unsafe impl<'a, I: Interface + 'static> com::Interface for core0xx::InterfaceRef<'a, I> {
		const IID: core::GUID = unsafe {
			transmute(I::IID)
		};

		fn as_raw(&self) -> *mut com::InterfaceTarget {
			I::as_raw(self)
		}

		type Owned = InterfaceOwned<I> where
			Self: Sized;

		// TODO: cast?

		type Vtable = I::Vtable;

		fn vtable(&self) -> &Self::Vtable {
			I::vtable(self)
		}

		#[cfg(todo)]
		unsafe fn assume_vtable<I: ?Sized + com::Interface>(&self) -> &I::Vtable {
			com::InterfaceOwned::<I>::assume_vtable::<I>(self)
		}
	}

	unsafe impl<'a, I: Interface> com::interface::InterfaceBase<com::IUnknown> for core0xx::InterfaceRef<'a, I> where
		Self: com::Interface,
	{
		fn as_parent(&self) -> com::InterfaceRef<'_, com::IUnknown> {
			match I::UNKNOWN {
				true => unsafe {
					com::InterfaceRef::from_raw(NonNull::new_unchecked(self.as_raw()))
				},
				false => com::IUnknown::UNIMPLEMENTED_REF,
			}
		}
	}

	unsafe impl<'a, I: Interface> com::InterfacePtr for core0xx::InterfaceRef<'a, I> where
		Self: com::Interface,
	{
		unsafe fn from_nn(ptr: NonNull<com::InterfaceTarget>) -> Self {
			Self::from_raw(ptr)
		}

		unsafe fn from_raw_borrowed<'p>(ptr: &'p *mut com::InterfaceTarget) -> Option<&'p Self> {
			let b: Option<&'p I> = I::from_raw_borrowed(ptr);
			transmute(b)
		}

		unsafe fn from_nn_borrowed<'p>(ptr: &'p NonNull<com::InterfaceTarget>) -> &'p Self {
			let ptr: &'p *mut com::InterfaceTarget = transmute(ptr);
			transmute(Self::from_raw_borrowed(ptr))
		}
	}

	#[cfg(todo)]
	impl<I: Interface + 'static> core0xx::imp::CanInto<I> for com::InterfaceOwned<core0xx::InterfaceRef<'static, I>> {
	}

	pub type InterfaceRef<'a, I> = com::InterfaceRef<'a, core0xx::InterfaceRef<'a, I>>;
	pub type InterfaceOwned<I> = com::owned::Irc<core0xx::InterfaceRef<'static, I>>;
	impl<I: Interface + 'static> From<I> for com::InterfaceOwned<core0xx::InterfaceRef<'static, I>> {
		fn from(v: I) -> Self {
			unsafe {
				let ptr = NonNull::new_unchecked(I::into_raw(v));
				com::InterfaceOwned::from_raw(ptr)
			}
		}
	}
}

windows_adapter! { pub mod windows as windows0xx =>
	use windows0xx::{
		core as core0xx,
		Win32::System::Com::{
			CoInitializeEx, CoUninitialize, CoCreateGuid,
			CoRegisterClassObject, CoRevokeClassObject,
			COINIT_MULTITHREADED,
			CLSCTX, REGCLS,
		},
	};
	use crate::windows::core::{self, Result};

	pub unsafe fn com_init_mta() -> Result<()> {
		CoInitializeEx(None, COINIT_MULTITHREADED).ok()
			.map_err(Into::into)
	}

	pub unsafe fn com_uninit() {
		CoUninitialize();
	}

	pub fn create_guid() -> Result<core::GUID> {
		unsafe {
			CoCreateGuid()
				.map_err(Into::into)
				.map(Into::into)
		}
	}

	pub unsafe fn register_class_object<'u, U>(clsid: &core::GUID, unk: U, context: CLSCTX, flags: REGCLS) -> Result<u32> where
		U: Into<core0xx::InterfaceRef<'u, core0xx::IUnknown>>,
	{
		let unk = unk.into();
		let clsid: &core0xx::GUID = clsid.into();
		unsafe {
			CoRegisterClassObject(clsid, unk, context, flags)
				.map_err(Into::into)
		}
	}

	pub unsafe fn revoke_class_object(registration: u32) -> Result<()> {
		unsafe {
			CoRevokeClassObject(registration)
				.map_err(Into::into)
		}
	}
}

#[doc(alias = "CoInitializeEx")]
pub unsafe fn com_init_mta() -> Result<()> {
	windows_adapter! { match self::windows as com0xx, _windows
		=> com0xx::com_init_mta(),
		_ => Err(CO_E_NOTINITIALIZED.into())
	}
}

#[doc(alias = "CoUninitialize")]
pub unsafe fn com_uninit() {
	windows_adapter! { match self::windows as com0xx, _windows
		=> com0xx::com_uninit(),
		_ => {
			//warn!("versioned windows feature required");
		},
	}
}

#[doc(alias = "CoCreateGuid")]
pub fn create_guid() -> Result<GUID> {
	windows_adapter! { match self::windows as com0xx, _windows
		=> com0xx::create_guid(),
		_ => windows_adapter! { match self::core as com0xx_core, _core
			=> com0xx_core::create_guid(),
			_ => Err(CO_E_NOTINITIALIZED.into())
		}
	}
}

#[doc(alias = "CoRegisterClassObject")]
pub unsafe fn register_class_object<'u, U>(clsid: &GUID, unk: &U, context: CLSCTX, flags: REGCLS) -> Result<u32> where
	U: interface::InterfaceAs<IUnknown>,
{
	let unk = unk.get_parent();
	windows_adapter! { match self::windows as com0xx, windows0xx
		=> com0xx::register_class_object(clsid, windows0xx::core::InterfaceRef::from_raw(unk.as_nn()), context.into(), flags.into()),
		_ => Err(CO_E_NOTINITIALIZED.into())
	}
}

#[doc(alias = "CoRevokeClassObject")]
pub unsafe fn revoke_class_object(registration: u32) -> Result<()> {
	windows_adapter! { match self::windows as com0xx, windows0xx
		=> com0xx::revoke_class_object(registration),
		_ => Err(CO_E_NOTINITIALIZED.into())
	}
}

windows_newtype! {
	pub struct Com::CLSCTX(pub u32);
}
impl CLSCTX {
	pub const INPROC_SERVER: Self = Self(0x01);
	pub const INPROC_HANDLER: Self = Self(0x02);
	pub const LOCAL_SERVER: Self = Self(0x04);
	pub const INPROC_SERVER16: Self = Self(0x08);
	pub const REMOTE_SERVER: Self = Self(0x10);
	pub const SERVER: Self = Self(Self::REMOTE_SERVER.0 | Self::LOCAL_SERVER.0 | Self::INPROC_SERVER.0);
	pub const ALL: Self = Self(Self::REMOTE_SERVER.0 | Self::LOCAL_SERVER.0 | Self::INPROC_HANDLER.0 | Self::INPROC_SERVER.0);
	pub const INPROC_HANDLER16: Self = Self(0x20);
	pub const RESERVED1: Self = Self(0x40);
	pub const RESERVED2: Self = Self(0x80);
	pub const RESERVED3: Self = Self(0x100);
	pub const RESERVED4: Self = Self(0x200);
	pub const NO_CODE_DOWNLOAD: Self = Self(0x400);
	pub const RESERVED5: Self = Self(0x800);
	pub const NO_CUSTOM_MARSHAL: Self = Self(0x1000);
	pub const ENABLE_CODE_DOWNLOAD: Self = Self(0x2000);
	pub const NO_FAILURE_LOG: Self = Self(0x4000);
	pub const DISABLE_AAA: Self = Self(0x8000);
	pub const ENABLE_AAA: Self = Self(0x10000);
	pub const FROM_DEFAULT_CONTEXT: Self = Self(0x20000);
	pub const ACTIVATE_32_BIT_SERVER: Self = Self(0x40000);
	pub const ACTIVATE_X86_SERVER: Self = Self::ACTIVATE_32_BIT_SERVER;
	pub const ACTIVATE_64_BIT_SERVER: Self = Self(0x80000);
	pub const ENABLE_CLOAKING: Self = Self(0x100000);
	pub const APPCONTAINER: Self = Self(0x400000);
	pub const ACTIVATE_AAA_AS_IU: Self = Self(0x800000);
	pub const ACTIVATE_ARM32_SERVER: Self = Self(0x2000000);
	pub const ALLOW_LOWER_TRUST_REGISTRATION: Self = Self(0x4000000);
	pub const RESERVED6: Self = Self(0x1000000);
	pub const PS_DLL: Self = Self(0x80000000);
}

windows_newtype! {
	pub struct Com::REGCLS(pub i32);
}
impl REGCLS {
	pub const SINGLEUSE: Self = Self(0x00);
	pub const MULTIPLEUSE: Self = Self(0x01);
	pub const MULTI_SEPARATE: Self = Self(0x02);
	pub const SUSPENDED: Self = Self(0x04);
	pub const SURROGATE: Self = Self(0x08);
	pub const AGILE: Self = Self(0x10);
}
impl ops::BitOr for REGCLS {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}
