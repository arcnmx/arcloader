use crate::{
	extensions::nexus::NexusHost,
	util::{arc::{config_dir, game_dir}, ffi::cstr_opt},
};
use std::{borrow::Cow, ffi::{c_char, CStr, CString}, path::{Path, PathBuf}, sync::{Arc, OnceLock}};

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_path_get_game_dir() -> *const c_char {
		static GAME_DIR: OnceLock<CString> = OnceLock::new();
		const FALLBACK: &'static CStr = unsafe {
			CStr::from_bytes_with_nul_unchecked(b"./\0")
		};

		let dir = GAME_DIR.get_or_init(||
			cstring_try_dir(game_dir(), FALLBACK)
		);
		addonapi_stub!(path::get_game_dir());
		dir.as_ptr()
	}

	pub unsafe extern "C-unwind" fn addonapi_path_get_addon_dir(name_c: *const c_char) -> *const c_char {
		static ADDONS_DIR: OnceLock<PathBuf> = OnceLock::new();
		const FALLBACK: &'static CStr = unsafe {
			CStr::from_bytes_with_nul_unchecked(b"addons/\0")
		};

		let addons_dir = ADDONS_DIR.get_or_init(|| {
			let addons_dir = match config_dir() {
				Some(mut config_dir) => {
					config_dir.pop();
					config_dir
				},
				None => PathBuf::from("."),
			};
			addons_dir
		});

		let name = cstr_opt(&name_c);
		addonapi_stub!(path::get_addon_dir("{:?}", name));

		let dir = match name {
			Some(name) => Cow::Owned(addons_dir
				.join(&name.to_string_lossy()[..])
			),
			None => Cow::Borrowed(addons_dir.as_path()),
		};
		let dir = cstring_try_dir(Some(&dir), FALLBACK);

		log::warn!("produced {:?}", dir);

		let dir_ptr = Self::cache_write_with(name_c as *const _, move |mut cache| {
			*cache.cstrings.entry(Arc::new(dir))
				.or_insert_with_key(|k| k.as_ptr())
		});
		dir_ptr
	}

	pub unsafe extern "C-unwind" fn addonapi_path_get_common_dir() -> *const c_char {
		let common = CStr::from_bytes_with_nul_unchecked(b"common/\0");
		Self::addonapi_path_get_addon_dir(common.as_ptr())
	}
}

fn cstring_dir(dir: CString) -> CString {
	#[cfg(todo)]
	match dir.as_bytes().last() {
		Some(&c) if is_separator(c as char) => dir,
		_ => {
			let mut s = dir.into_bytes();
			s.push(MAIN_SEPARATOR as u8);
			unsafe {
				CString::from_vec_unchecked(s)
			}
		},
	}
	dir
}

fn cstring_try_dir(dir: Option<&Path>, fallback: &'static CStr) -> CString {
	let dir = dir
		.and_then(|d| CString::new(d.to_string_lossy().into_owned()).ok())
		.unwrap_or_else(|| {
			warn!("addonapi dir not available");
			fallback.to_owned()
		});
	cstring_dir(dir)
}

