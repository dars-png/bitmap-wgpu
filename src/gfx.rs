use std::sync::Arc;
use winit::window::Window;

pub struct GFX {
    pub surface : wgpu::Surface<'static>,
    pub device  : wgpu::Device,
    pub queue   : wgpu::Queue,
    pub config  : wgpu::SurfaceConfiguration,
}

impl GFX {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await.expect("GPU adapter cringe");

        let (device, queue) = adapter
            .request_device(&wgpu::wgt::DeviceDescriptor::default())
            .await.expect("request_device failed");

        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        Self{ surface, device, queue, config }
    }
}
