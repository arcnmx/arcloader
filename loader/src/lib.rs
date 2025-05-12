#[cfg(feature = "arcdps")]
pub use arcdps::imgui::{self, sys as imgui_sys};
#[cfg(all(feature = "addonapi", not(feature = "arcdps")))]
pub use nexus::imgui::{self, sys as imgui_sys};

#[macro_use]
pub mod util;
pub mod extensions;
pub mod host;
pub mod ui;
pub mod supervisor;
pub mod export;

pub struct RenderThread;
