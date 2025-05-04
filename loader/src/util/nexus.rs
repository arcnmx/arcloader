use std::{ffi::CStr, fmt, hash::Hash, mem::transmute, ops::{Deref, DerefMut}, ptr};
use nexus::{addon::{self, AddonDefinition}, AddonFlags};
#[cfg(windows)]
use windows::Win32::{Foundation::{ERROR_NOT_SUPPORTED, HMODULE}, System::LibraryLoader::GetProcAddress};
#[cfg(windows)]
use crate::util::win::{WinResult, WinError};

use super::ffi::cstr_opt;

pub type NexusId = i32;
pub type AddonApiVersion = i32;

pub type GetAddonDef = unsafe extern "system-unwind" fn() -> *const AddonDefinition;

#[cfg(windows)]
pub fn get_addon_def_addr(module: HMODULE) -> Option<GetAddonDef> {
	unsafe {
		GetProcAddress(module, windows_strings::s!("GetAddonDef"))
			.map(|f| transmute(f))
	}
}

#[cfg(windows)]
pub fn get_addon_def(module: &HMODULE) -> WinResult<(&AddonDesc, AddonApiVersion)> {
	let def_fn = get_addon_def_addr(*module)
		.ok_or_else(|| WinError::new(ERROR_NOT_SUPPORTED.to_hresult(), "Nexus GetAddonDef missing"))?;
	let def = unsafe {
		def_fn()
	};

	let api_version = unsafe { *ptr::addr_of!((*def).api_version) };

	Ok((unsafe { AddonDesc::with_ptr(def) }, api_version))
}

#[derive(Copy, Clone)]
pub struct AddonDesc {
	def: AddonDefinition,
}

impl AddonDesc {
	pub const unsafe fn new(def: AddonDefinition) -> Self {
		Self {
			def,
		}
	}

	pub unsafe fn with_ptr(def: *const AddonDefinition) -> &'static Self {
		transmute(def)
	}

	pub unsafe fn with_ref(def: &AddonDefinition) -> &Self {
		transmute(def)
	}

	pub unsafe fn with_mut(def: &mut AddonDefinition) -> &mut Self {
		transmute(def)
	}

	pub fn ptr_from(def: *const AddonDefinition) -> *const Self {
		def as *const Self
	}

	pub fn ptr_from_mut(def: *mut AddonDefinition) -> *mut Self {
		def as *mut Self
	}

	#[inline]
	pub fn def(&self) -> &AddonDefinition {
		&self.def
	}

	pub unsafe fn def_mut(&mut self) -> &mut AddonDefinition {
		&mut self.def
	}

	pub fn is_raidcore(&self) -> bool {
		self.def().signature >= 0
	}

	pub fn can_hotload(&self) -> bool {
		!self.def().flags.contains(AddonFlags::DisableHotloading)
	}

	pub fn name(&self) -> &CStr {
		unsafe {
			cstr_opt(&self.def().name)
		}.unwrap_or_default()
	}

	pub fn author(&self) -> Option<&CStr> {
		unsafe { cstr_opt(&self.def().author) }
	}

	pub fn description(&self) -> Option<&CStr> {
		unsafe { cstr_opt(&self.def().description) }
	}

	pub fn update_link(&self) -> Option<&CStr> {
		unsafe { cstr_opt(&self.def().update_link) }
	}

	pub fn version(&self) -> &AddonVersion {
		AddonVersion::from_ref(&self.def().version)
	}
}

impl Deref for AddonDesc {
	type Target = AddonDefinition;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.def()
	}
}

impl fmt::Display for AddonDesc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = self.name().to_string_lossy();
		write!(f, "{name} {}",
			format_args!("{}.{}.{}-{}", self.def.version.major, self.def.version.minor, self.def.version.revision, self.def.version.build),
		)?;
		
		#[cfg(todo)]
		if let Some(author) = self.author() {
			let author = self.name().to_string_lossy();
			write!(f, " by {author}")?;
		}

		Ok(())
	}
}

impl fmt::Debug for AddonDesc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut f = f.debug_struct("AddonDesc");
		f
			.field("desc", &format_args!("{}", self))
			.field("def", &self.def)
			.finish()
	}
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AddonVersion {
	pub version: addon::AddonVersion,
}

impl AddonVersion {
	pub const fn new(major: i16, minor: i16, revision: i16, build: i16) -> Self {
		Self::with_version(addon::AddonVersion {
			major,
			minor,
			revision,
			build,
		})
	}

	pub const fn with_version(version: addon::AddonVersion) -> Self {
		Self {
			version,
		}
	}

	pub const fn from_ref(version: &addon::AddonVersion) -> &Self {
		unsafe {
			transmute(version)
		}
	}

	pub fn from_mut(version: &mut addon::AddonVersion) -> &mut Self {
		unsafe {
			transmute(version)
		}
	}
}

impl Deref for AddonVersion {
	type Target = addon::AddonVersion;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.version
	}
}

impl DerefMut for AddonVersion {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.version
	}
}

impl fmt::Display for AddonVersion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}.{}.{}-{}", self.major, self.minor, self.revision, self.build)
	}
}

impl fmt::Debug for AddonVersion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self.version, f)
	}
}

impl From<addon::AddonVersion> for AddonVersion {
	fn from(v: addon::AddonVersion) -> Self {
		Self::with_version(v)
	}
}

impl From<AddonVersion> for addon::AddonVersion {
	fn from(v: AddonVersion) -> Self {
		v.version
	}
}
