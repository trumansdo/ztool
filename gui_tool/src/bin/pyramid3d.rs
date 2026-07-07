//! =========================================================================
//! 金字塔 3D 渲染 — 基于 Iced + WGPU 的旋转金字塔示例
//! =========================================================================
//!
//! ── 3D 渲染流水线全景图 ──
//!
//!   CPU 顶点数据 (Vertex[])
//!        │
//!        ▼  queue.write_buffer() 拷贝到 GPU Buffer
//!   GPU 顶点缓冲区 (Vertex Buffer)            GPU Uniform 缓冲区 (MVP 矩阵等)
//!        │                                           │
//!        └──────────────┬────────────────────────────┘
//!                       ▼
//!             GPU 顶点着色器 (vs_main)
//!             · 从 Vertex Buffer 读取 position + color
//!             · 从 Uniform Buffer 读取 vp_matrix + rotation_matrix
//!             · 执行 MVP 变换：模型空间 → 裁剪空间
//!                       │
//!                       ▼
//!             图元装配 (Primitive Assembly)
//!             · 将顶点组装成三角形
//!                       │
//!                       ▼
//!             光栅化 (Rasterization)
//!             · 三角形 → 片段（像素级离散化）
//!             · 深度插值、属性插值（颜色→逐片段插值）
//!                       │
//!                       ▼
//!             深度测试 (Depth Test)
//!             · 比较片段深度值与深度缓冲区
//!             · 被遮挡的片段在此被丢弃
//!                       │
//!                       ▼
//!             GPU 片段着色器 (fs_main)
//!             · 对每个存活片段计算最终颜色
//!             · 本示例直接透传颜色（无光照计算）
//!                       │
//!                       ▼
//!             颜色混合 (Blending) → 帧缓冲区 (Framebuffer)
//!             · 输出到屏幕
//!
//!
//! ── 坐标空间变换链 ──
//!
//!   模型空间 (Model Space)         ← 顶点的原始坐标，以模型自身为原点
//!        │  M = Model 矩阵（旋转矩阵）
//!        ▼
//!   世界空间 (World Space)         ← 所有物体在同一个坐标系中
//!        │  V = View 矩阵（视图矩阵）
//!        ▼
//!   视图空间 / 相机空间 (View/Camera Space)
//!        │                     ← 以相机为原点，相机朝向为 -Z
//!        │  P = Projection 矩阵（投影矩阵）
//!        ▼
//!   裁剪空间 (Clip Space)         ← 齐次坐标 (x,y,z,w)，超出 [-w,w] 的被裁剪
//!        │  透视除法（自动，由 GPU 硬件执行）
//!        ▼
//!   NDC (归一化设备坐标)           ← x,y,z ∈ [-1,1] (OpenGL) 或 [0,1] (WGPU Z)
//!        │  视口变换（自动，由 GPU 硬件执行）
//!        ▼
//!   屏幕空间 (Screen Space)       ← 像素坐标
//!
//!
//! ── 矩阵详解 ──
//!
//!   M (Model)    = 旋转矩阵（本示例：陀螺仪进动 — 倾斜自转轴 + 绕Y轴进动）
//!                   将顶点从模型空间变换到世界空间
//!
//!   V (View)     = 视图矩阵 (look_at_rh)
//!                   将世界空间坐标变换到视图（相机）空间
//!                   相机位置: (0, 1.5, 3.0)，看向原点，上方向为 Y 轴
//!
//!   P (Projection)= 透视投影矩阵 (perspective_rh)
//!                   将视图空间坐标变换到裁剪空间
//!                   FOV=45°, 近平面=0.1, 远平面=100.0
//!
//!   MVP = P × V × M
//!         即：投影矩阵 × 视图矩阵 × 模型矩阵
//!         GPU 中分两步：rotation_matrix(M) × position → world_pos
//!                      vp_matrix(P×V) × world_pos → clip_pos
//!
//!
//! ── OPENGL_TO_WGPU_MATRIX 的作用详解 ──
//!
//!   OpenGL 的 NDC Z 范围是 [-1, 1]（近平面=-1，远平面=1）
//!   WGPU / Vulkan / D3D12 的 NDC Z 范围是 [0, 1]（近平面=0，远平面=1）
//!
//!   glam 的 Mat4::perspective_rh() 默认按 OpenGL 约定生成投影矩阵（Z ∈ [-1,1]），
//!   而 WGPU 期望 Z ∈ [0,1]。因此需要在投影矩阵前面乘一个修正矩阵，
//!   将 OpenGL 风格的投影矩阵转换为 WGPU 兼容的投影矩阵。
//!
//!   修正矩阵的推导：
//!     设 OpenGL 投影后齐次坐标为 (x, y, z, w)
//!     透视除法后 NDC Z = z / w，范围 [-1, 1]
//!
//!     需要变换为 WGPU NDC Z'，范围 [0, 1]
//!     Z' = 0.5 * Z + 0.5 = 0.5 * (z/w) + 0.5 = (0.5*z + 0.5*w) / w
//!
//!     即在裁剪空间中：z' = 0.5*z + 0.5*w，w' = w，x' = x，y' = y
//!
//!       [ x' ]   [ 1  0   0   0 ] [ x ]
//!       [ y' ] = [ 0  1   0   0 ] [ y ]
//!       [ z' ]   [ 0  0  0.5 0.5 ] [ z ]
//!       [ w' ]   [ 0  0   0   1 ] [ w ]
//!
//!   注意：glam 使用列主序 (column-major)，每列对应 mat4 的一个 vec4 参数。
//!   Y 轴方向在 OpenGL 和 WGPU 之间也可能不同（纹理坐标），但本例只涉及 3D 顶点
//!   坐标的 MVP 变换，且使用了 Iced 框架的 viewport 处理，故未额外翻转 Y 轴。
//!
//!
//! ── Rust ↔ GPU 代码的数据流转 ──
//!
//!   Rust 侧:
//!     1. 定义 bytemuck::Pod + Zeroable 结构体 (Vertex, Uniforms)
//!        → Pod 保证内存布局与 C 兼容，可安全地当作字节数组传输
//!     2. device.create_buffer_init() → 创建 Vertex Buffer / Index Buffer
//!        将顶点数据上传到 GPU 显存
//!     3. device.create_buffer() → 创建 Uniform Buffer（可写缓冲区）
//!     4. queue.write_buffer() → 每帧将 CPU 侧的 Uniforms 拷贝到 GPU Uniform Buffer
//!     5. device.create_bind_group() → 将 Uniform Buffer 绑定到着色器的 @group(0)
//!
//!   GPU 侧 (WGSL):
//!     1. @group(0) @binding(0) var<uniform> uniforms: Uniforms;
//!        → 声明 Uniform 变量，GPU 自动从 Bind Group 0 读取数据
//!     2. @location(0) position: vec3<f32>
//!        → 对应 Rust 侧 VertexBufferLayout 中 shader_location=0 的属性
//!
//!   Buffer 类型:
//!     · Vertex Buffer  → 顶点位置 + 颜色，每顶点一份数据
//!     · Index Buffer   → 索引（减少重复顶点），三角形面由索引组合
//!     · Uniform Buffer → MVP 矩阵等逐帧变化的数据，CPU→GPU 单向传输
//!     · Bind Group     → 把 Buffer 绑定到着色器的 @group @binding 声明
//!
//!
//! ── Uniforms 为什么是必需的 ──
//!
//!   CPU 内存和 GPU 显存是物理上独立的两个地址空间。
//!   CPU 侧的 Mat4 变量存在于系统内存中，GPU 无法直接访问。
//!   Uniform Buffer 是 CPU → GPU 的唯一数据通道：
//!     CPU: 构造 Uniforms { vp_matrix, rotation_matrix }
//!       → bytemuck::cast_slice() 转为字节序列
//!       → queue.write_buffer() 拷贝到 GPU 显存
//!     GPU: 通过 @group(0) @binding(0) 读取 Uniform Buffer 内容
//!
//!   如果没有 Uniform Buffer，着色器就无法知道相机位置、旋转角度等信息。
//!
//!
//! ── 深度缓冲区的作用 ──
//!
//!   深度缓冲区 (Depth Buffer) 是 GPU 内部的二维数组，每个像素存储一个深度值。
//!   作用：解决遮挡问题 — 确保后方物体被前方物体正确遮挡。
//!
//!   流程：
//!     1. 每个片段光栅化后得到插值深度值
//!     2. 深度测试：当前片段深度 vs 深度缓冲区已有深度
//!        depth_compare = Less → 只有更近（深度值更小）的片段通过
//!     3. 通过 → 更新颜色缓冲 + 更新深度缓冲
//!        未通过 → 丢弃该片段
//!
//!   本示例中，金字塔的三角形面可能相互遮挡（取决于旋转角度），
//!   深度缓冲区确保正确绘制可见面。
//!
//! =========================================================================

use std::time::Instant;

use iced::{
    Length::Fill,
    Task,
    window,
    widget::shader::{Primitive, Pipeline, Program, Viewport},
    Element,
};

use iced::wgpu::util::DeviceExt;

use glam::{Mat4, Vec3, mat4, vec3, vec4};

// =========================================================================
// WGSL 着色器代码（嵌入为 Rust 字符串常量）
// =========================================================================
//
// 本段 WGSL 是 GPU 上运行的着色器程序，编译时作为字符串嵌入，运行时由 WGPU 编译。
//
// 入口点说明：
//   @vertex  fn vs_main → 顶点着色器入口，GPU 对每个顶点调用一次
//   @fragment fn fs_main → 片段着色器入口，GPU 对每个光栅化后的片段调用一次
//
// 数据绑定说明：
//   Rust 侧 Vertex Buffer → WGSL @location(0) position, @location(1) color
//     映射：Rust VertexBufferLayout 中 array_stride=24字节（6个f32），
//           shader_location=0 读取前12字节（position vec3），
//           shader_location=1 读取后12字节（color vec3）
//   Rust 侧 Uniform Buffer → WGSL @group(0) @binding(0) uniforms
//
// @builtin(position) 的语义：
//   顶点着色器输出的 clip_position 是裁剪空间坐标 (vec4<f32>)。
//   GPU 硬件自动执行：
//     1. 透视除法：clip_position.xyz / clip_position.w → NDC 坐标
//     2. 视口变换：NDC → 屏幕像素坐标
//   片段着色器不直接访问 @builtin(position)，但 GPU 内部使用它做光栅化。
//
const VS_SHADER: &str = r#"
// ── 顶点输入结构体 ──
// @location(0): 从 Rust 侧 Vertex Buffer 的 shader_location=0 读取 position
// @location(1): 从 Rust 侧 Vertex Buffer 的 shader_location=1 读取 color
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

// ── 顶点着色器输出 / 片段着色器输入结构体 ──
// @builtin(position): GPU 内置语义，标识裁剪空间坐标
//   顶点着色器输出后，GPU 自动做透视除法 + 视口变换得到屏幕坐标
// @location(0): 传递给片段着色器的插值后颜色
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// ── Uniform 结构体（与 Rust 侧 Uniforms 布局一致） ──
// mat4x4<f32> 在 WGSL 中是列主序，与 Rust glam Mat4 的 to_cols_array_2d() 兼容
struct Uniforms {
    vp_matrix: mat4x4<f32>,       // VP 矩阵 = OPENGL_TO_WGPU_MATRIX × P × V
    rotation_matrix: mat4x4<f32>, // 旋转矩阵 = M（陀螺仪矩阵）
}

// ── Uniform Buffer 绑定 ──
// @group(0) @binding(0): 对应 Rust 侧 BindGroup 的 binding=0
// 由 Rust 侧 queue.write_buffer() 每帧更新
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// ── 顶点着色器 ──
// 每个顶点调用一次。输入：VertexInput（来自 Vertex Buffer），输出：VertexOutput。
// 核心逻辑：
//   1. uniforms.rotation_matrix × position → world_pos（模型空间→世界空间）
//   2. uniforms.vp_matrix × world_pos → clip_pos（世界空间→裁剪空间）
//   3. 颜色直接透传给片段着色器
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // 将局部坐标扩展到齐次坐标 vec4(position, 1.0)
    // rotation_matrix (M) 将顶点从模型空间变换到世界空间
    let world_pos = uniforms.rotation_matrix * vec4<f32>(in.position, 1.0);
    // vp_matrix (P×V) 将世界空间坐标变换到裁剪空间
    // 注意：OPENGL_TO_WGPU_MATRIX 已经在 vp_matrix 中（Rust 侧乘入）
    let clip_pos = uniforms.vp_matrix * world_pos;
    return VertexOutput(clip_pos, in.color);
}

// ── 片段着色器 ──
// 每个光栅化后的片段调用一次。输入：VertexOutput（经过透视校正插值）。
// @location(0) 输出：颜色值，写入帧缓冲区的第一个颜色附件。
// 核心逻辑：直接将顶点颜色透传，alpha=1.0（完全不透明）
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

// =========================================================================
// Vertex 结构体 — 顶点数据格式
// =========================================================================
//
// 每个顶点包含两部分数据：
//   position: Vec3  — 3D 位置 (x, y, z)，对应 WGSL @location(0)
//   color:    Vec3  — RGB 颜色 (r, g, b)，对应 WGSL @location(1)
//
// 使用 glam 的 Vec3/Mat4 与 Iced 自身风格一致。
//
// #[repr(C)]: 保证内存布局与 C 结构体一致，按顺序排列字段，无填充优化。
// bytemuck::Pod + Zeroable: 允许将 &[Vertex] 直接转换为 &[u8]，
//   从而通过 queue.write_buffer() 上传到 GPU。
//
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    color: Vec3,
}

impl Vertex {
    // ── 顶点属性定义 ──
    // 使用 wgpu::vertex_attr_array! 宏自动计算 offset
    // location 0: position (Vec3, 12 字节), location 1: color (Vec3, 12 字节)
    const ATTRIBS: [iced::wgpu::VertexAttribute; 2] = iced::wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];

    // ── 顶点缓冲区布局描述 ──
    // array_stride: 每个顶点占用的字节数（= sizeof(Vertex) = 2×Vec3 = 24 字节）
    // step_mode: Vertex — 每个顶点步进一次（非 Instance 模式）
    // attributes: 顶点属性的 shader_location → 格式映射
    // desc — 顶点缓冲区布局描述
    // 返回 VertexBufferLayout 告诉 GPU 顶点数据的内存排布方式
    fn desc() -> iced::wgpu::VertexBufferLayout<'static> {
        iced::wgpu::VertexBufferLayout {
            // 每个顶点的字节跨度（2 × Vec3 = 24 字节）
            array_stride: std::mem::size_of::<Self>() as iced::wgpu::BufferAddress,
            // 逐顶点步进（非实例化模式）
            step_mode: iced::wgpu::VertexStepMode::Vertex,
            // 顶点属性列表（位置 + 颜色，各 Float32x3）
            attributes: &Self::ATTRIBS,
        }
    }
}

// =========================================================================
// Uniforms 结构体 — CPU→GPU 数据传输载体
// =========================================================================
//
// 包含两个 4×4 矩阵：
//   vp_matrix:       VP 矩阵（OPENGL_TO_WGPU_MATRIX × 投影 × 视图）
//   rotation_matrix: 旋转矩阵（陀螺仪矩阵）
//
// 使用 glam 的 Vec3/Mat4 与 Iced 自身风格一致。
//
// #[repr(C)]: 确保字段连续排列，与 WGSL 的 Uniforms 结构体字节对齐一致。
// bytemuck::Pod + Zeroable: 允许零拷贝序列化上传到 GPU。
//
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    vp_matrix: Mat4,
    rotation_matrix: Mat4,
}

// =========================================================================
// OPENGL_TO_WGPU_MATRIX — 坐标系修正矩阵
// =========================================================================
//
// 作用：将 OpenGL 风格的投影矩阵转换为 WGPU 兼容的投影矩阵。
//
// 每个元素的计算原理（列主序，每列一个 vec4）：
//
//   目标：NDC Z 从 [-1,1] 变换到 [0,1]
//   Z' = 0.5 × Z + 0.5
//   齐次坐标下：z' = 0.5×z + 0.5×w, w' = w
//
//   列1: (1, 0, 0, 0) — x 不变
//   列2: (0, 1, 0, 0) — y 不变
//   列3: (0, 0, 0.5, 0.5) — z' = 0.5×z + 0.5×w
//   列4: (0, 0, 0, 1) — w' = w
//
//   注意：Y 轴也可能在不同图形 API 间存在差异（纹理坐标的上下翻转），
//   但在 3D 顶点坐标的 MVP 变换流程中，Iced 的 viewport 处理已做相应调整。
//
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

// =========================================================================
// PyramidPipeline — GPU 渲染管线资源的集合体
// =========================================================================
//
// 包含以下 WGPU 资源，每个字段的作用：
//
//   render_pipeline: 编译好的渲染管线（着色器 + 顶点布局 + 混合模式 + 深度模板）
//                    定义"怎么画" — 这是整个渲染过程的配置核心
//   vertex_buffer:   顶点缓冲区（在 GPU 显存中）
//                    存储金字塔的 18 个顶点（位置 + 颜色）
//                      = 4 个侧面 × 3 + 底面 2 个三角形 × 3
//   uniform_buffer:  Uniform 缓冲区（在 GPU 显存中）
//                    存储 VP 矩阵和旋转矩阵，每帧由 prepare() 更新
//   bind_group:      绑定组 — 将 uniform_buffer 按绑定布局绑定到着色器的 @group(0)
//                    GPU 通过 bind_group 知道"从哪个 buffer 读 uniform 数据"
//   vertex_count:    顶点总数（18），用于 draw() 指定绘制多少个顶点
//   depth_texture:   深度纹理 — 由于 Iced 主 RenderPass 不提供深度附件，
//                    在 prepare() 中自建深度纹理，在 render() 中挂载到离屏 RenderPass
//
#[derive(Debug)]
struct PyramidPipeline {
    render_pipeline: iced::wgpu::RenderPipeline,
    vertex_buffer: iced::wgpu::Buffer,
    uniform_buffer: iced::wgpu::Buffer,
    bind_group: iced::wgpu::BindGroup,
    vertex_count: u32,
    depth_texture: Option<iced::wgpu::Texture>,
}

// =========================================================================
// impl Pipeline for PyramidPipeline — GPU 管线初始化
// =========================================================================
//
// Pipeline trait 是 Iced shader 模块定义的接口，要求实现 new() 构造函数。
// 这在 Iced 框架中只调用一次（应用启动时），完成全部 GPU 资源的创建。
//
impl Pipeline for PyramidPipeline {
    // new — 初始化所有 WGPU 管线资源（仅应用启动时调用一次）
    fn new(
        // WGPU 设备引用：用于创建着色器模块、Buffer、BindGroup、管线等
        device: &iced::wgpu::Device,
        // 命令队列引用（本示例未使用，数据上传在 prepare() 中通过 queue.write_buffer 完成）
        _queue: &iced::wgpu::Queue,
        // 颜色附件像素格式，从 Iced 窗口表面格式传入，与 ColorTargetState.format 匹配
        format: iced::wgpu::TextureFormat,
    ) -> Self {
        println!(">>> Pipeline::new 被调用");
        let shader_module = device.create_shader_module(iced::wgpu::ShaderModuleDescriptor {
            // 调试标签，GPU 调试器（如 RenderDoc）中标识该对象
            label: Some("pyramid shader"),
            // WGSL 源码字符串，GPU 驱动编译为可执行着色器
            source: iced::wgpu::ShaderSource::Wgsl(VS_SHADER.into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&iced::wgpu::BindGroupLayoutDescriptor {
                label: Some("uniforms bind group layout"),
                entries: &[iced::wgpu::BindGroupLayoutEntry {
                    // 对应 WGSL 中 @binding(0) 的索引
                    binding: 0,
                    // 该绑定对哪些着色器阶段可见：VERTEX=仅在顶点着色器可用
                    visibility: iced::wgpu::ShaderStages::VERTEX,
                    ty: iced::wgpu::BindingType::Buffer {
                        // Uniform 缓冲区，GPU 只读，大小受 maxUniformBufferBindingSize 限制
                        ty: iced::wgpu::BufferBindingType::Uniform,
                        // 是否允许 set_bind_group 时指定动态字节偏移（false=固定绑定整个 buffer）
                        has_dynamic_offset: false,
                        // GPU 验证的最小字节数，None=不限制，Some(size)=小于此值报错
                        min_binding_size: None,
                    },
                    // None=单资源绑定，Some(n)=绑定数组（对应 WGSL 的 binding_array）
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&iced::wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline layout"),
                // 绑定点组布局数组，索引 0 对应 set_bind_group(0, ...)
                bind_group_layouts: &[&bind_group_layout],
                // 推送常量：无需 BindGroup 的小数据传递，本示例不使用
                push_constant_ranges: &[],
            });

        let render_pipeline =
            device.create_render_pipeline(&iced::wgpu::RenderPipelineDescriptor {
                label: Some("pyramid render pipeline"),
                // Some=固定管线布局；None=自动派生（但不同管线间不可共享 BindGroup）
                layout: Some(&pipeline_layout),
                vertex: iced::wgpu::VertexState {
                    // 已编译着色器模块
                    module: &shader_module,
                    // 顶点着色器入口函数名，对应 WGSL 中 @vertex fn vs_main
                    entry_point: Some("vs_main"),
                    // 编译选项（调试信息/优化等级），默认即可
                    compilation_options: iced::wgpu::PipelineCompilationOptions::default(),
                    // 顶点缓冲区布局数组，描述顶点数据排布（stride + attribute 列表）
                    buffers: &[Vertex::desc()],
                },
                primitive: iced::wgpu::PrimitiveState {
                    // 每 3 个顶点组成一个三角形
                    topology: iced::wgpu::PrimitiveTopology::TriangleList,
                    // TriangleStrip 时需指定索引格式；TriangleList 下为 None
                    strip_index_format: None,
                    // 逆时针顶点绕序为正面；与 cull_mode 配合决定哪些三角形被剔除
                    front_face: iced::wgpu::FrontFace::Ccw,
                    // None=不剔除（双面渲染），Some(Back)=剔除背面（性能优化约 50%）
                    cull_mode: None,
                    // Fill=实心，Line=线框，Point=点模式
                    polygon_mode: iced::wgpu::PolygonMode::Fill,
                    // 是否允许深度值超出 [0,1] 范围，需要对应 wgpu feature
                    unclipped_depth: false,
                    // 保守光栅化：每个被三角形覆盖的像素都生成片段（需要 feature）
                    conservative: false,
                },
                depth_stencil: Some(iced::wgpu::DepthStencilState {
                    // 深度纹理格式（32位浮点，精度最高）
                    format: iced::wgpu::TextureFormat::Depth32Float,
                    // 渲染时更新深度缓冲区
                    depth_write_enabled: true,
                    // 新片段深度 < 已有深度时才通过（近处遮挡远处）
                    depth_compare: iced::wgpu::CompareFunction::Less,
                    // 模板缓冲状态，本例不使用模板测试
                    stencil: iced::wgpu::StencilState::default(),
                    // 深度偏移，用于阴影贴图防止自遮挡阴影锯齿
                    bias: iced::wgpu::DepthBiasState::default(),
                }),
                multisample: iced::wgpu::MultisampleState {
                    // 多重采样数：1=不启用 MSAA，4=4x MSAA 抗锯齿
                    count: 1,
                    // 采样掩码：!0 = 所有采样点启用
                    mask: !0,
                    // Alpha-to-Coverage：用于草地/毛发等半透明纹理的抗锯齿
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(iced::wgpu::FragmentState {
                    // 已编译着色器模块
                    module: &shader_module,
                    // 片段着色器入口函数名，对应 WGSL 中 @fragment fn fs_main
                    entry_point: Some("fs_main"),
                    compilation_options: iced::wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(iced::wgpu::ColorTargetState {
                        // 颜色附件像素格式，从 Iced 框架传入（与窗口表面格式一致）
                        format,
                        // REPLACE = src_factor:One, dst_factor:Zero, op:Add → 直接覆盖不混合
                        blend: Some(iced::wgpu::BlendState::REPLACE),
                        // 写入哪些通道：ALL = R|G|B|A 全部写入
                        write_mask: iced::wgpu::ColorWrites::ALL,
                    })],
                }),
                // 多视图渲染（VR/立体渲染），None=不启用
                multiview: None,
                // 管线缓存，跨运行复用 GPU 编译结果，None=不启用
                cache: None,
            });

        let vertices = build_pyramid();

        let vertex_buffer = device.create_buffer_init(&iced::wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            // 零拷贝：将 Vertex 数组序列化为原始字节上传 GPU
            contents: bytemuck::cast_slice(&vertices),
            // VERTEX=顶点缓冲区；INDEX=索引缓冲区；UNIFORM=Uniform 缓冲区
            usage: iced::wgpu::BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer(&iced::wgpu::BufferDescriptor {
            label: Some("uniform buffer"),
            // 128 字节（2×64 位 Mat4）
            size: std::mem::size_of::<Uniforms>() as iced::wgpu::BufferAddress,
            // Uniform 读取 + CPU 写入
            usage: iced::wgpu::BufferUsages::UNIFORM | iced::wgpu::BufferUsages::COPY_DST,
            // false=不映射，通过 queue.write_buffer 写入；true=映射后直接写内存
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&iced::wgpu::BindGroupDescriptor {
            label: Some("uniform bind group"),
            // 必须与创建 PipelineLayout 时传入的 BindGroupLayout 一致
            layout: &bind_group_layout,
            entries: &[iced::wgpu::BindGroupEntry {
                // 对应 WGSL 中 @group(0) @binding(0) 的索引
                binding: 0,
                // 绑定整个缓冲区：offset=0，size=None（到末尾）
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            vertex_count: vertices.len() as u32,
            depth_texture: None,
        }
    }
}

// =========================================================================
// build_pyramid() — 构建金字塔的几何数据
// =========================================================================
//
// 金字塔结构：
//          peak (顶点，y=0.5)
//          /|\
//         / | \
//        /  |  \
//       /   |   \
//      /    |    \
//     v3----+----v2   ← 底面 (y=-0.5)
//     |\    |    /|
//     | \   |   / |
//     |  \  |  /  |
//     |   \ | /   |
//     |    \|/    |
//     v0---------v1
//
// 18 个顶点 = 4 个侧面 × 3 个顶点 + 底面 2 个三角形 × 3 个顶点。
// 每个面使用独立顶点（不共享），简化 GPU 管线流程（无索引）。
//
// 颜色分配策略：
//   每个侧面使用统一色调，底部顶点较深，顶部顶点（peak）较浅，
//   产生从深到浅的渐变效果，增强立体感。
//
fn build_pyramid() -> [Vertex; 18] {
    // 底面 Y 坐标（-0.5，金字塔下半部）
    let base_y = -0.5f32;
    // 塔尖 Y 坐标（0.5，金字塔上半部，总高度 = 1.0）
    let peak_y = 0.5f32;

    // 底面正方形的四个角（y = base_y = -0.5）
    let v0 = vec3(-0.5, base_y, -0.5);
    let v1 = vec3(0.5, base_y, -0.5);
    let v2 = vec3(0.5, base_y, 0.5);
    let v3 = vec3(-0.5, base_y, 0.5);
    // 塔尖顶点（y = peak_y = 0.5）
    let peak = vec3(0.0, peak_y, 0.0);

    // 底面 2 个三角形（复用已有顶点位置），6 个顶点
    // 三角形1: v0→v1→v2
    // 三角形2: v0→v2→v3
    let base_color = vec3(0.3, 0.3, 0.3);
    let vertices = [
        // 侧面1: v0→v1→peak (红色系)
        Vertex { position: v0, color: vec3(1.0, 0.2, 0.2) },
        Vertex { position: v1, color: vec3(1.0, 0.2, 0.2) },
        Vertex { position: peak, color: vec3(1.0, 0.5, 0.5) },
        // 侧面2: v1→v2→peak (绿色系)
        Vertex { position: v1, color: vec3(0.2, 1.0, 0.2) },
        Vertex { position: v2, color: vec3(0.2, 1.0, 0.2) },
        Vertex { position: peak, color: vec3(0.5, 1.0, 0.5) },
        // 侧面3: v2→v3→peak (蓝色系)
        Vertex { position: v2, color: vec3(0.2, 0.2, 1.0) },
        Vertex { position: v3, color: vec3(0.2, 0.2, 1.0) },
        Vertex { position: peak, color: vec3(0.5, 0.5, 1.0) },
        // 侧面4: v3→v0→peak (黄色系)
        Vertex { position: v3, color: vec3(1.0, 1.0, 0.2) },
        Vertex { position: v0, color: vec3(1.0, 1.0, 0.2) },
        Vertex { position: peak, color: vec3(1.0, 1.0, 0.5) },
        // 底面三角形1: v0→v1→v2
        Vertex { position: v0, color: base_color },
        Vertex { position: v1, color: base_color },
        Vertex { position: v2, color: base_color },
        // 底面三角形2: v0→v2→v3
        Vertex { position: v0, color: base_color },
        Vertex { position: v2, color: base_color },
        Vertex { position: v3, color: base_color },
    ];

    vertices
}
// =========================================================================
// PyramidPrimitive — 图元：CPU 侧的可绘制对象封装
// =========================================================================
//
// 持有每帧变化的 CPU 数据（本示例中只有旋转矩阵）。
// 实现 shader::Primitive trait，提供 prepare()、draw()、render() 三个阶段。
//
// Iced 框架的渲染流程：
//   1. Program::draw() → 返回 PyramidPrimitive（CPU 侧数据打包）
//   2. Primitive::prepare() → 上传数据到 GPU + 创建/重建深度纹理
//   3. Primitive::draw() → 返回 false，触发 render()
//   4. Primitive::render() → 离屏渲染：自建带深度附件的 RenderPass 完成绘制
//
#[derive(Debug)]
struct PyramidPrimitive {
    rotation_matrix: Mat4,
}

impl Primitive for PyramidPrimitive {
    type Pipeline = PyramidPipeline;

    // prepare — 准备阶段：上传 CPU 数据到 GPU + 创建/重建深度纹理
    // 为什么在这里构造 VP 矩阵：Program::draw() 返回 Primitive 时还未确定视口大小和 bounds，
    // bounds.width / height 决定的 aspect 在窗口大小变化时重新计算 P 矩阵
    fn prepare(
        &self,
        // 可变引用：更新 Uniform Buffer、重建深度纹理
        pipeline: &mut Self::Pipeline,
        // WGPU 设备引用：创建 GPU 资源（纹理、Buffer 等）
        device: &iced::wgpu::Device,
        // 命令队列引用：通过 write_buffer() 上传 CPU 数据到 GPU
        queue: &iced::wgpu::Queue,
        // 控件边界矩形（像素），用于计算投影矩阵 aspect 比
        bounds: &iced::Rectangle,
        // 视口信息，提供物理尺寸用于判断是否需要重建深度纹理
        viewport: &Viewport,
    ) {
        println!(">>> Primitive::prepare 被调用");
        let aspect = bounds.width / bounds.height;
        let view = Mat4::look_at_rh(vec3(0.0, 1.5, 3.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        let vp_matrix = OPENGL_TO_WGPU_MATRIX * proj * view;
        let uniforms = Uniforms {
            vp_matrix,
            rotation_matrix: self.rotation_matrix,
        };
        let size = viewport.physical_size();
        if size.width > 0 && size.height > 0 {
            // 目标 GPU Buffer
            queue.write_buffer(
                &pipeline.uniform_buffer,
                // 目标 Buffer 内字节偏移（0=从头开始）
                0,
                // CPU 数据 → 字节切片，零拷贝上传 GPU
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        let w = bounds.width.ceil() as u32;
        let h = bounds.height.ceil() as u32;
        let needs_new = match &pipeline.depth_texture {
            Some(tex) => tex.width() < w || tex.height() < h,
            None => true,
        };
        if needs_new && w > 0 && h > 0 {
            pipeline.depth_texture = Some(device.create_texture(&iced::wgpu::TextureDescriptor {
                label: Some("pyramid depth texture"),
                size: iced::wgpu::Extent3d {
                    // 纹理宽度（像素），与视口/控件尺寸匹配
                    width: w,
                    // 纹理高度（像素）
                    height: h,
                    // 2D 纹理固定为 1；3D/纹理数组则为层数
                    depth_or_array_layers: 1,
                },
                // 1=不生成 mip 链（深度测试不需要多级精度）
                mip_level_count: 1,
                // 1=不启用 MSAA（深度纹理一般不做多重采样）
                sample_count: 1,
                // D2=标准 2D 纹理
                dimension: iced::wgpu::TextureDimension::D2,
                // 32位浮点深度（精度最高，兼容所有 GPU）
                format: iced::wgpu::TextureFormat::Depth32Float,
                // 用途：作为渲染管线的深度附件
                usage: iced::wgpu::TextureUsages::RENDER_ATTACHMENT,
                // 允许创建的 TextureView 使用此格式
                view_formats: &[iced::wgpu::TextureFormat::Depth32Float],
            }));
        }
    }

    // draw — Iced 框架调用此方法决定是否使用内置渲染路径
    // 返回 false 表示跳过内置渲染，由 render() 执行自定义离屏绘制
    fn draw(
        &self,
        // 渲染管线资源（本示例未使用，实际数据在 prepare/render 中传递）
        _pipeline: &Self::Pipeline,
        // Iced 内置 RenderPass（本示例未使用，采用自定义离屏渲染）
        _render_pass: &mut iced::wgpu::RenderPass<'_>,
    ) -> bool {
        println!(">>> Primitive::draw 被调用");
        // false = 不使用内置渲染路径，触发 render() 执行自定义绘制
        false
    }

    // render — 离屏渲染阶段，执行带深度测试的实际 GPU 绘制
    // Iced 框架为 Shader Widget 创建一个离屏 RenderPass，传入目标纹理视图和编码器，
    // 在此方法中自建带深度附件的 RenderPass 完成 3D 绘制后输出到 target
    fn render(
        &self,
        // 渲染管线资源引用（顶点缓冲、Uniform、绑定组等）
        pipeline: &Self::Pipeline,
        // GPU 命令编码器，录制绘制命令后由框架提交到队列
        encoder: &mut iced::wgpu::CommandEncoder,
        // 目标颜色纹理视图，Iced 框架提供的帧缓冲，最终输出到屏幕
        target: &iced::wgpu::TextureView,
        // 裁剪边界（像素坐标），限制绘制区域，防止越界渲染
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        println!(">>> Primitive::render 被调用");
        let depth_texture = match &pipeline.depth_texture {
            Some(tex) => tex,
            None => return,
        };

        let w = clip_bounds.width.max(1);
        let h = clip_bounds.height.max(1);

        // 从深度纹理创建 TextureView（使用默认描述符，使用完整纹理作为视图）
        let depth_view = depth_texture
            .create_view(&iced::wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&iced::wgpu::RenderPassDescriptor {
            label: Some("pyramid render pass"),
            color_attachments: &[Some(iced::wgpu::RenderPassColorAttachment {
                // 颜色纹理视图（Iced 框架提供的帧缓冲，最终输出到屏幕）
                view: target,
                // None=非 3D 纹理，无需深度切片
                depth_slice: None,
                // None=无多重采样，无需解析目标
                resolve_target: None,
                ops: iced::wgpu::Operations {
                    // Load=保留已有内容（不擦除 Iced 已绘制的内容）
                    load: iced::wgpu::LoadOp::Load,
                    // Store=保存渲染结果到颜色附件
                    store: iced::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(iced::wgpu::RenderPassDepthStencilAttachment {
                // 深度纹理视图（每帧自建的深度缓冲区）
                view: &depth_view,
                depth_ops: Some(iced::wgpu::Operations {
                    // Clear(1.0)=每帧清除深度为最远值，重新进行深度测试
                    load: iced::wgpu::LoadOp::Clear(1.0),
                    // Store=保留深度值供后续读取
                    store: iced::wgpu::StoreOp::Store,
                }),
                // None=不启用模板测试
                stencil_ops: None,
            }),
            ..Default::default()
        });

        render_pass.set_viewport(
            // 视口左上角 x（像素坐标）
            clip_bounds.x as f32,
            // 视口左上角 y（像素坐标）
            clip_bounds.y as f32,
            // 视口宽度（像素）
            w as f32,
            // 视口高度（像素）
            h as f32,
            // min_depth：近平面深度值
            0.0,
            // max_depth：远平面深度值
            1.0,
        );
        render_pass.set_scissor_rect(
            // 裁剪区域左上角 x（像素坐标）
            clip_bounds.x,
            // 裁剪区域左上角 y（像素坐标）
            clip_bounds.y,
            // 裁剪区域宽度
            w,
            // 裁剪区域高度
            h,
        );

        // 绑定渲染管线（着色器/混合/深度/图元等全部状态）
        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(
            // 对应 WGSL 中 @group(index) 的索引
            0,
            &pipeline.bind_group,
            // 动态偏移数组，本例不使用
            &[],
        );
        render_pass.set_vertex_buffer(
            // 顶点缓冲区槽索引
            0,
            // Buffer 切片（..=整个缓冲区）
            pipeline.vertex_buffer.slice(..),
        );
        render_pass.draw(
            // 顶点范围：从 0 到 vertex_count-1
            0..pipeline.vertex_count,
            // 实例范围：0..1 = 1 个实例（非实例化渲染）
            0..1,
        );
    }
}

// =========================================================================
// Scene — 场景：管理旋转状态
// =========================================================================
//
// 为什么持有 start: Instant？
//   start 记录了场景创建的起始时间。
//   通过 Instant::now() - start 得到已流逝时间，
//   再乘以旋转速度得到旋转角度。
//   这使旋转角度随时间线性增长，产生持续的旋转动画效果。
//
// 旋转效果：陀螺仪进动
//   自转轴倾斜偏离 Y 轴，绕自身高速旋转（3.0 rad/s），
//   同时自转轴又绕全局 Y 轴慢速进动（0.6 rad/s），
//   自转轴在空间中扫描出一个球面轨迹。
//
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

    // ── rotation_matrix() — 计算当前帧陀螺仪旋转矩阵 ──
    //
    // 模拟陀螺仪进动效果：
    //   1. 自转轴倾斜偏离 Y 轴约 17°，绕自身高速旋转（3.0 rad/s）
    //   2. 自转轴同时绕全局 Y 轴慢速进动（0.6 rad/s）
    //   3. 自转轴在空间中 360° 扫描整个球面
    //
    fn rotation_matrix(&self) -> Mat4 {
        let t = self.start.elapsed().as_secs_f32();

        // 自转轴初始方向：从 Y 轴倾斜
        let tilt_axis = vec3(0.3, 0.95, 0.0).normalize();

        // 进动：倾斜轴绕 Y 轴旋转
        let precession_angle = t * 0.6;
        let precession = Mat4::from_axis_angle(Vec3::Y, precession_angle);
        let current_axis = precession.transform_vector3(tilt_axis);

        // 自转：绕当前倾斜轴高速旋转
        let spin_angle = t * 3.0;
        Mat4::from_axis_angle(current_axis, spin_angle)
    }
}

// =========================================================================
// impl Program<Message> for Scene — 场景渲染协议
// =========================================================================
//
// Program trait 是 Iced shader 模块的顶层接口，
// 它将 Scene（场景数据）连接到 Iced 的渲染管线。
//
// draw() 方法：
//   返回 PyramidPrimitive，包含当前帧的旋转矩阵。
//   Iced 框架内部会调用 Primitive::prepare() → draw()(返回false) → render()
//   来完成实际渲染。prepare() 上传 Uniform + 创建深度纹理，
//   render() 自建带深度附件的离屏 RenderPass 完成绘制。
//
// 三个关联类型：
//   type State:  内部状态（本例为 ()，无状态）
//   type Primitive: 返回的图元类型（PyramidPrimitive）
//
impl<Message> Program<Message> for Scene {
    type State = ();
    type Primitive = PyramidPrimitive;

    // draw — 返回当前帧的渲染图元
    // Iced 框架每帧调用此方法获取 PyramidPrimitive，后续由 prepare/render 完成 GPU 渲染
    fn draw(
        &self,
        // 内部状态（本示例使用 ()，无状态）
        _state: &Self::State,
        // 鼠标光标位置（本示例未使用）
        _cursor: iced::advanced::mouse::Cursor,
        // 控件边界矩形（本示例未使用，prepare 阶段使用 bounds）
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        println!(">>> Program::draw 被调用");
        PyramidPrimitive {
            rotation_matrix: self.rotation_matrix(),
        }
    }
}

// =========================================================================
// App — Iced 应用程序的顶层结构
// =========================================================================
#[derive(Debug)]
struct App {
    scene: Scene,
}

// ── Message 枚举：应用的消息类型 ──
//     唯一的消息是 Tick，由定时器触发
#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl App {
    fn title(&self) -> String {
        "金字塔 3D 渲染".into()
    }

    // ── update() — 消息处理 ──
    //     Tick 消息不需要实际更新数据（旋转时间由 Scene.start 自动计算），
    //     但需要返回 Task::none() 来驱动重绘（Iced 在 update 返回后重新调用 view()）。
    // update — 消息处理器
    // 框架每收到一个 Message 就调用此方法，返回 Task 驱动后续副效应（本例无副效应）
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Tick 事件：无需状态更新，返回空 Task 驱动重绘
            Message::Tick => Task::none(),
        }
    }

    // ── view() — 构建 UI ──
    //
    // iced::widget::shader() 函数：
    //   这是 Iced 提供的自定义着色器 Widget 构造函数。
    //   它接收一个实现了 Program trait 的对象（&self.scene），
    //   并创建一个 Iced Widget，Iced 框架会：
    //     1. 测量 Widget 的空间大小
    //     2. 每帧调用 Program::draw() → 获取 Primitive
    //     3. 调用 Primitive::prepare() → 上传 Uniform + 创建深度纹理
    //     4. 调用 Primitive::draw() → 返回 false，触发 render()
    //     5. 调用 Primitive::render() → 离屏渲染带深度测试的绘制
    //
    // view — 构建 UI 元素树
    // 返回 Element 描述整个界面布局，Iced 框架据此计算布局并驱动绘制
    fn view(&self) -> Element<'_, Message> {
        // 创建自定义着色器 Widget，绑定 Scene 作为 Program 实现
        iced::widget::shader(&self.scene)
            // 宽度占满父容器
            .width(Fill)
            // 高度占满父容器
            .height(Fill)
            // 转换为 Element<'_, Message> 类型
            .into()
    }

    // ── subscription() — 订阅定时器 ──
    //
    // iced::time::every(16ms) 的含义：
    //   16ms ≈ 1000ms / 60 ≈ 60fps（每秒 60 帧）
    //   这是一个标准的刷新率，与大多数显示器的 60Hz 刷新率匹配。
    //   每 16ms 触发一次 Tick 消息 → update() 返回 Task::none()
    //   → Iced 重新调用 view() → 触发重绘 → 产生连续旋转动画。
    //
    // subscription — 订阅 Iced 框架事件源
    // 返回 Subscription 枚举，框架每 16ms 自动发射 Tick 消息驱动重绘
    fn subscription(&self) -> iced::Subscription<Message> {
        // 创建定时器：16ms ≈ 60fps
        iced::time::every(std::time::Duration::from_millis(16))
            // 将定时器事件映射为 Message::Tick
            .map(|_| Message::Tick)
    }
}

// =========================================================================
// main() — 应用程序入口
// =========================================================================
//
// iced::application() 接收三个参数：
//   1. 初始化闭包: || App { scene: Scene::new() }  → 创建应用状态
//   2. update 函数: App::update                     → 消息处理
//   3. view 函数:   App::view                      → UI 构建
//
// 链式调用：
//   .title()       → 设置窗口标题
//   .window()      → 设置窗口大小 800×600
//   .subscription() → 注册定时器订阅（驱动动画帧）
//   .run()         → 启动事件循环
//
pub fn main() -> iced::Result {
    // 应用启动入口，接收初始化闭包、update 函数、view 函数
    iced::application(
        // 初始化闭包：创建 App 实例并初始化 Scene
        || App { scene: Scene::new() },
        // update 函数引用：处理 Message 消息，驱动状态变更
        App::update,
        // view 函数引用：构建 UI 元素树，描述界面布局
        App::view,
    )
        // 设置窗口标题
        .title(App::title)
        // 窗口配置：大小 800×600，其余默认
        .window(window::Settings {
            size: iced::Size::new(800.0, 600.0),
            ..window::Settings::default()
        })
        // 注册定时器订阅，驱动动画帧
        .subscription(App::subscription)
        // 启动 Iced 事件循环
        .run()
}
