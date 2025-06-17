#[cfg(feature = "library")]
pub mod LibraryLoader {
	#[cfg(all(
		any(feature = "windows-core-060", feature = "windows-core-061"),
		not(any(feature = "windows-060", feature = "windows-061")),
	))]
	pub use crate::windows::core_0xx::imp::{GetProcAddress, LoadLibraryExA, FreeLibrary};
	#[cfg(any(feature = "windows-060", feature = "windows-061"))]
	pub use crate::windows::Win32_0xx::System::LibraryLoader::{GetProcAddress, LoadLibraryA, LoadLibraryExA, LoadLibraryW, LoadLibraryExW};
}
pub mod Diagnostics {
	pub mod Debug {
		#[cfg(all(
			any(feature = "windows-core-060", feature = "windows-core-061"),
			not(any(feature = "windows-060", feature = "windows-061")),
		))]
		pub use crate::windows::core_0xx::imp::EncodePointer;
		#[cfg(any(feature = "windows-060", feature = "windows-061"))]
		pub use crate::windows::Win32_0xx::System::Diagnostics::Debug::{EncodePointer, DecodePointer};
	}
}
