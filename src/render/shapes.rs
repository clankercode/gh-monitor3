use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

const SHAPE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};
@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

pub struct ShapeRenderer {
    pipeline: wgpu::RenderPipeline,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
}

impl ShapeRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shape_shader"),
            source: wgpu::ShaderSource::Wgsl(SHAPE_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shape_pipeline_layout"),
            bind_group_layouts: &[],
            ..Default::default()
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shape_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
        }
    }

    fn screen_to_clip(x: f32, y: f32, width: f32, height: f32) -> (f32, f32) {
        let cx = (x / width) * 2.0 - 1.0;
        let cy = 1.0 - (y / height) * 2.0;
        (cx, cy)
    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn push_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        screen_w: f32,
        screen_h: f32,
    ) {
        let (x0, y0) = Self::screen_to_clip(x, y, screen_w, screen_h);
        let (x1, y1) = Self::screen_to_clip(x + w, y + h, screen_w, screen_h);

        let base = self.vertices.len() as u32;
        self.vertices.push(Vertex {
            position: [x0, y0],
            color,
        });
        self.vertices.push(Vertex {
            position: [x1, y0],
            color,
        });
        self.vertices.push(Vertex {
            position: [x1, y1],
            color,
        });
        self.vertices.push(Vertex {
            position: [x0, y1],
            color,
        });

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn push_rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radius: f32,
        color: [f32; 4],
        segments: u32,
        screen_w: f32,
        screen_h: f32,
    ) {
        let r = radius.min(w / 2.0).min(h / 2.0);
        let base = self.vertices.len() as u32;

        let corners = [
            (x + r, y + r),
            (x + w - r, y + r),
            (x + w - r, y + h - r),
            (x + r, y + h - r),
        ];
        let angle_step = std::f32::consts::FRAC_PI_2 / segments as f32;

        for (i, &(cx_px, cy_px)) in corners.iter().enumerate() {
            let (cx, cy) = Self::screen_to_clip(cx_px, cy_px, screen_w, screen_h);
            let start_angle = std::f32::consts::FRAC_PI_2 * i as f32;

            self.vertices.push(Vertex {
                position: [cx, cy],
                color,
            });

            for s in 0..=segments {
                let angle = start_angle + angle_step * s as f32;
                let raw_x = cx_px + r * angle.cos();
                let raw_y = cy_px + r * angle.sin();

                let (mirrored_x, mirrored_y) = match i {
                    0 => (raw_x, raw_y),
                    1 => (2.0 * cx_px - raw_x, raw_y),
                    2 => (2.0 * cx_px - raw_x, 2.0 * cy_px - raw_y),
                    3 => (raw_x, 2.0 * cy_px - raw_y),
                    _ => (raw_x, raw_y),
                };

                let (px, py) = Self::screen_to_clip(mirrored_x, mirrored_y, screen_w, screen_h);
                self.vertices.push(Vertex {
                    position: [px, py],
                    color,
                });
            }
        }

        for i in 0..4u32 {
            let center = base + i * (segments + 2);
            let first_outer = center + 1;
            for s in 0..segments {
                self.indices
                    .extend_from_slice(&[center, first_outer + s, first_outer + s + 1]);
            }
        }

        let strip_base = self.vertices.len() as u32;

        let top_y = y;
        let bot_y = y + h;
        let left_x = x;
        let right_x = x + w;

        let strip_points = [
            (x + r, top_y),
            (x + w - r, top_y),
            (left_x, y + r),
            (right_x, y + r),
            (right_x, y + h - r),
            (left_x, y + h - r),
            (x + r, bot_y),
            (x + w - r, bot_y),
        ];

        for &(px, py) in &strip_points {
            let (cx, cy) = Self::screen_to_clip(px, py, screen_w, screen_h);
            self.vertices.push(Vertex {
                position: [cx, cy],
                color,
            });
        }

        let tl = strip_base;
        let tr = strip_base + 1;
        let lt = strip_base + 2;
        let rt = strip_base + 3;
        let rb = strip_base + 4;
        let lb = strip_base + 5;
        let bl = strip_base + 6;
        let br = strip_base + 7;

        self.indices.extend_from_slice(&[
            tl, tr, rt, tl, rt, lt, lt, rt, rb, lt, rb, lb, lb, rb, br, lb, br, bl,
        ]);
    }

    pub fn upload(&mut self, device: &wgpu::Device) {
        if self.vertices.is_empty() {
            self.vertex_buffer = None;
            self.index_buffer = None;
            self.index_count = 0;
            return;
        }

        self.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shape_vertices"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );

        self.index_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shape_indices"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
        );

        self.index_count = self.indices.len() as u32;
    }

    pub fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        let (Some(vb), Some(ib)) = (&self.vertex_buffer, &self.index_buffer) else {
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, vb.slice(..));
        render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
