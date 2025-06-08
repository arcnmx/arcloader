#[macro_export]
macro_rules! arc_log_trace {
	($($tt:tt)*) => {$crate::log::arc_log!{!!;trace($($tt)*)}};
}
#[macro_export]
macro_rules! arc_log_debug {
	($($tt:tt)*) => {$crate::log::arc_log!{!!;debug($($tt)*)}};
}
#[macro_export]
macro_rules! arc_log_info {
	($($tt:tt)*) => {$crate::log::arc_log!{!!;info($($tt)*)}};
}
#[macro_export]
macro_rules! arc_log_warn {
	($($tt:tt)*) => {$crate::log::arc_log!{!!;warn($($tt)*)}};
}
#[macro_export]
macro_rules! arc_log_error {
	($($tt:tt)*) => {$crate::log::arc_log!{!!;error($($tt)*)}};
}
pub use {arc_log_trace as trace, arc_log_debug as debug, arc_log_info as info, arc_log_warn as warn, arc_log_error as error};

#[cfg(not(feature = "log"))]
#[macro_export]
macro_rules! arc_log {
	($($tt:tt)*) => {{}};
}
#[cfg(feature = "log")]
#[macro_export]
macro_rules! arc_log {
	(!$ex:tt; $level:ident($($tt:tt)*)) => {
		$crate::log::log::$level $ex { $($tt)* }
	};
}
#[doc(hidden)]
pub use arc_log;
#[cfg(feature = "log")]
#[doc(hidden)]
pub use ::log;
