#[cfg(feature = "arcdps")]
pub use ::arcdps as imp;

#[cfg(feature = "evtc-016")]
pub use ::evtc as imp_evtc;
#[cfg(all(feature = "arcdps-015", not(feature = "evtc-016")))]
pub use ::arcdps_015::evtc as imp_evtc;

#[cfg(feature = "evtc-016")]
pub use ::evtc::agent::realtime as imp_realtime;
#[cfg(all(feature = "arcdps-015", not(feature = "evtc-016")))]
pub use ::arcdps_015::evtc as imp_realtime;
