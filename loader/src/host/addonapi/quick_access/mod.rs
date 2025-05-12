use crate::{
	host::addonapi::NexusHost,
	util::ffi::cstr_opt,
};
use nexus::gui::RawGuiRender;
use std::{ffi::c_char, ptr};

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_quick_access_add(identifier: *const c_char, texture_identifier: *const c_char, texture_hover_identifier: *const c_char, keybind_identifier: *const c_char, tooltip_text: *const c_char) {
		let id = cstr_opt(&identifier);
		let texture_id = cstr_opt(&texture_identifier);
		let texture_hover_id = cstr_opt(&texture_hover_identifier);
		let keybind_id = cstr_opt(&keybind_identifier);
		let tooltip = cstr_opt(&tooltip_text);

		addonapi_stub!(quick_access::add("{:?}, {:?}, {:?}, {:?}, {:?}", id, texture_id, texture_hover_id, keybind_id, tooltip))
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_remove(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(quick_access::remove("{:?}", id))
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_notify(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(quick_access::notify("{:?}", id))
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_add_context_menu(identifier: *const c_char, target_identifier: *const c_char, shortcut_render_callback: RawGuiRender)  {
		let id = cstr_opt(&identifier);
		let target_id = cstr_opt(&target_identifier);

		addonapi_stub!(quick_access::add_context_menu("{:?}, {:?}, {:?}", id, target_id, shortcut_render_callback))
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_add_context_menu_v2(identifier: *const c_char, shortcut_render_callback: RawGuiRender)  {
		Self::addonapi_quick_access_add_context_menu(identifier, ptr::null_mut(), shortcut_render_callback)
	}

	pub unsafe extern "C-unwind" fn addonapi_quick_access_remove_context_menu(identifier: *const c_char) {
		let id = cstr_opt(&identifier);

		addonapi_stub!(quick_access::remove_context_menu("{:?}", id))
	}
}
