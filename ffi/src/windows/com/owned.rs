use core::{cmp::Ordering, fmt, hash, mem::transmute, ops, ptr::NonNull};
use crate::windows::{
	com::{
		interface::{Interface, InterfacePtr, InterfaceBase, InterfaceTarget},
		unknown::{IUnknown, IUnknownImpl},
		InterfaceRef,
	},
	core::GUID,
};

pub trait InterfaceRc: Interface {
	fn ref_increment(&self) -> usize;
	unsafe fn ref_decrement(&self) -> usize;
	fn ref_clone(&self) -> Self where
		Self: Sized;
	unsafe fn ref_drop(&mut self) {
		self.ref_decrement();
	}

	fn ref_count(&self) -> usize;
}

impl<I: InterfaceBase<IUnknown> + Copy> InterfaceRc for I {
	fn ref_increment(&self) -> usize {
		self.as_parent().AddRef() as usize
	}

	unsafe fn ref_decrement(&self) -> usize {
		self.as_parent().Release() as usize
	}

	fn ref_clone(&self) -> Self where
		Self: Sized
	{
		self.ref_increment();
		*self
	}

	fn ref_count(&self) -> usize {
		self.ref_increment();
		unsafe {
			self.ref_decrement()
		}
	}
}

pub type InterfaceOwned<I> = Irc<InterfaceRef<'static, I>>;

#[repr(transparent)]
pub struct Irc<I: InterfaceRc> {
	interface: I,
}

impl<I: InterfaceRc> Irc<I> {
	#[inline(always)]
	pub fn new(interface: I::Owned) -> Self where
		I: InterfacePtr,
	{
		let p = interface.into_raw();
		unsafe {
			Self::new_unchecked(I::from_raw(p))
		}
	}

	#[inline(always)]
	pub const unsafe fn new_unchecked(interface: I) -> Self {
		Self {
			interface,
		}
	}

	#[inline(always)]
	pub fn as_interface(&self) -> &I where
		I: InterfacePtr,
	{
		unsafe {
			transmute(self)
			// TODO: bound unnecessary..? or is that what makes it safe??
			//&self.interface
		}
	}

	#[inline(always)]
	pub const unsafe fn interface_ref_unchecked(&self) -> &I {
		&self.interface
	}

	#[inline(always)]
	pub unsafe fn interface_mut_unchecked(&mut self) -> &mut I {
		&mut self.interface
	}
}

impl<I: InterfaceBase<IUnknown> + InterfacePtr> InterfaceOwned<I> {
	#[inline(always)]
	pub const unsafe fn from_raw(ptr: NonNull<InterfaceTarget>) -> Self {
		Self {
			interface: InterfaceRef::from_raw(ptr),
		}
	}

	#[inline(always)]
	pub const fn interface_ref<'a>(&'a self) -> &'a InterfaceRef<'a, I> {
		unsafe {
			transmute(&self.interface)
		}
	}
}

unsafe impl<I: Interface + InterfaceRc> Interface for Irc<I> {
	const IID: GUID = I::IID;
	type Vtable = I::Vtable;
	type Owned = I::Owned;

	fn as_raw(&self) -> *mut InterfaceTarget {
		self.interface.as_raw()
	}
}

unsafe impl<I: InterfacePtr + InterfaceRc> InterfacePtr for Irc<I> {
	unsafe fn from_nn(ptr: NonNull<InterfaceTarget>) -> Self {
		Self::new_unchecked(I::from_nn(ptr))
	}

	unsafe fn from_nn_borrowed(ptr: &NonNull<InterfaceTarget>) -> &Self {
		transmute(ptr)
	}
}

impl<I: InterfaceRc> Clone for Irc<I> {
	fn clone(&self) -> Self {
		Self {
			interface: self.interface.ref_clone(),
		}
	}
}

impl<I: InterfaceRc> Drop for Irc<I> {
	fn drop(&mut self) {
		unsafe {
			self.interface.ref_drop();
		}
	}
}

impl<'a, I: InterfaceRc + InterfacePtr> ops::Deref for Irc<I> {
	type Target = I;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		self.as_interface()
	}
}

impl<I: InterfaceRc + PartialEq> PartialEq for Irc<I> {
	fn eq(&self, rhs: &Self) -> bool {
		self.interface.eq(&rhs.interface)
	}
}
impl<I: InterfaceRc + Eq> Eq for Irc<I> {}

impl<I: InterfaceRc + PartialOrd> PartialOrd for Irc<I> {
	fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
		self.interface.partial_cmp(&rhs.interface)
	}
}
impl<I: InterfaceRc + Ord> Ord for Irc<I> {
	fn cmp(&self, rhs: &Self) -> Ordering {
		self.interface.cmp(&rhs.interface)
	}
}
impl<I: InterfaceRc + hash::Hash> hash::Hash for Irc<I> {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.interface.hash(state)
	}
}

impl<I: InterfaceRc> fmt::Debug for Irc<I> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("Interface")
			.field(&format_args!("{}", I::IID))
			.field(&self.interface.as_raw())
			.finish()
	}
}
