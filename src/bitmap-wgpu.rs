use wgpu::util::DeviceExt;

use bitmap::BitmapFont;
use gfx::{DEPTH_FORMAT, GFX};

const MAX_CHARS: usize = 256;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    mvp: [f32; 16],
    color: [f32; 4],
}

pub struct BitmapRenderer {
    pipeline: wgpu::RenderPipeline,
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    ubuf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    index_count: u32,
}

impl BitmapRenderer {
    pub fn new(gfx_instance: &gfx::GFX, font: &BitmapFont) -> Self {
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

}