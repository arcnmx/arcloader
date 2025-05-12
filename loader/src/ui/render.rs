use std::{cell::RefCell, mem::ManuallyDrop};
use crate::export::imgui_ui;
use std::sync::Once;
use crate::imgui::Ui;
use crate::RenderThread;

thread_local! {
	static IMGUI_UI: RefCell<Option<ManuallyDrop<Ui<'static>>>> = RefCell::new(None);
}

impl RenderThread {
	pub fn render_start() {
		let ui = imgui_ui();

		static WARN_ONCE: Once = Once::new();
		if let None = &ui {
			WARN_ONCE.call_once(|| {
				warn!("imgui context unavailable");
			});
		}

		IMGUI_UI.set(ui);
	}

	pub fn render_end() {
		IMGUI_UI.set(None);
	}

	#[cfg(feature = "arcdps")]
	pub fn with_ui<R, F: FnOnce(&Ui) -> R>(f: F) -> Option<R> {
		IMGUI_UI.with_borrow(|ui| match ui.as_ref() {
			None => {
				static WARN_ONCE: Once = Once::new();
				if let None = &ui {
					WARN_ONCE.call_once(|| {
						warn!("unexpected imgui access");
					});
				}

				None
			},
			Some(ui) => Some(f(ui)),
		})
	}
}
