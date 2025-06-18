use crate::windows::{
	adapter::windows_adapter,
	core::{Result, HRESULT, GUID},
};

pub const CO_E_NOTINITIALIZED: HRESULT = HRESULT(0x800401f0u32 as _);
pub const E_INVALIDARG: HRESULT = HRESULT(0x80070057u32 as _);
pub const E_NOINTERFACE: HRESULT = HRESULT(0x80004002u32 as _);
pub const E_POINTER: HRESULT = HRESULT(0x80004003u32 as _);

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
		Win32::System::Com::{CoInitializeEx, CoUninitialize, CoCreateGuid, COINIT_MULTITHREADED},
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
