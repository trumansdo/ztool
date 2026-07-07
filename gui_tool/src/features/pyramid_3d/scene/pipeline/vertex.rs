use iced::wgpu;

/// 暂时忽视法线，切线，纹理
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    // 顶点在模型空间(局部坐标系)中的三维坐标位置 (x, y, z)
    pub pos: glam::Vec3,
    pub color: glam::Vec3,
    // 表面法线方向，归一化向量，指向面外侧，用于漫反射/镜面反射等光照计算
    // pub normal: glam::Vec3,
    // // 切线向量(Tangent)，沿纹理UV的U方向，与法线、副法线构成切线空间(TBN矩阵)
    // // 用于法线贴图(Normal Mapping): 将切线空间法线变换到世界空间以正确计算光照
    // pub tangent: glam::Vec3,
    // // 纹理坐标(UV)，将2D纹理映射到模型表面
    // // (0,0) 表示纹理左下角，(1,1) 表示纹理右上角
    // pub uv: glam::Vec2,
}

impl Vertex {
    // 顶点属性数组，定义 GPU 如何解释顶点缓冲区中的每个字段
    // 使用 wgpu::vertex_attr_array! 宏生成 [wgpu::VertexAttribute; 4]
    // 每个属性包含: shader_location (slot编号), format (数据格式), offset (字节偏移，由宏自动计算)
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        // slot 0 = 位置坐标: 三个 f32，在 WGSL 中对应 @location(0) vec3<f32>
        0 => Float32x3,
        1 => Float32x3,
        // // slot 1 = 法线向量: 三个 f32，在 WGSL 中对应 @location(1) vec3<f32>
        // 1 => Float32x3,
        // // slot 2 = 切线向量: 三个 f32，在 WGSL 中对应 @location(2) vec3<f32>
        // 2 => Float32x3,
        // // slot 3 = 纹理坐标: 两个 f32，在 WGSL 中对应 @location(3) vec2<f32>
        // 3 => Float32x2,
    ];

    // 返回顶点缓冲区的布局描述，告诉 GPU 如何读取顶点数据
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            // array_stride: 从一个顶点数据的起始位置到下一个顶点数据的起始位置之间的字节跨度
            // size_of::<Vertex>() = 11 * 4 = 44 字节 (3+3+3+2 个 f32)
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            // step_mode: Vertex 模式表示每次着色器调用步进一个顶点
            //   即第 i 次顶点着色器调用读取第 i 个顶点的数据
            //   区别于 Instance 模式: 第 i 个实例的所有顶点读取同一份实例数据
            step_mode: wgpu::VertexStepMode::Vertex,
            // 指向顶点属性数组的引用，数组长度=4，对应 pos/normal/tangent/uv
            attributes: &Self::ATTRIBS,
        }
    }
}
