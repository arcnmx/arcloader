macro_rules! extern_fns {
	($($s:tt)*) => {::arcffi::extern_fns!{$($s)*}};
}
macro_rules! cstr {
	($($s:tt)*) => {::arcffi::cstr!{$($s)*}};
}
macro_rules! trace {
	($($tt:tt)*) => {::dyload::log::trace!{$($tt)*}};
}
macro_rules! debug {
	($($tt:tt)*) => {::dyload::log::debug!{$($tt)*}};
}
macro_rules! info {
	($($tt:tt)*) => {::dyload::log::info!{$($tt)*}};
}
macro_rules! warn {
	($($tt:tt)*) => {::dyload::log::warn!{$($tt)*}};
}
macro_rules! error {
	($($tt:tt)*) => {::dyload::log::error!{$($tt)*}};
}

#[cfg(any(feature = "arcdps", feature = "host-arcdps"))]
pub mod arc;
#[cfg(any(feature = "addonapi", feature = "host-addonapi"))]
pub mod nexus;
pub(crate) use arcffi as ffi;
pub(crate) mod win {
	pub use ::dyload::windows::*;
}
