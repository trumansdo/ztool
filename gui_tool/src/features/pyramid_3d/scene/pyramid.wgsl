// ============================================================
// pyramid.wgsl — 金字塔自定义着色器
//
// 核心功能：
// 1. 实例化渲染(instancing)：所有金字塔共享同一份顶点缓冲，
//    每个实例通过 4×4 模型矩阵变换到独立位置
// 2. 高度渐变：基于顶点在局部空间的 y 坐标（-0.5~0.5），
//    片元从底部的暗色平滑过渡到顶部的亮色
// 3. 漫反射光照：简单的环境光 + 定向光，法线由顶点传入
// 4. 无纹理：所有颜色通过计算产生，无需纹理采样
// ============================================================

// ---- uniform 缓冲区（每帧由 CPU 更新） ----
struct Uniforms {
    projection: mat4x4<f32>,  // 视图×投影矩阵（VP）
    camera_pos: vec4<f32>,    // 相机世界位置
    light_color: vec4<f32>,   // 光照颜色
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// ---- 顶点输入（逐顶点） ----
struct Vertex {
    @location(0) position: vec3<f32>,  // 局部空间位置
    @location(1) normal: vec3<f32>,    // 归一化表面法线
}

// ---- 实例输入（逐实例） ----
struct Instance {
    @location(2) matrix_0: vec4<f32>,  // 模型矩阵第0行
    @location(3) matrix_1: vec4<f32>,  // 模型矩阵第1行
    @location(4) matrix_2: vec4<f32>,  // 模型矩阵第2行
    @location(5) matrix_3: vec4<f32>,  // 模型矩阵第3行
    @location(6) color: vec4<f32>,     // 实例颜色 (r,g,b,a)
}

// ---- 顶点→片元插值数据 ----
struct Output {
    @builtin(position) clip_pos: vec4<f32>,  // 裁剪空间位置
    @location(0) height: f32,                // 局部高度 (-0.5~0.5)
    @location(1) world_normal: vec3<f32>,    // 世界空间法线
    @location(2) world_pos: vec3<f32>,       // 世界空间位置
    @location(3) instance_color: vec4<f32>,  // 实例颜色（插值）
}

// ---- 光照常量 ----
// 定向光方向（世界空间）
const LIGHT_DIR: vec3<f32> = vec3<f32>(0.5, 1.0, 0.3);
// 环境光强度（0~1），越大整体亮度越高
const AMBIENT_STRENGTH: f32 = 0.25;

// ===== 顶点着色器 =====
// 将顶点从局部空间变换到裁剪空间，同时传递高度和法线
@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> Output {
    // 从4个vec4重组模型矩阵
    let model_matrix = mat4x4<f32>(
        instance.matrix_0,
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3,
    );

    // 顶点变换到世界空间
    let world_pos = model_matrix * vec4<f32>(vertex.position, 1.0);

    // 法线变换：取模型矩阵前 3 行的 xyz 分量组 3×3 矩阵，变换法线到世界空间
    // 注意：对于非均匀缩放，应使用逆转置矩阵。
    // 本场景中所有实例使用均匀缩放，直接用 mat3x3 即可
    let world_normal = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz,
    ) * vertex.normal;

    var out: Output;
    out.clip_pos = uniforms.projection * world_pos;
    out.height = vertex.position.y;                     // -0.5 ~ 0.5
    out.world_normal = world_normal;
    out.world_pos = world_pos.xyz;
    out.instance_color = instance.color;

    return out;
}

// ===== 片元着色器 =====
// 计算高度渐变 + 漫反射光照
@fragment
fn fs_main(in: Output) -> @location(0) vec4<f32> {
    // ---- 高度渐变 ----
    // 将高度 [-0.5, 0.5] 映射到 [0.0, 1.0]
    // t=0: 底部最暗（0.4倍颜色）
    // t=1: 顶部最亮（1.3倍颜色）
    let t = in.height + 0.5;
    let gradient_color = in.instance_color * mix(0.4, 1.3, t);

    // ---- 漫反射光照 ----
    let normal = normalize(in.world_normal);
    let light_dir = normalize(LIGHT_DIR);
    let diffuse = max(dot(normal, light_dir), 0.0);
    let lighting = AMBIENT_STRENGTH + (1.0 - AMBIENT_STRENGTH) * diffuse;

    // ---- 最终输出 ----
    // 颜色 = 渐变颜色 × 光照系数，Alpha = 1.0（不透明）
    return vec4<f32>(gradient_color.xyz * lighting, in.instance_color.w);
}
