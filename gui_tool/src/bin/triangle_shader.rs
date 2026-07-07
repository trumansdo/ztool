//! 三角形渲染 — 基于 Iced + WGPU 的三角形示例

use glam::{self, Mat4, mat4, vec3, vec4};
use std::{mem, time::Instant};

use iced::{
    Element,
    Length::Fill,
    Task,
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
        BlendState, Buffer, BufferAddress, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
        DepthBiasState, DepthStencilState, Extent3d, FragmentState, FrontFace, LoadOp, MultisampleState, Operations,
        PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
        RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
        RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StencilState, StoreOp, Texture,
        TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, VertexAttribute,
        VertexBufferLayout, VertexState, VertexStepMode,
        util::{BufferInitDescriptor, DeviceExt},
        wgt::{BufferDescriptor, TextureDescriptor},
    },
    widget::shader::{Pipeline, Primitive, Program, Viewport},
    window,
};
const OPENGL_TO_WGPU_MATRIX: Mat4 = mat4(
    // 列1: x 不变
    vec4(1.0, 0.0, 0.0, 0.0),
    // 列2: y 不变
    vec4(0.0, 1.0, 0.0, 0.0),
    // 列3: z' = 0.5 × z
    vec4(0.0, 0.0, 0.5, 0.0),
    // 列4: z' += 0.5 × w, w' = w
    vec4(0.0, 0.0, 0.5, 1.0),
);
// ── 管线资源 ──
#[derive(Debug)]
struct TrianglePipeline {
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub vp_buffer: Buffer,
    pub rotation_buffer: Buffer,
    pub bind_group: BindGroup,
    pub vertex_count: u32,
    pub depth_texture: Option<Texture>,
    pub depth_texture_view: Option<TextureView>,
}

impl Pipeline for TrianglePipeline {
    fn new(device: &iced::wgpu::Device, _queue: &iced::wgpu::Queue, format: iced::wgpu::TextureFormat) -> Self {
        let shader_str = include_str!("triangle_shader.wgsl");
        //  创建着色器模块 (device.create_shader_module)
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: ShaderSource::Wgsl(shader_str.into()),
        });
        //  创建管线布局 (device.create_pipeline_layout, bind_group_layouts 为空)
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Triangle BindGroup Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: iced::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: iced::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Triangle Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        //  创建渲染管线 (device.create_render_pipeline)
        //   - 顶点着色器入口 vs_main
        //   - 片段着色器入口 fs_main
        //   - 图元拓扑 TriangleList
        //   - 深度格式 Depth32Float, depth_compare = Less
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vx_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: 6 * mem::size_of::<f32>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: iced::wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: iced::wgpu::VertexFormat::Float32x3,
                            offset: 3 * mem::size_of::<f32>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fg_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });
        //  构建三角形顶点数据（3个顶点，用原生数组而非 Vertex 结构体）
        let vertices: [f32; 18] = [
            -0.5, -0.5, 0.0, 1.0, 0.0, 0.0, // 顶点0: 位置(-0.5,-0.5,0), 颜色红
            0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // 顶点1: 位置(0.5,-0.5,0), 颜色绿
            0.0, 0.5, 0.0, 0.0, 0.0, 1.0, // 顶点2: 位置(0.0,0.5,0), 颜色蓝
        ];

        //  创建顶点缓冲区 (device.create_buffer_init)
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertices),
            usage: BufferUsages::VERTEX,
        });
        //  创建 vp_buffer
        let vp_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("VP Buffer"),
            size: std::mem::size_of::<Mat4>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        //  创建 rotation_buffer
        let rotation_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Rotation Buffer"),
            size: std::mem::size_of::<Mat4>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        //  创建 bind_group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: vp_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: rotation_buffer.as_entire_binding(),
                },
            ],
        });
        //  返回 Self (depth_texture 初始为 None)
        Self {
            render_pipeline,
            vertex_buffer,
            vp_buffer,
            rotation_buffer,
            bind_group,
            vertex_count: 3,
            depth_texture: None,
            depth_texture_view: None,
        }
    }
}

// ── 图元 ──
#[derive(Debug)]
struct TrianglePrimitive {
    rotation_matrix: Mat4,
}

impl Primitive for TrianglePrimitive {
    type Pipeline = TrianglePipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &iced::wgpu::Device,
        queue: &iced::wgpu::Queue,
        bounds: &iced::Rectangle,
        viewport: &Viewport,
    ) {
        //  检测 bounds 宽高，与现有 depth_texture 尺寸对比，未变则跳过重建
        //  创建深度纹理 (device.create_texture)
        //   - 格式 Depth32Float
        //   - 用途 RENDER_ATTACHMENT
        //   - 尺寸匹配视口物理像素
        let physical_size = viewport.physical_size();
        let bw = physical_size.width;
        let bh = physical_size.height;
        let need_new: bool = match &pipeline.depth_texture {
            Some(x) => x.width() != bw || x.height() != bh,
            None => true,
        };
        if need_new && bw > 0 && bh > 0 {
            //  将纹理视图存入 pipeline.depth_texture
            pipeline.depth_texture = Some(device.create_texture(&TextureDescriptor {
                label: Some("Triangle depth texture"),
                size: Extent3d {
                    width: bw,
                    height: bh,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[iced::wgpu::TextureFormat::Depth32Float],
            }));
        }
        //  创建深度纹理视图 (texture.create_view)
        pipeline.depth_texture_view = if let Some(ref tex) = pipeline.depth_texture {
            Some(tex.create_view(&TextureViewDescriptor::default()))
        } else {
            None
        };
        if physical_size.width > 0 && physical_size.height > 0 {
            //  计算并上传 uniform 数据（投影矩阵等）到 pipeline.uniform_buffer
            // 这里面对应相机概念
            let eye = vec3(0.0, 1.5, 3.0);
            let center = glam::Vec3::ZERO;
            let up = glam::Vec3::Y;
            let view_matrix = glam::Mat4::look_at_rh(eye, center, up);
            let fov_y = 45.0;
            let aspect = bounds.width / bounds.height;
            let z_near = 0.1;
            let z_far = 100.0;
            let projection_matrix = glam::Mat4::perspective_rh(fov_y, aspect, z_near, z_far);

            let vp_matrix = OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix;
            queue.write_buffer(&pipeline.vp_buffer, 0, bytemuck::cast_slice(&[vp_matrix]));
            queue.write_buffer(&pipeline.rotation_buffer, 0, bytemuck::cast_slice(&[self.rotation_matrix]));
        }
    }

    fn draw(&self, _pipeline: &Self::Pipeline, _render_pass: &mut iced::wgpu::RenderPass<'_>) -> bool {
        // 不使用 draw 方法，用 render 自建 RenderPass
        false
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut iced::wgpu::CommandEncoder,
        target: &iced::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        //  获取深度纹理，为空则 return
        if pipeline.depth_texture == None {
            return;
        }
        //  创建深度纹理 view
        let depth_view = pipeline
            .depth_texture_view
            .as_ref()
            .unwrap();

        //  encoder.begin_render_pass
        //   - 颜色附件: view=target, load=Load, store=Store
        //   - 深度附件: load=Clear(1.0), store=Store
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Triangle RenderPass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        //  render_pass.set_viewport
        render_pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );
        //  render_pass.set_scissor_rect
        render_pass.set_scissor_rect(clip_bounds.x, clip_bounds.y, clip_bounds.width, clip_bounds.height);
        //  render_pass.set_pipeline
        render_pass.set_pipeline(&pipeline.render_pipeline);
        //  render_pass.set_vertex_buffer
        render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        //  render_pass.draw(0..vertex_count, 0..1)
        render_pass.draw(0..pipeline.vertex_count, 0..1);
    }
}

// ── 场景 ──
#[derive(Debug, Clone)]
struct Scene {
    start: Instant,
}

impl Scene {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    fn rotation_matrix(&self) -> Mat4 {
        let t = self
            .start
            .elapsed()
            .as_secs_f32();

        // z轴旋转
        let spin_angle = t * 1.0;
        Mat4::from_rotation_z(spin_angle)
    }
}

impl<Message> Program<Message> for Scene {
    type State = ();
    type Primitive = TrianglePrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        TrianglePrimitive {
            rotation_matrix: self.rotation_matrix(),
        }
    }
}

// ── App ──
#[derive(Debug)]
struct App {
    scene: Scene,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl App {
    fn title(&self) -> String {
        "三角形渲染".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        iced::widget::shader(&self.scene)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Tick)
    }
}

pub fn main() -> iced::Result {
    iced::application(|| App { scene: Scene::new() }, App::update, App::view)
        .title(App::title)
        .window(window::Settings {
            size: iced::Size::new(800.0, 600.0),
            ..window::Settings::default()
        })
        .subscription(App::subscription)
        .run()
}
