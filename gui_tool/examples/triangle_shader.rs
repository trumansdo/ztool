//! 自定义Shader示例 —— 可旋转的彩色三角形
//!
//! 参考 `iced\examples\custom_shader` 的实现模式，展示如何使用 Iced 的
//! `shader` widget 嵌入自定义 wgpu 渲染管线。
//!
//! 功能：渲染一个RGB三色三角形，通过滑块手动控制绕Z轴旋转角度。

use iced::wgpu;
use iced::wgpu::util::DeviceExt;
use iced::widget::{center, column, shader, slider, text};
use iced::{mouse, Element, Fill, Rectangle, Subscription};
use iced::time::Instant;

fn main() -> iced::Result {
    eprintln!("[main] 程序启动，调用 iced::application(...)");
    iced::application(TriangleApp::default, TriangleApp::update, TriangleApp::view)
        .subscription(TriangleApp::subscription)
        .run()
}

struct TriangleApp {
    start: Instant,
    scene: Scene,
}

#[derive(Debug, Clone)]
struct Scene {
    rotation: f32,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    RotationChanged(f32),
}

impl TriangleApp {
    fn new() -> Self {
        eprintln!("[TriangleApp::new] 创建 TriangleApp 实例");
        Self {
            start: Instant::now(),
            scene: Scene { rotation: 0.0 },
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(time) => {
                eprintln!("[TriangleApp::update] Tick 消息 - 时间:{:?}", time);
                self.scene.rotation = (time - self.start).as_secs_f32();
            }
            Message::RotationChanged(angle) => {
                eprintln!("[TriangleApp::update] RotationChanged 消息 - 角度:{}", angle);
                self.scene.rotation = angle;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        eprintln!("[TriangleApp::view] 构建视图");
        let shader_widget = shader(&self.scene).width(Fill).height(Fill);

        let controls = column![
            text(format!("旋转角度: {:.2} rad", self.scene.rotation)).size(16),
            slider(
                0.0..=std::f32::consts::TAU,
                self.scene.rotation,
                Message::RotationChanged
            )
            .step(0.01)
            .width(300),
        ]
        .spacing(10)
        .padding(20);

        center(
            column![shader_widget, controls]
                .spacing(10)
                .width(Fill)
                .height(Fill),
        )
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        eprintln!("[TriangleApp::subscription] 设置帧订阅");
        iced::window::frames().map(Message::Tick)
    }
}

impl Default for TriangleApp {
    fn default() -> Self {
        eprintln!("[TriangleApp::default] Default trait 调用，委托给 new()");
        Self::new()
    }
}

// ============ shader::Program 实现 ============

impl<Message> shader::Program<Message> for Scene {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        eprintln!("[Scene::draw] Shader draw 回调 - rotation:{}", self.rotation);
        Primitive {
            rotation: self.rotation,
        }
    }
}

// ============ Primitive ============

#[derive(Debug)]
struct Primitive {
    rotation: f32,
}

impl shader::Primitive for Primitive {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Pipeline,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        eprintln!("[Primitive::prepare] 准备渲染数据 - rotation:{}", self.rotation);
        let rotation_matrix = glam::Mat4::from_rotation_z(self.rotation);
        let uniforms = Uniforms {
            rotation: rotation_matrix,
        };
        queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        eprintln!("[Primitive::render] 执行渲染");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("triangle render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.08,
                        g: 0.08,
                        b: 0.12,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );

        pass.set_pipeline(&pipeline.render_pipeline);
        pass.set_bind_group(0, &pipeline.bind_group, &[]);
        pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }
}

// ============ Pipeline ============

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    rotation: glam::Mat4,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

const TRIANGLE_SHADER: &str = r#"
struct Uniforms {
    rotation: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.rotation * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

impl shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        eprintln!("[Pipeline::new] 开始初始化管线 - format:{:?}", format);
        let vertices = [
            Vertex {
                position: [0.0, 0.6],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.6, -0.4],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.6, -0.4],
                color: [0.0, 0.0, 1.0],
            },
        ];

        eprintln!("[Pipeline::new] 创建顶点缓冲区");
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("triangle vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        eprintln!("[Pipeline::new] 创建 Uniform 缓冲区");
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("triangle uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let initial_uniform = Uniforms {
            rotation: glam::Mat4::IDENTITY,
        };
        queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&initial_uniform));
        eprintln!("[Pipeline::new] 写入初始 Uniform 数据");

        eprintln!("[Pipeline::new] 创建 BindGroupLayout");
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("triangle bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("triangle bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        eprintln!("[Pipeline::new] 创建 BindGroup");

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        eprintln!("[Pipeline::new] 创建 PipelineLayout");

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(TRIANGLE_SHADER)),
        });
        eprintln!("[Pipeline::new] 创建 ShaderModule");

        let vertex_attributes = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attributes,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        eprintln!("[Pipeline::new] 创建 RenderPipeline");

        Self {
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
        }
    }
}
