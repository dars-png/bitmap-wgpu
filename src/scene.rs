use crate::gfx;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
}

const QUAD: [Vertex; 4] = [
    Vertex { pos: [-0.5, -0.5, 0.0], uv: [0.0, 0.0] },
    Vertex { pos: [ 0.5, -0.5, 0.0], uv: [0.0, 0.0] },
    Vertex { pos: [ 0.5,  0.5, 0.0], uv: [0.0, 0.0] },
    Vertex { pos: [-0.5,  0.5, 0.0], uv: [0.0, 0.0] },
];
const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct Scene {
    pipeline: wgpu::RenderPipeline,
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    ubuf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth_view: wgpu::TextureView,
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

impl Scene {
    pub fn new(gfx_instance: &gfx::GFX, rgba: &[u8], img_w: u32, img_h: u32) -> Self {
        let device = &gfx_instance.device;

        let size = wgpu::Extent3d { width: img_w, height: img_h, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tex"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        gfx_instance.queue.write_texture(
            texture.as_image_copy(), 
            rgba, 
            wgpu::TexelCopyBufferLayout { 
                offset: 0, 
                bytes_per_row: Some(4 * img_w), 
                rows_per_image: Some(img_h), 
            }, 
            size,
        );

        let tex_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Verts"),
            contents: bytemuck::cast_slice(&QUAD),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Indices"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let ubuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MVP"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bg1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), 
                    count: None,
                },
            ] 
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
        label: None, 
        layout: &bg1,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&tex_view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
        ],
     });

     let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
     let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
        label: None, 
        bind_group_layouts: &[Some(&bg1)], 
        immediate_size: 0,
    });

    let additive = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
        label: Some("Pipeline"), 
        layout: Some(&layout), 
        vertex: wgpu::VertexState { 
            module: &shader, 
            entry_point: Some("vs_main"), 
            compilation_options: Default::default(), 
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
            })], 
        }, 
        primitive: wgpu::PrimitiveState { 
            topology: wgpu::PrimitiveTopology::TriangleList, 
            cull_mode: None,
            ..Default::default()
        }, 
        depth_stencil: Some(wgpu::DepthStencilState { 
            format: DEPTH_FORMAT, 
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(), 
            bias: Default::default() 
        }), 
        multisample: Default::default(), 
        fragment: Some(wgpu::FragmentState { 
            module: &shader, 
            entry_point: Some("fs_main"), 
            compilation_options: Default::default(), 
            targets: &[Some(wgpu::ColorTargetState { 
                format: gfx_instance.config.format, 
                blend: Some(wgpu::BlendState { color: additive, alpha: additive }), 
                write_mask: wgpu::ColorWrites::ALL, 
            })], 
        }), 
        multiview_mask: None, 
        cache: None,
    });

    let depth_view = make_depth_view(device, gfx_instance.config.width, gfx_instance.config.height);
    Self { pipeline, vbuf, ibuf, ubuf, bind_group, depth_view }
    }

    pub fn resize(&mut self, gfx_instance: &gfx::GFX) {
        self.depth_view = make_depth_view(&gfx_instance.device, gfx_instance.config.width, gfx_instance.config.height);
    }

    pub fn render(&self, gfx_instance: &gfx::GFX, angle: f32, uv: [[f32; 2]; 4]) {
        // Cook up the MVP
        let aspect = gfx_instance.config.width as f32 / gfx_instance.config.height as f32;
        let proj = glam::camera::rh::proj::directx::perspective(45f32.to_radians(), aspect, 0.1, 100.0);
        let view = glam::camera::rh::view::look_at_mat4(
            glam::Vec3::new(0.0, 0.0, 2.0), 
            glam::Vec3::ZERO, 
            glam::Vec3::Y,
        );
        let model = glam::Mat4::from_rotation_y(angle);
        let mvp = proj * view * model;
        gfx_instance.queue.write_buffer(&self.ubuf, 0, bytemuck::cast_slice(&mvp.to_cols_array()));

        let mut verts = QUAD;
        for (v, t) in verts.iter_mut().zip(uv) { v.uv = t; }
        gfx_instance.queue.write_buffer(&self.vbuf, 0, bytemuck::cast_slice(&verts));

        use wgpu::CurrentSurfaceTexture as Cst;
        let frame = match gfx_instance.surface.get_current_texture() {
            Cst::Success(f) | Cst::Suboptimal(f) => f,
            Cst::Outdated => { gfx_instance.surface.configure(&gfx_instance.device, &gfx_instance.config); return; }
            Cst::Timeout | Cst::Occluded => return,
            Cst::Lost | Cst::Validation => return,
        };

        let color_view = frame.texture.create_view(&Default::default());
        let mut enc = gfx_instance.device.create_command_encoder(&Default::default());

        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: None, 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &color_view, 
                    depth_slice: None, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.2, g: 0.3, b: 0.3, a: 1.0 }), 
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

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vbuf.slice(..));
            pass.set_index_buffer(self.ibuf.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..6, 0, 0..1);
        }
        gfx_instance.queue.submit([enc.finish()]);
        gfx_instance.queue.present(frame);
    }
}