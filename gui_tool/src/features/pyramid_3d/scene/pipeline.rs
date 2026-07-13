pub mod buffer;
pub mod pyramid_shape;
pub mod uniforms;
pub mod vertex;

use iced::widget::shader::Pipeline;
use iced::{wgpu, Rectangle, Size};

use buffer::Buffer;
use pyramid_shape::PyramidRaw;
use uniforms::Uniforms;

/// 金字塔渲染管线——持有所有 GPU 资源，管理每帧的数据上传和绘制
///
/// 架构层次（参考 iced custom_shader 案例）：
/// 1. `new()`    ：一次性创建管线（GPU 编译着色器、创建缓冲区、绑定组等）
/// 2. `update()` ：每帧调用，上传更新 uniform 和实例数据
/// 3. `render()` ：每帧调用，创建 RenderPass 并发出 draw call
#[derive(Debug)]
pub struct PyramidPipeline {
    /// wgpu 渲染管线——着色器、顶点布局、深度模板、混合等全部渲染状态
    pipeline: wgpu::RenderPipeline,
    /// 顶点缓冲区——18 个金字塔顶点（pos + normal），所有实例共享
    vertices: wgpu::Buffer,
    /// 实例缓冲区——每个金字塔的变换矩阵 + 颜色，动态扩容
    instances: Buffer,
    /// 统一变量缓冲区——投影矩阵 + 相机位置 + 光照颜色
    uniforms: wgpu::Buffer,
    /// 绑定组——将 uniform 缓冲区绑定到着色器 @group(0) @binding(0)
    uniform_bind_group: wgpu::BindGroup,
    /// 深度纹理尺寸——追踪窗口变化以重建深度缓冲
    depth_texture_size: Size<u32>,
    /// 深度纹理视图——Z-buffer 深度测试
    depth_view: wgpu::TextureView,
}

impl Pipeline for PyramidPipeline {
    fn new(
        device: &iced::wgpu::Device,
        queue: &iced::wgpu::Queue,
        format: iced::wgpu::TextureFormat,
    ) -> Self
    where
        Self: Sized,
    {
        Self::new_inner(device, queue, format)
    }
}

impl PyramidPipeline {
    /// 创建渲染管线及其所有 GPU 资源（共 9 步）
    fn new_inner(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        // ======== 1. 顶点缓冲区（所有实例共享） ========
        let vertices_data = PyramidRaw::vertices();
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pyramid vertex buffer"),
            size: std::mem::size_of_val(&vertices_data) as u64,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: true,
        });
        // 映射后写入顶点数据
        {
            let mut mapped = vertices.slice(..).get_mapped_range_mut();
            let bytes = bytemuck::cast_slice(&vertices_data);
            mapped[..bytes.len()].copy_from_slice(bytes);
        }
        vertices.unmap();

        // ======== 2. 实例缓冲区（动态扩容） ========
        let instances = Buffer::new(
            device,
            "pyramid instance buffer",
            std::mem::size_of::<PyramidRaw>() as u64,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        // ======== 3. Uniform 缓冲区 ========
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pyramid uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ======== 4. 深度纹理（初始 1×1，运行时重建） ========
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pyramid depth texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ======== 5. 绑定组布局（1 个 uniform 入口） ========
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("pyramid bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );

        // ======== 6. 绑定组 ========
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pyramid uniform bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        // ======== 7. 管线布局 ========
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pyramid pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // ======== 8. 着色器 ========
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pyramid shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "pyramid.wgsl"
            ))),
        });

        // ======== 9. 渲染管线 ========
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pyramid pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex::Vertex::desc(), PyramidRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            vertices,
            instances,
            uniforms,
            uniform_bind_group,
            depth_texture_size: Size::new(1, 1),
            depth_view,
        }
    }

    /// 深度纹理重建（窗口尺寸变化时）
    fn update_depth_texture(&mut self, device: &wgpu::Device, size: Size<u32>) {
        if self.depth_texture_size != size {
            let text = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("pyramid depth texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.depth_view = text.create_view(&wgpu::TextureViewDescriptor::default());
            self.depth_texture_size = size;
        }
    }

    /// 每帧更新——上传 uniform 和实例数据到 GPU
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        uniforms: &Uniforms,
        num_instances: usize,
        raw_instances: &[PyramidRaw],
    ) {
        // 1. 深度纹理重建
        self.update_depth_texture(device, target_size);

        // 2. 写入 uniform
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(uniforms));

        // 3. 扩容实例缓冲区
        let needed = (num_instances * std::mem::size_of::<PyramidRaw>()) as u64;
        self.instances.resize(device, needed);

        // 4. 写入实例数据
        queue.write_buffer(
            &self.instances.raw,
            0,
            bytemuck::cast_slice(raw_instances),
        );
    }

    /// 执行渲染——创建 RenderPass 并发出 instanced draw call
    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clip_bounds: Rectangle<u32>,
        num_instances: u32,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pyramid render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
        });

        pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_vertex_buffer(1, self.instances.raw.slice(..));
        // 18 个顶点 × num_instances 个实例
        pass.draw(0..18, 0..num_instances);
    }
}
