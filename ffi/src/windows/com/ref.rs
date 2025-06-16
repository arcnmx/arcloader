use core::{cmp::Ordering, fmt, hash, marker::PhantomData, mem::transmute, ops, ptr::NonNull};
use crate::windows::{
	com::{
		interface::{Interface, InterfacePtr, InterfaceTarget, InterfaceBase},
		unknown::{IUnknown, IUnknownImpl},
		InterfaceOwned,
	},
	core::GUID,
};
#[cfg(feature = "windows-core-060")]
use crate::windows::core060;
#[cfg(feature = "windows-core-061")]
use crate::windows::core061;

#[repr(transparent)]
pub struct InterfaceRef<'i, I: ?Sized> {
	ptr: NonNull<InterfaceTarget>,
	borrow: PhantomData<(&'i InterfaceTarget, *const I)>,
}

impl<'i, I> InterfaceRef<'i, I> {
	#[inline(always)]
	pub const unsafe fn from_raw(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			ptr,
			borrow: PhantomData,
		}
	}

	#[inline(always)]
	pub const fn raw(self) -> NonNull<InterfaceTarget> {
		unsafe {
			transmute(self)
		}
	}

	#[inline(always)]
	pub const fn raw_ref(&self) -> &NonNull<InterfaceTarget> {
		unsafe {
			transmute(self)
		}
	}

	#[inline(always)]
	pub unsafe fn raw_mut(&mut self) -> &mut NonNull<InterfaceTarget> {
		unsafe {
			transmute(self)
		}
	}
}

impl<'i, I: Interface> InterfaceRef<'i, I> {
	#[inline(always)]
	pub fn from_interface(i: &'i I) -> Self {
		unsafe {
			let p = NonNull::new_unchecked(i.as_raw());
			Self::from_raw(p)
		}
	}

	#[inline(always)]
	pub fn interface_ref(&self) -> &I where
		I: InterfacePtr,
	{
		unsafe {
			transmute(self)
		}
	}

	pub fn parent<P: Interface>(&self) -> InterfaceRef<'i, P> where
		I: InterfaceBase<P> + InterfacePtr,
	{
		let p = self.interface_ref().as_parent().as_raw();
		unsafe {
			InterfaceRef::from_raw(NonNull::new_unchecked(p))
		}
	}
}

impl<'i, I: IUnknownImpl> InterfaceRef<'i, I> {
	pub fn to_owned(self) -> InterfaceOwned<I> where
		I: InterfacePtr,
	{
		unsafe {
			let i = self.interface_ref();
			i.AddRef();
			let i = i.as_raw();
			let p = NonNull::new_unchecked(i);
			InterfaceOwned::from_raw(p)
		}
	}

	pub fn as_unknown(&self) -> &IUnknown {
		unsafe {
			IUnknown::from_nn_borrowed(&self.ptr)
		}
	}

	pub fn to_unknown(self) -> IUnknown {
		self.as_unknown().clone()
	}
}

#[cfg(feature = "windows-core-060")]
impl<'i, I: IUnknownImpl> core060::Free for InterfaceRef<'i, I> {
	unsafe fn free(&mut self) {
		self.as_unknown().Release();
	}
}
#[cfg(feature = "windows-core-061")]
impl<'i, I: IUnknownImpl> core061::Free for InterfaceRef<'i, I> {
	unsafe fn free(&mut self) {
		self.as_unknown().Release();
	}
}

impl<'i, I: InterfacePtr> ops::Deref for InterfaceRef<'i, I> {
	type Target = I;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		self.interface_ref()
	}
}

impl<'i, I> Copy for InterfaceRef<'i, I> {}
impl<'i, I> Clone for InterfaceRef<'i, I> {
	#[inline(always)]
	fn clone(&self) -> Self {
		*self
	}
}

unsafe impl<'i, I: Sync> Sync for InterfaceRef<'i, I> {}
unsafe impl<'i, I: Send> Send for InterfaceRef<'i, I> {}

impl<'i, I> PartialEq for InterfaceRef<'i, I> {
	fn eq(&self, rhs: &Self) -> bool {
		self.ptr.eq(&rhs.ptr)
	}
}
impl<'i, I> Eq for InterfaceRef<'i, I> {}
impl<'i, I> PartialOrd for InterfaceRef<'i, I> {
	fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
		self.ptr.partial_cmp(&rhs.ptr)
	}
}
impl<'i, I> Ord for InterfaceRef<'i, I> {
	fn cmp(&self, rhs: &Self) -> Ordering {
		self.ptr.cmp(&rhs.ptr)
	}
}
impl<'i, I> hash::Hash for InterfaceRef<'i, I> {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		hash::Hash::hash(&self.ptr, state)
	}
}

impl<'i, I: Interface> fmt::Debug for InterfaceRef<'i, I> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("InterfaceRef")
			.field(&format_args!("{}", I::IID))
			.field(&self.ptr)
			.finish()
	}
}

unsafe impl<'i, I: Interface> Interface for InterfaceRef<'i, I> {
	const IID: GUID = I::IID;
	type Vtable = I::Vtable;

	type Owned = I::Owned;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.ptr.as_ptr()
	}
}

unsafe impl<'i, P, I: InterfaceBase<P>> InterfaceBase<P> for InterfaceRef<'i, I> where
	P: Interface,
	I: InterfacePtr,
{
	fn as_parent(&self) -> InterfaceRef<P> {
		self.parent()
	}
}

unsafe impl<'i, I: Interface> InterfacePtr for InterfaceRef<'i, I> {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self::from_raw(ptr)
	}
}
