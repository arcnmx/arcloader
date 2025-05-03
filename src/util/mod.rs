pub mod arc;
#[cfg(feature = "nexus")]
pub mod nexus;
pub mod ffi;
#[cfg(windows)]
pub mod win;

macro_rules! cstr {
	($($s:tt)*) => {
		unsafe {
			::std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($($s)*, "\0").as_bytes())
		}
	};
}

#[cfg(all(feature = "unwind", panic = "unwind"))]
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

macro_rules! extern_fns {
	() => {};
	(
		$vis:vis unsafe extern $abi:tt fn $id:ident($($args:tt)*) $(-> $res:ty)? {
			$($body:tt)*
		}
		$($tt:tt)*
	) => {
		extern_fns_impl! { @fn;
			$vis unsafe extern $abi fn $id($($args)*) $(-> $res)? {
				$($body)*
			}
		}

		extern_fns! {
			$($tt)*
		}
	};
	(
		$vis:vis type $id:ident = unsafe extern $abi:tt fn($($args:tt)*) $(-> $res:ty)?;

		$($tt:tt)*
	) => {
		extern_fns_impl! { @type(fn);
			$vis type $id = unsafe extern $abi fn($($args)*) $(-> $res)?;
		}

		extern_fns! {
			$($tt)*
		}
	};
}

#[cfg(not(feature = "log"))]
macro_rules! arc_log {
	($($tt:tt)*) => {};
}
#[cfg(feature = "log")]
macro_rules! arc_log {
	(!$ex:tt; $level:ident($($tt:tt)*)) => {
		log::$level $ex { $($tt)* }
	};
}
macro_rules! trace {
	($($tt:tt)*) => {arc_log!{!!;trace($($tt)*)}};
}
macro_rules! debug {
	($($tt:tt)*) => {arc_log!{!!;debug($($tt)*)}};
}
macro_rules! info {
	($($tt:tt)*) => {arc_log!{!!;info($($tt)*)}};
}
macro_rules! warn {
	($($tt:tt)*) => {arc_log!{!!;warn($($tt)*)}};
}
macro_rules! error {
	($($tt:tt)*) => {arc_log!{!!;error($($tt)*)}};
}
