use wgpu::util::DeviceExt;

use crate::bitmap::BitmapFont;
use crate::gfx::{DEPTH_FORMAT, GFX};

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
    pub fn new(gfx_instance: &GFX, font: &BitmapFont) -> Self {
        let device = &gfx_instance.device;
        let img = font.image();
        let (w, h) = img.dimensions();

        let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
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
            img.as_raw(), 
            wgpu::TexelCopyBufferLayout { 
                offset: 0, 
                bytes_per_row: Some(4 * w), 
                rows_per_image: Some(h), 
            }, 
            size,
        );

        let tex_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let vbuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Char Verts"),
            size: (MAX_CHARS * 4 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices: Vec<u16> = (0..MAX_CHARS as u16)
            .flat_map(|g| {
                let b = g * 4;
                [b, b + 1, b + 2, b, b + 2, b + 3]
            })
            .collect();

        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Char Indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let ubuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bitmap Font Uniform"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bg1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
        label: Some("Bitmat Font Pipeline"), 
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
            depth_write_enabled: Some(false),
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

    Self { pipeline, vbuf, ibuf, ubuf, bind_group, index_count: 0 }
    }

    pub fn set_text(&mut self, gfx_instance: &GFX, font: &BitmapFont, text: &str, gap: f32) {
        let aspect = font.char_aspect();
        let advance = aspect + gap;
        let chars: Vec<char> = text.chars().take(MAX_CHARS).collect();

        let total = chars.len() as f32 * advance - gap;
        let x_start = -total * 0.5;

        let mut verts = Vec::with_capacity(chars.len() * 4);
        for (i, &c) in chars.iter().enumerate() {
            let uv = font.get_char_uv(c);
            let x0 = x_start + i as f32 * advance;
            let x1 = x0 + aspect;

            verts.push(Vertex { pos: [x0, -0.5, 0.0], uv: [uv.u0, uv.v1] });
            verts.push(Vertex { pos: [x1, -0.5, 0.0], uv: [uv.u1, uv.v1] });
            verts.push(Vertex { pos: [x1,  0.5, 0.0], uv: [uv.u1, uv.v0] });
            verts.push(Vertex { pos: [x0,  0.5, 0.0], uv: [uv.u0, uv.v0] });
        }
        if !verts.is_empty() {
            gfx_instance.queue.write_buffer(&self.vbuf, 0, bytemuck::cast_slice(&verts));
        }
        self.index_count = (chars.len() * 6) as u32;
    }

    pub fn set_transform(&self, gfx_instance: &GFX, mvp: glam::Mat4, color: [f32; 4]) {
        let u = Uniforms { mvp: mvp.to_cols_array(), color };
        gfx_instance.queue.write_buffer(&self.ubuf, 0, bytemuck::bytes_of(&u));
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.index_count == 0 { return; }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vbuf.slice(..));
        pass.set_index_buffer(self.ibuf.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}