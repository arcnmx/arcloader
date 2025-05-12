use core::str;
use std::{ffi::{CStr, OsString}, fmt, hash::Hash, mem::transmute, num::{NonZeroU16, NonZeroU32, NonZeroU8}, ops::{Deref, DerefMut}, ptr, str::FromStr};
use nexus::{addon::{self, AddonDefinition}, keybind::Keybind as NexusKeybind, AddonFlags};
use windows::Win32::{Foundation::{ERROR_BAD_ARGUMENTS, ERROR_CALL_NOT_IMPLEMENTED, ERROR_INTERNAL_ERROR, ERROR_KEY_DOES_NOT_EXIST, LPARAM}, UI::Input::KeyboardAndMouse::{self as vk, VIRTUAL_KEY}};
#[cfg(windows)]
use windows::Win32::{Foundation::{ERROR_NOT_SUPPORTED, HMODULE}, System::LibraryLoader::GetProcAddress};
#[cfg(windows)]
use crate::util::win::{WinResult, WinError};

use super::{ffi::cstr_opt, win::{get_key_name, get_scan_code}};

pub type NexusId = i32;
pub type AddonApiVersion = i32;

pub type GetAddonDef = unsafe extern "system-unwind" fn() -> *const AddonDefinition;

#[cfg(windows)]
pub fn get_addon_def_addr(module: HMODULE) -> Option<GetAddonDef> {
	unsafe {
		GetProcAddress(module, windows_strings::s!("GetAddonDef"))
			.map(|f| transmute(f))
	}
}

#[cfg(windows)]
pub fn get_addon_def(module: &HMODULE) -> WinResult<(&AddonDesc, AddonApiVersion)> {
	let def_fn = get_addon_def_addr(*module)
		.ok_or_else(|| WinError::new(ERROR_NOT_SUPPORTED.to_hresult(), "Nexus GetAddonDef missing"))?;
	let def = unsafe {
		def_fn()
	};

	let api_version = unsafe { *ptr::addr_of!((*def).api_version) };

	Ok((unsafe { AddonDesc::with_ptr(def) }, api_version))
}

#[derive(Copy, Clone)]
pub struct AddonDesc {
	def: AddonDefinition,
}

impl AddonDesc {
	pub const unsafe fn new(def: AddonDefinition) -> Self {
		Self {
			def,
		}
	}

	pub unsafe fn with_ptr(def: *const AddonDefinition) -> &'static Self {
		transmute(def)
	}

	pub unsafe fn with_ref(def: &AddonDefinition) -> &Self {
		transmute(def)
	}

	pub unsafe fn with_mut(def: &mut AddonDefinition) -> &mut Self {
		transmute(def)
	}

	pub fn ptr_from(def: *const AddonDefinition) -> *const Self {
		def as *const Self
	}

	pub fn ptr_from_mut(def: *mut AddonDefinition) -> *mut Self {
		def as *mut Self
	}

	#[inline]
	pub fn def(&self) -> &AddonDefinition {
		&self.def
	}

	pub unsafe fn def_mut(&mut self) -> &mut AddonDefinition {
		&mut self.def
	}

	pub fn is_raidcore(&self) -> bool {
		self.def().signature >= 0
	}

	pub fn can_hotload(&self) -> bool {
		!self.def().flags.contains(AddonFlags::DisableHotloading)
	}

	pub fn name(&self) -> &CStr {
		unsafe {
			cstr_opt(&self.def().name)
		}.unwrap_or_default()
	}

	pub fn author(&self) -> Option<&CStr> {
		unsafe { cstr_opt(&self.def().author) }
	}

	pub fn description(&self) -> Option<&CStr> {
		unsafe { cstr_opt(&self.def().description) }
	}

	pub fn update_link(&self) -> Option<&CStr> {
		unsafe { cstr_opt(&self.def().update_link) }
	}

	pub fn version(&self) -> &AddonVersion {
		AddonVersion::from_ref(&self.def().version)
	}
}

impl Deref for AddonDesc {
	type Target = AddonDefinition;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.def()
	}
}

impl fmt::Display for AddonDesc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = self.name().to_string_lossy();
		write!(f, "{name} {}", self.version())?;
		
		#[cfg(todo)]
		if let Some(author) = self.author() {
			let author = self.name().to_string_lossy();
			write!(f, " by {author}")?;
		}

		Ok(())
	}
}

impl fmt::Debug for AddonDesc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut f = f.debug_struct("AddonDesc");
		f
			.field("desc", &format_args!("{}", self))
			.field("def", &self.def)
			.finish()
	}
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AddonVersion {
	pub version: addon::AddonVersion,
}

impl AddonVersion {
	pub const fn new(major: i16, minor: i16, revision: i16, build: i16) -> Self {
		Self::with_version(addon::AddonVersion {
			major,
			minor,
			revision,
			build,
		})
	}

	pub const fn with_version(version: addon::AddonVersion) -> Self {
		Self {
			version,
		}
	}

	pub const fn from_ref(version: &addon::AddonVersion) -> &Self {
		unsafe {
			transmute(version)
		}
	}

	pub fn from_mut(version: &mut addon::AddonVersion) -> &mut Self {
		unsafe {
			transmute(version)
		}
	}

	pub fn revision(&self) -> Option<i16> {
		match self.revision {
			-1 => None,
			rev => Some(rev),
		}
	}
}

impl Deref for AddonVersion {
	type Target = addon::AddonVersion;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.version
	}
}

impl DerefMut for AddonVersion {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.version
	}
}

impl fmt::Display for AddonVersion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}.{}.{}", self.major, self.minor, self.build)?;
		if let Some(rev) = self.revision() {
			write!(f, ".{rev}")?;
		}

		Ok(())
	}
}

impl fmt::Debug for AddonVersion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self.version, f)
	}
}

impl From<addon::AddonVersion> for AddonVersion {
	fn from(v: addon::AddonVersion) -> Self {
		Self::with_version(v)
	}
}

impl From<AddonVersion> for addon::AddonVersion {
	fn from(v: AddonVersion) -> Self {
		v.version
	}
}

pub type KeybindMods = [u8; 3];

#[derive(Debug, Copy, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Keybind {
	pub code: u16,
	pub alt: u8,
	pub ctrl: u8,
	pub shift: u8,
}

impl Keybind {
	pub const LMB: Self = Self::new_button(1);
	pub const RMB: Self = Self::new_button(2);
	pub const MMB: Self = Self::new_button(3);
	pub const M4: Self = Self::new_button(4);
	pub const M5: Self = Self::new_button(5);
	pub const MODS_MOUSE: KeybindMods = [2, 1, 1];

	pub const LPARAM_SCANCODE_SHIFT: usize = 16;
	pub const LPARAM_SCANCODE_MASK: u32 = 0x007f_0000;
	/// Indicates extended modifier key
	pub const LPARAM_MODIFIER: u32 = 0x0080_0000;
	/// Virtual code indicates lack of information
	/// (such as whether a modifier was the left or right key)
	pub const LPARAM_DO_NOT_CARE: u32 = 0x0100_0000;

	pub const fn new_key(code: u16, alt: bool, ctrl: bool, shift: bool) -> Self {
		Self {
			code,
			alt: alt as u8,
			ctrl: ctrl as u8,
			shift: shift as u8,
		}
	}

	pub const fn new_button(button: u8) -> Self {
		let [alt, ctrl, shift] = Self::MODS_MOUSE;
		Self {
			code: button as u16,
			alt,
			ctrl,
			shift,
		}
	}

	pub fn with_mods(mods: KeybindMods, code: u16) -> Self {
		let [alt, ctrl, shift] = mods;

		Self {
			code,
			alt,
			ctrl,
			shift,
		}
	}

	pub fn mods(&self) -> &KeybindMods {
		let p = ptr::addr_of!(self.alt) as *const KeybindMods;
		unsafe {
			transmute(p)
		}
	}

	pub fn code(&self) -> Result<Option<InputCode>, ()> {
		match *self.mods() {
			Self::MODS_MOUSE => NonZeroU8::new(self.code as u8)
				.ok_or(())
				.map(|button| Some(InputCode::Mouse {
				button,
			})),
			[0, 0, 0] if self.code == 0 => Ok(None),
			[0|1, 0|1, 0|1] => NonZeroU16::new(self.code)
				.ok_or(())
				.map(|scan_code| Some(InputCode::Keyboard {
					scan_code,
			})),
			_ => Err(()),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self { code: 0, alt: 0, ctrl: 0, shift: 0 } => true,
			_ => false,
		}
	}

	pub fn is_keyboard(&self) -> bool {
		match self {
			Self { alt: 0 | 1, ctrl: 0 | 1, shift: 0 | 1, .. } => true,
			_ => false,
		}
	}

	pub fn alt(&self) -> bool {
		self.alt != 0
	}

	pub fn ctrl(&self) -> bool {
		self.ctrl != 0
	}

	pub fn shift(&self) -> bool {
		self.shift != 0
	}

	pub fn mouse_button(&self) -> Option<NonZeroU8> {
		match *self.mods() {
			Self::MODS_MOUSE => NonZeroU8::new(self.code as u8),
			_ => None,
		}
	}

	pub fn key_scan(&self) -> Option<NonZeroU16> {
		match self.is_keyboard() {
			true => NonZeroU16::new(self.code),
			_ => None,
		}
	}

	pub fn key_param(&self) -> Option<LPARAM> {
		self.key_scan()
			.map(|scan| {
				let param = (scan.get() as u32) << Self::LPARAM_SCANCODE_SHIFT;
				if param >= Self::LPARAM_DO_NOT_CARE {
					info!("keycode {scan} is virtual?");
				}
				LPARAM(param as isize)
			})
	}

	pub fn key_name(&self) -> WinResult<OsString> {
		self.key_param()
			.ok_or_else(|| WinError::new(ERROR_BAD_ARGUMENTS.to_hresult(), "not a key"))
			.and_then(get_key_name)
	}

	pub fn extra24(&self) -> Option<NonZeroU32> {
		const MASK7: u8 = 0x7e;

		let alt7 = self.alt & MASK7;
		let ctrl7 = self.ctrl & MASK7;
		let shift7 = self.shift & MASK7;

		if let (0, 0, 0) = (alt7, ctrl7, shift7) {
			// at least one upper bit must be set,
			// meaning that values 0 to 7 (0b111) are reserved
			return None
		}

		let value =
			(alt7 as u32) << (3-1)
			| (ctrl7 as u32) << (3+7-1)
			| (shift7 as u32) << (3+7*2-1)
			| (self.alt & 1) as u32
			| ((self.ctrl & 1) << 1) as u32
			| ((self.shift & 1) << 2) as u32;

		Some(unsafe {
			// early check filters out the all-zero case
			NonZeroU32::new_unchecked(value)
		})
	}

	pub fn interpret_ascii(s: &[u8]) -> WinResult<(KeybindMods, &[u8])> {
		let mut bind = Keybind::default();
		let mut split = s.splitn(4, |&b| b == b'+');
		let key = loop {
			let seg = match split.next() {
				Some(seg) if split.size_hint().0 == 0 => break seg,
				Some(seg) => seg,
				None => return Err(ERROR_INTERNAL_ERROR.into()),
			};
			let nonsensical;
			match seg {
				seg if seg.eq_ignore_ascii_case(b"alt") => {
					nonsensical = bind.alt;
					bind.alt = 1;
				},
				seg if seg.eq_ignore_ascii_case(b"ctrl") => {
					nonsensical = bind.ctrl;
					bind.ctrl = 1;
				},
				seg if seg.eq_ignore_ascii_case(b"shift") => {
					nonsensical = bind.shift;
					bind.shift = 1;
				}
				seg => return Err(WinError::new(
					ERROR_KEY_DOES_NOT_EXIST.to_hresult(),
					format!("unknown modifier {:?}", str::from_utf8(seg)),
				)),
			}
			if nonsensical != 0 {
				warn!("nonsensical keybind {:?}", str::from_utf8(s));
			}
		};
		debug_assert!(split.next().is_none());

		Ok((*bind.mods(), key))
	}

	pub fn parse_ascii(s: &[u8]) -> WinResult<Keybind> {
		let (mods, key) = Self::interpret_ascii(s)?;
		let mut bind = Keybind::with_mods(mods, 0);

		let mouse = match key {
			k if k.eq_ignore_ascii_case(b"LMB") =>
				Some(Keybind::LMB),
			k if k.eq_ignore_ascii_case(b"RMB") =>
				Some(Keybind::RMB),
			k if k.eq_ignore_ascii_case(b"MMB") =>
				Some(Keybind::MMB),
			k if k.eq_ignore_ascii_case(b"M4") =>
				Some(Keybind::M4),
			k if k.eq_ignore_ascii_case(b"M5") =>
				Some(Keybind::M5),
			_ => None,
		};
		match (mouse, bind) {
			(Some(mouse), Self { alt: 0, ctrl: 0, shift: 0, .. }) =>
				return Ok(mouse),
			(Some(mouse), ..) => {
				warn!("nonsensical mouse binding");
				return Ok(mouse)
			},
			_ => (),
		}

		let vk = match key {
			&[c] if c.is_ascii_alphanumeric() => Some(VIRTUAL_KEY(c as u16)),
			&[b'N' | b'n', _, _, _, _, _, c @ b'1'..=b'9'] if key[..6].eq_ignore_ascii_case(b"numpad") =>
				Some(VIRTUAL_KEY(vk::VK_NUMPAD0.0 + (c - b'0') as u16)),
			&[b'F', c @ b'1'..=b'9'] => Some(VIRTUAL_KEY(vk::VK_F1.0 + (c - b'1') as u16)),
			&[b'F', b'1', c @ b'0'..=b'9'] => Some(VIRTUAL_KEY(vk::VK_F10.0 + (c - b'0') as u16)),
			&[b'F', b'2', c @ b'0'..=b'4'] => Some(VIRTUAL_KEY(vk::VK_F20.0 + (c - b'0') as u16)),
			k if k.eq_ignore_ascii_case(b"return") || k.eq_ignore_ascii_case(b"enter") => Some(vk::VK_RETURN),
			k if k.eq_ignore_ascii_case(b"escape") || k.eq_ignore_ascii_case(b"esc") => Some(vk::VK_ESCAPE),
			k if k.eq_ignore_ascii_case(b"up") => Some(vk::VK_UP),
			k if k.eq_ignore_ascii_case(b"down") => Some(vk::VK_DOWN),
			k if k.eq_ignore_ascii_case(b"left") => Some(vk::VK_LEFT),
			k if k.eq_ignore_ascii_case(b"right") => Some(vk::VK_RIGHT),
			k if k.eq_ignore_ascii_case(b"tab") => Some(vk::VK_TAB),
			k if k.eq_ignore_ascii_case(b"alt") => Some(vk::VK_MENU),
			k if k.eq_ignore_ascii_case(b"ctrl") || k.eq_ignore_ascii_case(b"control") => Some(vk::VK_CONTROL),
			k if k.eq_ignore_ascii_case(b"shift") => Some(vk::VK_SHIFT),
			k if k.eq_ignore_ascii_case(b"lwin") || k.eq_ignore_ascii_case(b"win") || k.eq_ignore_ascii_case(b"windows") || k.eq_ignore_ascii_case(b"super") => Some(vk::VK_LWIN),
			k if k.eq_ignore_ascii_case(b"rwin") => Some(vk::VK_RWIN),
			_ => None,
		};
		if let Some(scancode) = vk.and_then(get_scan_code) {
			bind.code = scancode.get();
			return Ok(bind)
		}

		#[cfg(todo)]
		if let Some(key) = lookup_cache(s) {
			bind.code = key;
			return Ok(bind)
		}

		Err(WinError::new(ERROR_CALL_NOT_IMPLEMENTED.to_hresult(), format!("virtual keycode lookup for {:?}", String::from_utf8_lossy(key))))
	}

	pub fn to_nexus(self) -> NexusKeybind {
		NexusKeybind {
			key: self.code,
			alt: self.alt != 0,
			ctrl: self.ctrl != 0,
			shift: self.shift != 0,
		}
	}

	pub fn as_nexus_ptr(&self) -> *const NexusKeybind {
		self as *const Self as *const NexusKeybind
	}
}

impl From<NexusKeybind> for Keybind {
	fn from(bind: NexusKeybind) -> Self {
		Self::new_key(bind.key, bind.alt, bind.ctrl, bind.shift)
	}
}

impl From<Keybind> for NexusKeybind {
	fn from(bind: Keybind) -> Self {
		bind.to_nexus()
	}
}

impl TryFrom<&'_ CStr> for Keybind {
	type Error = WinError;

	fn try_from(s: &CStr) -> Result<Self, Self::Error> {
		Self::parse_ascii(s.as_ref().to_bytes())
	}
}

impl FromStr for Keybind {
	type Err = WinError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse_ascii(s.as_bytes())
	}
}

impl fmt::Display for Keybind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut prefix = "";
		if self.ctrl() {
			write!(f, "{prefix}CTRL")?;
			prefix = "+";
		}
		if self.alt() {
			write!(f, "{prefix}ALT")?;
			prefix = "+";
		}
		if self.alt() {
			write!(f, "{prefix}SHIFT")?;
			prefix = "+";
		}
		if let Ok(name) = self.key_name() {
			write!(f, "{prefix}{}", name.to_string_lossy())
		} else if let Ok(Some(input)) = self.code() {
			write!(f, "{prefix}{input}")
		} else {
			write!(f, "{prefix}{}", self.code)
		}
	}
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InputCode {
	Keyboard {
		scan_code: NonZeroU16,
	} = 1,
	Mouse {
		button: NonZeroU8,
	},
}

impl InputCode {
	pub fn to_keybind(self) -> Keybind {
		match self {
			Self::Keyboard { scan_code } =>
				Keybind::new_key(scan_code.get(), false, false, false),
			Self::Mouse { button } =>
				Keybind::new_button(button.get()),
		}
	}

	pub fn to_virtual(self) -> Option<VIRTUAL_KEY> {
		let vk = match self {
			Self::Keyboard {
				scan_code,
			} => {
				todo!()
			},
			Self::Mouse { button } => {
				let vk = button.get();
				if vk as u16 > vk::VK_XBUTTON2.0 {
					// TODO?
					return None
				}
				vk.into()
			},
		};
		Some(VIRTUAL_KEY(vk))
	}
}

impl From<InputCode> for Keybind {
	fn from(code: InputCode) -> Self {
		code.to_keybind()
	}
}

impl fmt::Display for InputCode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Keyboard { scan_code } => {
				debug!("TODO: InputCode as Display");
				write!(f, "{scan_code}")
			},
			Self::Mouse { button } => match button.get() {
				0 => write!(f, "LMB"),
				1 => write!(f, "RMB"),
				2 => write!(f, "MMB"),
				_ => write!(f, "M{button}"),
			},
		}
	}
}

// TODO: From<imgui::MouseButton>
