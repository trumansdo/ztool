use iced::wgpu;

/// 顶点数据结构——存储位置和法线
///
/// 颜色不在顶点中存储，而是通过实例数据传入，在着色器中基于高度计算渐变。
/// 法线用于漫反射光照计算。
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    /// 局部空间顶点位置 (x, y, z)
    pub pos: glam::Vec3,
    /// 归一化表面法线，指向面外侧
    pub normal: glam::Vec3,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        // slot 0: 位置坐标，在 WGSL 中对应 @location(0) vec3<f32>
        0 => Float32x3,
        // slot 1: 法线向量，在 WGSL 中对应 @location(1) vec3<f32>
        1 => Float32x3,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
