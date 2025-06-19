use core::{
	ptr,
	sync::atomic::{AtomicPtr, Ordering},
};
use crate::{
	c_void,
	cstr::{CStrRef, CStrPtr, CStrPtr16},
	windows::{
		adapter::windows_adapter,
		core::{Error, Result, HRESULT},
		Win32::{
			Foundation::{
				HMODULE,
				ERROR_MOD_NOT_FOUND,
				FARPROC,
			},
		},
	},
};
pub use crate::windows::Win32::System::LibraryLoader::LOAD_LIBRARY_FLAGS;

pub const ERROR_UNIMPLEMENTED: Error = Error::with_hresult(crate::windows::Win32::Foundation::ERROR_CALL_NOT_IMPLEMENTED.to_hresult());

pub fn load_library_a<'n, N: Into<CStrPtr<'n>>>(name: N, flags: LOAD_LIBRARY_FLAGS) -> Result<HMODULE> {
	let name = name.into();
	unsafe {
		windows_adapter! { match windows as windows0xx
			=> windows0xx::Win32::System::LibraryLoader::LoadLibraryExA(windows0xx::core::PCSTR(name.as_ptr() as *const _), None, flags.into())
				.map(HMODULE::from)
				.map_err(Error::from),
			_ => {
				#[allow(unreachable_patterns)]
				let res = match () {
					#[cfg(feature = "windows-core")]
					() => HMODULE::from(crate::windows::core0xx::imp::LoadLibraryExA(name.as_ptr() as *const _, ptr::null_mut::<c_void>().into(), flags.into())),
					#[cfg(feature = "windows-link")]
					() => {
						windows_link::link!("kernel32.dll" "system" fn LoadLibraryExA(name: *const crate::c_char, handle: *mut crate::c_void, flags: LOAD_LIBRARY_FLAGS) -> HMODULE);
						LoadLibraryExA(name.as_ptr(), ptr::null_mut(), flags)
					},
					_ => return Err(ERROR_UNIMPLEMENTED),
				};
				match res {
					m if m.is_invalid() => Err(Error::from_win32()),
					m => Ok(m),
				}
			}
		}
	}
}

pub fn load_library_w<'n, N: Into<CStrPtr16<'n>>>(name: N, flags: LOAD_LIBRARY_FLAGS) -> Result<HMODULE> {
	let name = name.into();
	unsafe {
		windows_adapter! { match windows as windows0xx
			=> windows0xx::Win32::System::LibraryLoader::LoadLibraryExW(windows0xx::core::PCWSTR(name.as_ptr() as *const _), None, flags.into())
				.map(HMODULE::from)
				.map_err(Error::from),
			_ => {
				let res = match () {
					#[cfg(feature = "windows-link")]
					() => {
						windows_link::link!("kernel32.dll" "system" fn LoadLibraryExW(name: *const crate::c_wchar, handle: *mut crate::c_void, flags: LOAD_LIBRARY_FLAGS) -> HMODULE);
						LoadLibraryExW(name.as_ptr(), ptr::null_mut(), flags)
					},
					#[cfg(not(feature = "windows-link"))]
					_ => return Err(ERROR_UNIMPLEMENTED),
				};
				match res {
					m if m.is_invalid() => Err(Error::from_win32()),
					m => Ok(m),
				}
			}
		}
	}
}

pub unsafe fn free_library(library: HMODULE) -> Result<()> {
	unsafe {
		windows_adapter! { match windows as windows0xx
			=> windows0xx::Win32::Foundation::FreeLibrary(library.into())
				.map_err(Error::from),
			_ => {
				use crate::c_bool32;

				#[allow(unreachable_patterns)]
				let res = match () {
					#[cfg(feature = "windows-core")]
					() => c_bool32::from(crate::windows::core0xx::imp::FreeLibrary(library.into())),
					#[cfg(feature = "windows-link")]
					() => {
						windows_link::link!("kernel32.dll" "system" fn FreeLibrary(module: HMODULE) -> c_bool32);
						FreeLibrary(library)
					},
					_ => return Err(ERROR_UNIMPLEMENTED),
				};
				match res.get() {
					false => Err(Error::from_win32()),
					true => Ok(()),
				}
			}
		}
	}
}

pub fn get_proc_address<'n, N: Into<CStrPtr<'n>>>(library: HMODULE, name: N) -> FARPROC {
	let name = name.into();
	unsafe {
		windows_adapter! { match windows as windows0xx
			=> windows0xx::Win32::System::LibraryLoader::GetProcAddress(library.into(), windows0xx::core::PCSTR(name.as_ptr() as *const _)),
			_ => {
				#[allow(unreachable_patterns)]
				match () {
					#[cfg(feature = "windows-core")]
					() => crate::windows::core0xx::imp::GetProcAddress(library.into(), name.as_ptr() as *const _),
					#[cfg(feature = "windows-link")]
					() => {
						windows_link::link!("kernel32.dll" "system" fn GetProcAddress(module: HMODULE, name: *const crate::c_char) -> FARPROC);
						GetProcAddress(library, name.as_ptr())
					},
					_ => None,
				}
			}
		}
	}
}

pub struct ImportLibraryCell {
	module: AtomicPtr<c_void>,
}

impl ImportLibraryCell {
	pub const ERROR_NOT_FOUND: HRESULT = ERROR_MOD_NOT_FOUND.to_hresult();

	pub const fn new() -> Self {
		Self {
			module: AtomicPtr::new(ptr::null_mut()),
		}
	}

	pub unsafe fn store(&self, v: HMODULE) {
		self.module.store(v.0, Ordering::SeqCst);
	}

	pub fn module_from_ptr(p: *mut c_void) -> Result<HMODULE> {
		match p {
			p if p == ptr::dangling_mut() =>
				Err(Self::ERROR_NOT_FOUND.into()),
			p => Ok(HMODULE(p)),
		}
	}

	pub fn get(&self) -> Result<HMODULE> {
		let p = self.module.load(Ordering::Relaxed);
		Self::module_from_ptr(p)
	}

	pub unsafe fn get_or_load(&self, name: &CStrRef, flags: LOAD_LIBRARY_FLAGS) -> Result<HMODULE> {
		match self.get() {
			Err(e) => return Err(e),
			Ok(handle) if !handle.0.is_null() =>
				return Ok(handle),
			_ => (),
		}

		let (module, res) = match load_library_a(name, flags) {
			//Ok(m) if m.is_invalid() => (m, Err(Self::ERROR_NOT_FOUND.into())),
			Ok(m) => (m, Ok(())),
			Err(e) =>
				(HMODULE(ptr::dangling_mut()), Err(e)),
		};

		match self.module.compare_exchange(ptr::null_mut(), module.0, Ordering::SeqCst, Ordering::SeqCst) {
			Ok(_) => res.map(|()| module),
			Err(prev) => {
				if let Ok(..) = res {
					// undo what we've done
					let _res = free_library(module);
				}
				Self::module_from_ptr(prev)
			},
		}
	}

	pub fn unload(&self) -> Result<()> {
		let prev = self.module.swap(ptr::null_mut(), Ordering::SeqCst);
		match Self::module_from_ptr(prev) {
			Ok(m) if !m.is_invalid() => unsafe {
				free_library(m)
			},
			_ => Ok(()),
		}
	}

	pub fn shutdown(&self) -> Result<()> {
		let prev = self.module.swap(ptr::dangling_mut(), Ordering::SeqCst);

		match Self::module_from_ptr(prev) {
			Ok(m) if !m.is_invalid() => unsafe {
				free_library(m)
			},
			_ => Ok(()),
		}
	}
}

#[cfg(todo)]
#[cfg(all(windows, feature = "library"))]
#[macro_export]
macro_rules! windows_link {
	(@link($dll:literal)($proc:literal)
		$vis:vis unsafe extern $abi:tt
		fn $name:ident($($args:tt)*);
	) => {
		$crate::windows::adapter::windows_link! { @link($dll)($proc)
			$vis unsafe extern $abi
			fn $name($($args)*) -> ()
		}
	};
	(@link($dll:literal)($proc:literal)
		$vis:vis unsafe extern $abi:tt
		fn $name:ident($($arg:ident: $arg_ty:ty),*$(,)?)
		-> $res:ty;
	) => {
		$vis unsafe fn $name() -> (unsafe extern $abi fn($($arg: $arg_ty,)*) -> $res) {
			//type LinkFn = unsafe extern $abi fn($($arg: $arg_ty,)*) -> $res;
			// TODO: just make this a real struct thanks
			const DLL_NAME: &'static ::std::ffi::CStr = $crate::cstr!($dll);
			const PROC_NAME: &'static ::std::ffi::CStr = $crate::cstr!($name);
			static LINK_CACHE: ::std::sync::atomic::AtomicPtr<$crate::c_void> = ::std::sync::atomic::AtomicPtr::new(::core::ptr::null_mut());
			let ordering = ::std::sync::atomic::Ordering::Relaxed;
			if let Some(p) = ::core::ptr::NonNull::new(LINK_CACHE.load(ordering)) {
				return unsafe {
					::core::mem::transmute(p)
				}
			}

			let lib = unsafe {
				$crate::windows::Win32::System::LibraryLoader::LoadLibraryExA(&$crate::CStrPtr::with_cstr(DLL_NAME), None, 0)
			}.ok()?;
			let proc = unsafe {
				$crate::windows::Win32::System::LibraryLoader::GetProcAddress(&$crate::CStrPtr::with_cstr(PROC_NAME))
			}?;
			LINK_CACHE.store(proc as usize as *mut $crate::c_void, ordering);
			::core::mem::transmute(proc)
		}
	};
	($dll:literal $abi:tt $vis:vis fn $name:ident($($args:tt)*) $(-> $res:ty)?) => {
		$crate::windows::adapter::windows_link! {
			@link($dll)(stringify!($name)) $vis unsafe extern $abi fn $name($($args)*) $(-> $res)?;
		}
	};
}
#[cfg(todo)]
#[cfg(all(windows, feature = "library"))]
pub use windows_link;
