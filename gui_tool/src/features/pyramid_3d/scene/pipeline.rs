// ========== 金字塔渲染管线模块 ==========
// 连接 CPU 端数据结构与 GPU 端渲染资源
// 职责：创建 WGPU 资源、上传数据、执行绘制

pub mod buffer;
pub mod pyramid_shape;
pub mod uniforms;
pub mod vertex;

use std::sync::Mutex;

use iced::wgpu::util::DeviceExt;
use iced::widget::shader::Pipeline;
use iced::{wgpu, widget::shader};

use pyramid_shape::PyramidRaw;
use uniforms::Uniforms;
use vertex::Vertex;

/// 金字塔 GPU 渲染管线资源集合
///
/// 包含所有 WGPU 资源，在 Pipeline::new() 中一次性创建，
/// 在 Primitive::prepare() 中更新动态数据，在 Primitive::render() 中执行绘制。
#[derive(Debug)]
pub struct PyramidPipeline {
    /// WGPU 设备引用（克隆存储，用于 render 阶段按需重建深度纹理）
    pub device: wgpu::Device,
    /// 编译好的渲染管线（着色器 + 顶点布局 + 混合模式 + 深度模板）
    pub render_pipeline: wgpu::RenderPipeline,
    /// 顶点缓冲区（18个顶点，静态数据，所有实例共享）
    pub vertex_buffer: wgpu::Buffer,
    /// 实例数据缓冲区（每实例一个 PyramidRaw，动态更新）
    pub instance_buffer: wgpu::Buffer,
    /// Uniform 缓冲区（VP矩阵 + 相机位置 + 光源颜色，每帧更新）
    pub uniform_buffer: wgpu::Buffer,
    /// 绑定组：将 uniform_buffer 绑定到着色器 @group(0)
    pub bind_group: wgpu::BindGroup,
    /// 顶点总数（18）
    pub vertex_count: u32,
    /// 实例总数（动态变化）
    pub instance_count: u32,
    /// 深度纹理（离屏渲染用，按需重建）
    /// 使用 Mutex 实现内部可变性（需 Sync），因为 Iced 的 render() 传入 &self
    pub depth_texture: Mutex<Option<wgpu::Texture>>,
    /// target 纹理的尺寸（由 prepare 阶段从 viewport 获取并存储）
    /// 深度纹理必须与此尺寸严格一致，而非 clip_bounds（clip_bounds 只是子区域）
    pub target_size: Mutex<(u32, u32)>,
}

impl Pipeline for PyramidPipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self
    where
        Self: Sized,
    {
        // 1. 加载并编译 WGSL 着色器
        let shader_source = include_str!("pyramid.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pyramid shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // 2. 创建 BindGroup 布局（@group(0): uniform buffer）
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pyramid bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // 3. 创建管线布局
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pyramid pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // 4. 创建渲染管线（含深度模板状态）
        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("pyramid render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    // 两个缓冲区：顶点数据 + 实例数据
                    buffers: &[Vertex::desc(), PyramidRaw::desc()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None, // 双面渲染（金字塔底面可见）
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });

        // 5. 创建顶点缓冲区（静态：18个顶点，所有实例共享）
        let vertices = PyramidRaw::vertices();
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("pyramid vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // 6. 创建实例缓冲区（动态：初始容量1个实例，后续按需扩容）
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pyramid instance buffer"),
            size: std::mem::size_of::<PyramidRaw>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 7. 创建 Uniform 缓冲区
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pyramid uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 8. 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pyramid bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            device: device.clone(),
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            uniform_buffer,
            bind_group,
            vertex_count: vertices.len() as u32,
            instance_count: 0,
            depth_texture: Mutex::new(None),
            target_size: Mutex::new((0, 0)),
        }
    }
}

impl PyramidPipeline {
    /// 上传实例数据和 Uniform 数据到 GPU
    ///
    /// 每帧在 Primitive::prepare() 中调用。
    /// - 实例缓冲区按需扩容
    /// - Uniform 数据直接写入
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pyramid_raws: &[PyramidRaw],
        uniforms: &Uniforms,
    ) {
        // 上传实例数据（按需扩容实例缓冲区）
        if !pyramid_raws.is_empty() {
            let required_size =
                (std::mem::size_of::<PyramidRaw>() * pyramid_raws.len()) as wgpu::BufferAddress;

            // 如果当前缓冲区不够大，重新创建
            if required_size > self.instance_buffer.size() {
                self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("pyramid instance buffer"),
                    size: required_size,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(pyramid_raws),
            );
            self.instance_count = pyramid_raws.len() as u32;
        }

        // 上传 Uniform 数据
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    /// 存储 target 纹理尺寸（由 prepare 阶段从 viewport 获取）
    pub fn set_target_size(&self, width: u32, height: u32) {
        let mut size = self.target_size.lock().unwrap();
        *size = (width, height);
    }

    /// 管理深度纹理：按需创建或重建
    ///
    /// 使用存储的 Device 引用，在 prepare 或 render 阶段均可调用。
    /// 当尺寸变化或纹理不存在时，重新创建深度纹理。
    /// 通过 Mutex 实现内部可变性，兼容 Iced 的 &self render 签名。
    pub fn ensure_depth_texture(&self, width: u32, height: u32) -> bool {
        if width == 0 || height == 0 {
            return false;
        }

        let mut depth = self.depth_texture.lock().unwrap();
        let needs_new = match depth.as_ref() {
            Some(tex) => tex.width() != width || tex.height() != height,
            None => true,
        };

        if needs_new {
            *depth = Some(self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("pyramid depth texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[wgpu::TextureFormat::Depth32Float],
            }));
        }

        true
    }

    /// 执行 GPU 绘制
    ///
    /// 在 Primitive::render() 中调用，自建带深度附件的离屏 RenderPass。
    /// 如果深度纹理尺寸与 clip_bounds 不匹配，则用存储的 Device 重建。
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        // 使用存储的 target 尺寸（来自 viewport.physical_size），而非 clip_bounds
        // clip_bounds 只是 target 纹理的子区域，用它创建深度纹理会导致尺寸不匹配
        let (tw, th) = *self.target_size.lock().unwrap();
        let w = tw.max(1);
        let h = th.max(1);

        // 确保深度纹理尺寸与 target 纹理严格一致
        self.ensure_depth_texture(w, h);

        let depth = self.depth_texture.lock().unwrap();
        let depth_texture = match depth.as_ref() {
            Some(tex) => tex,
            None => return,
        };

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 创建带深度附件的离屏 RenderPass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pyramid render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // 保留 Iced 已绘制内容
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0), // 每帧清除深度缓冲
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        // 设置视口和裁剪区域：覆盖整个 target 纹理
        // clip_bounds 是窗口级坐标（含 menu bar 偏移），不能直接用于 target 子区域
        // shader widget 已占满 content_panel，全 target 渲染安全
        render_pass.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
        render_pass.set_scissor_rect(0, 0, w, h);

        // 绑定资源并执行实例化绘制
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..self.instance_count);
    }
}
