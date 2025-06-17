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
#[cfg(all(windows, feature = "library"))]
pub use windows_link;
