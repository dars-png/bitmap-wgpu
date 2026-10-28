use std::sync::Arc;
use winit::window::Window;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub struct GFX {
    pub surface : wgpu::Surface<'static>,
    pub device  : wgpu::Device,
    pub queue   : wgpu::Queue,
    pub config  : wgpu::SurfaceConfiguration,
    depth_view: wgpu::TextureView,
    pub clear_color : wgpu::Color,
}

fn make_depth_view(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::wgt::TextureViewDescriptor::default())
}


impl GFX {
    pub async fn new(window: Arc<Window>, r: f64, g: f64, b: f64, a: f64) -> Self {
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

        let depth_view = make_depth_view(&device, config.width, config.height);
        let clear_color = wgpu::Color { r, g, b, a };

        Self{ surface, device, queue, config, depth_view, clear_color }
    }

    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if (w == 0) || (h == 0) { return; }

        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = make_depth_view(&self.device, w, h);
    }


    pub fn render(&self, draw: impl FnOnce(&mut wgpu::RenderPass<'_>)) {
        use wgpu::CurrentSurfaceTexture as Cst;
        let frame = match self.surface.get_current_texture() {
            Cst::Success(f) | Cst::Suboptimal(f) => f,
            Cst::Outdated => { self.surface.configure(&self.device, &self.config); return; }
            Cst::Timeout | Cst::Occluded | Cst::Lost | Cst::Validation => return,
        };

        let color_view = frame.texture.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: None, 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &color_view, 
                    depth_slice: None, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(self.clear_color), 
                        store: wgpu::StoreOp::Store, 
                    }, 
                })], 
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                    view: &self.depth_view, 
                    depth_ops: Some(wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(1.0), 
                        store: wgpu::StoreOp::Store, 
                    }), 
                    stencil_ops: None, 
                }), 
                timestamp_writes: None, 
                occlusion_query_set: None, 
                multiview_mask: None, 
            });
            draw(&mut pass);
        }
        self.queue.submit([enc.finish()]);
        self.queue.present(frame);
    }
}
