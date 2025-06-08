macro_rules! extern_fns {
	($($s:tt)*) => {::arcffi::extern_fns!{$($s)*}};
}
macro_rules! cstr {
	($($s:tt)*) => {::arcffi::cstr!{$($s)*}};
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

#[cfg(any(feature = "arcdps", feature = "host-arcdps"))]
pub mod arc;
#[cfg(any(feature = "addonapi", feature = "host-addonapi"))]
pub mod nexus;
pub(crate) use arcffi as ffi;
#[cfg(windows)]
pub mod win;
