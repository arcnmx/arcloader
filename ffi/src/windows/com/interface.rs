use core::{ffi::c_void, mem::{forget, transmute}, ptr::NonNull};
use crate::windows::{
	com::{
		unknown::{IUnknown, IUnknownImpl},
		InterfaceRef,
		InterfaceOwned,
	},
	core::{Result, GUID},
};

pub type InterfaceTarget = c_void;

pub unsafe trait Interface {
	const IID: GUID;

	type Owned: Interface<Owned = Self::Owned, Vtable = Self::Vtable> where
		Self: Sized;

	fn as_raw(&self) -> *mut InterfaceTarget;

	fn as_nn(&self) -> NonNull<InterfaceTarget> {
		unsafe {
			NonNull::new_unchecked(self.as_raw())
		}
	}

	fn into_nn(self) -> NonNull<InterfaceTarget> where
		Self: Sized,
	{
		let p = self.as_nn();
		forget(self);
		p
	}

	fn into_raw(self) -> *mut InterfaceTarget where
		Self: Sized,
	{
		self.into_nn().as_ptr()
	}

	type Vtable;

	fn vtable(&self) -> &Self::Vtable {
		unsafe {
			self.assume_vtable::<Self>()
		}
	}

	unsafe fn assume_vtable<I: ?Sized + Interface>(&self) -> &I::Vtable {
		let vtable = self.as_raw() as *mut NonNull<I::Vtable>;
		transmute(*vtable)
	}

	fn cast<I: InterfacePtr + InterfaceBase<IUnknown>>(&self) -> Result<InterfaceOwned<I>> where
		Self: InterfaceBase<IUnknown>,
	{
		self.as_parent().query::<I>()
		/*let mut out = None;
		self.as_unknown().QueryInterface(&I::IID, &mut out)
			.ok()
			.and_then(|()| out.ok_or_else(|| E_POINTER.into()))
			.map(|p| unsafe { I::from_raw(p.as_ptr()) })*/
	}

	fn to_ref(&self) -> InterfaceRef<'_, Self::Owned> where
		Self: InterfacePtr,
	{
		unsafe {
			let p = self.as_raw();
			InterfaceRef::from_raw(NonNull::new_unchecked(p))
		}
	}

	fn to_parent<P: Interface>(&self) -> InterfaceRef<'_, P> where
		Self: InterfaceBase<P> + InterfacePtr,
	{
		let p = self.as_parent().as_raw();
		unsafe {
			InterfaceRef::from_raw(NonNull::new_unchecked(p))
		}
	}

	fn to_canon(self) -> Self::Owned where
		Self: Sized,
		Self::Owned: InterfacePtr,
	{
		unsafe {
			InterfacePtr::from_nn(self.into_nn())
		}
	}
}

pub unsafe trait InterfaceBase<P: Interface>: Interface {
	fn as_parent(&self) -> InterfaceRef<'_, P> {
		unsafe {
			let p = NonNull::new_unchecked(self.as_raw());
			InterfaceRef::from_raw(p)
		}
	}

	fn parent_vtable(&self) -> &P::Vtable {
		unsafe {
			self.assume_vtable::<P>()
		}
	}

	fn as_parent_ref(&self) -> &P where
		Self: InterfacePtr,
		P: InterfacePtr,
	{
		unsafe {
			let p = self.nn_borrowed();
			InterfacePtr::from_nn_borrowed(p)
		}
	}
}

pub unsafe trait InterfaceAs<P: Interface>: Interface {
	fn get_parent(&self) -> InterfaceRef<'_, P>;

	fn get_parent_vtable(&self) -> &P::Vtable;
}

unsafe impl<P, I> InterfaceAs<P> for I where
	P: Interface,
	I: InterfaceBase<P>,
{
	fn get_parent(&self) -> InterfaceRef<'_, P> {
		self.as_parent()
	}

	fn get_parent_vtable(&self) -> &P::Vtable {
		self.parent_vtable()
	}
}

pub unsafe trait InterfacePtr: Sized + Interface {
	unsafe fn from_raw(ptr: *mut InterfaceTarget) -> Self {
		Self::from_nn(NonNull::new_unchecked(ptr))
	}

	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self;
	unsafe fn try_from_raw(ptr: *mut InterfaceTarget) -> Option<Self> {
		NonNull::new(ptr)
			.map(|p| Self::from_nn(p))
	}

	unsafe fn from_raw_borrowed<'p>(ptr: &'p *mut InterfaceTarget) -> Option<&'p Self> {
		let ptr: &'p Option<NonNull<InterfaceTarget>> = transmute(ptr);
		ptr.as_ref().map(|p| Self::from_nn_borrowed(p))
	}

	unsafe fn from_nn_borrowed(ptr: &NonNull<InterfaceTarget>) -> &Self {
		transmute(ptr)
	}

	unsafe fn nn_borrowed(&self) -> &NonNull<InterfaceTarget> {
		transmute(self)
	}
}
