use nexus::gui::RawGuiRender;
use crate::extensions::nexus::NexusHost;

impl NexusHost {
	pub unsafe extern "C-unwind" fn addonapi_renderer_register(render_type: nexus::gui::RenderType, render_callback: RawGuiRender) {
		addonapi_stub!(renderer::add("{:?}, {:?}", render_type, render_callback));

		Self::cache_write_with(render_callback as *const _, move |mut cache| {
			let handlers = cache.renderers.entry(render_type)
				.or_default();
			handlers.insert(render_callback);
		});
	}

	pub unsafe extern "C-unwind" fn addonapi_renderer_deregister(render_callback: RawGuiRender) {
		addonapi_stub!(renderer::remove("{:?}", render_callback));

		let mut removed = false;
		Self::cache_write_with(render_callback as *const _, |mut cache| {
			for (_, r) in cache.renderers.iter_mut() {
				if r.remove(&render_callback) {
					removed = true;
				}
			}
		});
		if !removed {
			warn!("renderer not found");
		}
	}
}
