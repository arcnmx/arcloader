pub mod arc;
#[cfg(feature = "nexus")]
pub mod nexus;
pub mod ffi;
#[cfg(windows)]
pub mod win;

#[macro_export]
macro_rules! cstr {
	($($s:tt)*) => {
		unsafe {
			::std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($($s)*, "\0").as_bytes())
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
