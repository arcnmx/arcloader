#[cfg(all(feature = "unwind", panic = "unwind"))]
#[macro_export]
macro_rules! extern_fns_impl {
	(@fn;
		$vis:vis unsafe extern "C" $($tt:tt)*
	) => {
		$vis unsafe extern "C-unwind" $($tt)*
	};
	(@fn;
		$vis:vis unsafe extern "system" $($tt:tt)*
	) => {
		$vis unsafe extern "system-unwind" $($tt)*
	};
	(@type(fn);
		$vis:vis type $id = unsafe extern "C" $($tt:tt)*
	) => {
		$vis type $id = unsafe extern "C-unwind" $($tt)*
	};
	(@type(fn);
		$vis:vis type $id = unsafe extern "system" $($tt:tt)*
	) => {
		$vis type $id = unsafe extern "system-unwind" $($tt)*
	};
}

#[cfg(any(not(feature = "unwind"), not(panic = "unwind")))]
#[macro_export]
macro_rules! extern_fns_impl {
	(@fn;
		$vis:vis unsafe extern $abi:tt $($tt:tt)*
	) => {
		$vis unsafe extern $abi $($tt)*
	};
	(@fn(type);
		$vis:vis type $id:ident = unsafe extern $abi:tt $($tt:tt)*
	) => {
		$vis type $id = unsafe extern $abi $($tt)*
	};
}
#[doc(hidden)]
pub use extern_fns_impl;

#[macro_export]
macro_rules! extern_fns {
	() => {};
	(
		$vis:vis unsafe extern $abi:tt fn $id:ident($($args:tt)*) $(-> $res:ty)? {
			$($body:tt)*
		}
		$($tt:tt)*
	) => {
		$crate::externs::extern_fns_impl! { @fn;
			$vis unsafe extern $abi fn $id($($args)*) $(-> $res)? {
				$($body)*
			}
		}

		$crate::externs::extern_fns! {
			$($tt)*
		}
	};
	(
		$vis:vis type $id:ident = unsafe extern $abi:tt fn($($args:tt)*) $(-> $res:ty)?;

		$($tt:tt)*
	) => {
		$crate::externs::extern_fns_impl! { @type(fn);
			$vis type $id = unsafe extern $abi fn($($args)*) $(-> $res)?;
		}

		$crate::externs::extern_fns! {
			$($tt)*
		}
	};
}
pub use extern_fns;
