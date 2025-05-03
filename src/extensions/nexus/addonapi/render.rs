use arcdps::dxgi_swap_chain;
use nexus::gui::RawGuiRender;
use windows::core::InterfaceRef;
use windows::Win32::Foundation::ERROR_API_UNAVAILABLE;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11DeviceContext};
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;
use crate::extensions::nexus::NexusHost;
use crate::util::win::{WinResult, WinError};

impl NexusHost {
	pub fn dxgi_swap_chain() -> WinResult<InterfaceRef<'static, IDXGISwapChain>> {
		dxgi_swap_chain()
			.ok_or_else(|| WinError::new(ERROR_API_UNAVAILABLE.to_hresult(), "D3D11 context not available"))
	}

	pub fn dxgi_device() -> WinResult<ID3D11Device> {
		Self::dxgi_swap_chain()
			.and_then(|sc| unsafe { sc.GetDevice() })
	}

	pub fn dxgi_device_context() -> WinResult<ID3D11DeviceContext> {
		Self::dxgi_device()
			.and_then(|dev| unsafe { dev.GetImmediateContext() })
	}

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
