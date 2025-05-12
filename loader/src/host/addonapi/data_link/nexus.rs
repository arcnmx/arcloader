use nexus::{data_link::NexusLink, imgui::{Ui, FontId}};

use crate::util::ffi::nonnull_bytes;
use std::{cell::UnsafeCell, mem::{transmute, MaybeUninit}, ptr::{self, NonNull}, sync::LazyLock};

use super::DataLinkShare;

pub static NEXUS_LINK: LazyLock<NexusLinkProvider> = LazyLock::new(NexusLinkProvider::new);

pub struct NexusLinkProvider {
	data: UnsafeCell<NexusLink>,
}

impl NexusLinkProvider {
	pub const DATA_EMPTY: NexusLink = unsafe {
		MaybeUninit::zeroed().assume_init()
	};

	pub fn new() -> Self {
		Self {
			data: UnsafeCell::new(Self::DATA_EMPTY),
		}
	}

	pub fn init(ui: &Ui) {
		#[cfg(todo)] {
			let dl = vec![0u8; size_of::<NexusLink>()].into_boxed_slice();
			let dl = NEXUS_LINK;
			NexusHost::register_data_link(Self::DATA_LINK_NEXUS, dl);
		}
		NEXUS_LINK.update_fonts(ui);
	}

	pub fn as_ptr(&self) -> NonNull<NexusLink> {
		unsafe {
			NonNull::new_unchecked(self.data.get())
		}
	}

	pub fn get_ptr() -> Option<NonNull<NexusLink>> {
		Some(NEXUS_LINK.as_ptr())
	}

	fn update_fonts(&self, ui: &Ui) {
		if let Some(&font) = ui.fonts().fonts().first() {
			let nl = self.data.get();
			unsafe {
				let font: FontId = font;
				let font_ptr = transmute(font);
				ptr::write_volatile(ptr::addr_of_mut!((*nl).font), font_ptr);
				ptr::write_volatile(ptr::addr_of_mut!((*nl).font_ui), font_ptr);
				ptr::write_volatile(ptr::addr_of_mut!((*nl).font_big), font_ptr);
			}
		}
	}

	pub fn imgui_present(ui: &Ui, not_charsel_or_loading: bool) {
		NEXUS_LINK.update_ui(ui, not_charsel_or_loading)
	}

	fn update_ui(&self, ui: &Ui, not_charsel_or_loading: bool) {
		let [w, h] = ui.io().display_size;
		#[cfg(todo)]
		let [x, y] = ui.io().display_framebuffer_scale;
		//let scaling =  x / y;
		let scaling = 1.0;
		let nl = self.data.get();
		unsafe {
			ptr::write_volatile(ptr::addr_of_mut!((*nl).is_gameplay), not_charsel_or_loading);
			ptr::write_volatile(ptr::addr_of_mut!((*nl).width), w as u32);
			ptr::write_volatile(ptr::addr_of_mut!((*nl).height), h as u32);
			ptr::write_volatile(ptr::addr_of_mut!((*nl).scaling), scaling);
			//self.data.is_moving = ?;
			//self.data.is_camera_moving = ?;
			if ptr::read(ptr::addr_of!((*nl).font)).is_null() {
				self.update_fonts(ui);
			}
		}
	}
}

unsafe impl Send for NexusLinkProvider {}
unsafe impl Sync for NexusLinkProvider {}

unsafe impl DataLinkShare for NexusLinkProvider {
	unsafe fn get_data_share_pinned(&mut self) -> NonNull<[u8]> {
		let p = self.as_ptr();
		nonnull_bytes(p)
	}
}
