use glam::vec3;
use iced::wgpu;

use super::vertex::Vertex;

#[derive(Debug, Copy, Clone)]
pub struct PyramidShape {
    pub rotation: glam::Quat,
    pub position: glam::Vec3,
    pub size: f32,
    pub rotation_dir: f32,
    pub rotation_axis: glam::Vec3,
}

impl Default for PyramidShape {
    fn default() -> Self {
        Self {
            rotation: glam::Quat::IDENTITY,
            position: glam::Vec3::ZERO,
            size: 0.5f32,
            rotation_dir: 1.0,
            rotation_axis: glam::Vec3::Y,
        }
    }
}

// ========== GPU端的实例数据格式 ==========
// 每个实例包含：4x4模型矩阵 + 实例颜色
// 直接作为字节数据上传到GPU的实例缓冲区
// #[repr(C)]: C语言内存布局，保证与GPU着色器中Instance结构体内存布局一致
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct PyramidRaw {
    /// 4x4模型变换矩阵（缩放+旋转+平移），将顶点从模型空间变换到世界空间
    pub transformation: glam::Mat4,
    /// 实例颜色 (r, g, b, a)，在片元着色器中与高度渐变混合
    pub color: glam::Vec4,
}

// Raw的GPU布局定义
impl PyramidRaw {
    /// 顶点属性数组：映射到 WGSL Instance 结构体
    /// @location(2-5): 模型矩阵4行 (Float32x4 × 4)
    /// @location(6):   实例颜色 (Float32x4)
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        2 => Float32x4,  // 模型矩阵第0行 (location 2)
        3 => Float32x4,  // 模型矩阵第1行 (location 3)
        4 => Float32x4,  // 模型矩阵第2行 (location 4)
        5 => Float32x4,  // 模型矩阵第3行 (location 5)
        6 => Float32x4,  // 实例颜色 (location 6)
    ];

    // 返回实例数据的顶点缓冲区布局描述
    // 指定步进模式为Instance（每个实例取下一组数据）
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            // array_stride: 从一个实例数据到下一个实例数据的字节偏移量
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            // step_mode: Instance模式 = 每绘制一个实例，自动前进到下一个Raw结构体
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS, // 上面定义的属性数组
        }
    }
}

impl PyramidRaw {
    pub fn from_shape(pyramid_shape: &PyramidShape) -> PyramidRaw {
        Self {
            transformation: glam::Mat4::from_scale_rotation_translation(
                glam::vec3(pyramid_shape.size, pyramid_shape.size, pyramid_shape.size),
                pyramid_shape.rotation,
                pyramid_shape.position,
            ),
            // 默认金色半透明，后续可从 PyramidShape 扩展颜色配置
            color: glam::vec4(0.9, 0.75, 0.3, 1.0),
        }
    }

    /// 生成金字塔的 18 个顶点（4个侧面 + 底面，每面3顶点，含法线）
    /// 顶点颜色由实例的 color 字段统一控制，此处只提供几何数据
    pub fn vertices() -> [Vertex; 18] {
        let base_y = -0.5f32;
        let peak_y = 0.5f32;
        // 底面正方形四个角（绕Y轴旋转45度，使棱角对齐坐标轴）
        let v0 = vec3(0.5, base_y, 0.0);
        let v1 = vec3(0.0, base_y, -0.5);
        let v2 = vec3(-0.5, base_y, 0.0);
        let v3 = vec3(0.0, base_y, 0.5);
        let peak = vec3(0.0, peak_y, 0.0);

        // 计算各面法线（归一化叉积）
        let n_front = (v1 - v0).cross(peak - v0).normalize();
        let n_right = (v2 - v1).cross(peak - v1).normalize();
        let n_back = (v3 - v2).cross(peak - v2).normalize();
        let n_left = (v0 - v3).cross(peak - v3).normalize();
        let n_bottom = glam::Vec3::NEG_Y; // 底面法线朝下

        [
            // 侧面1: v0→v1→peak
            Vertex { pos: v0, normal: n_front },
            Vertex { pos: v1, normal: n_front },
            Vertex { pos: peak, normal: n_front },
            // 侧面2: v1→v2→peak
            Vertex { pos: v1, normal: n_right },
            Vertex { pos: v2, normal: n_right },
            Vertex { pos: peak, normal: n_right },
            // 侧面3: v2→v3→peak
            Vertex { pos: v2, normal: n_back },
            Vertex { pos: v3, normal: n_back },
            Vertex { pos: peak, normal: n_back },
            // 侧面4: v3→v0→peak
            Vertex { pos: v3, normal: n_left },
            Vertex { pos: v0, normal: n_left },
            Vertex { pos: peak, normal: n_left },
            // 底面: v0→v1→v3 + v1→v2→v3
            Vertex { pos: v0, normal: n_bottom },
            Vertex { pos: v1, normal: n_bottom },
            Vertex { pos: v3, normal: n_bottom },
            Vertex { pos: v1, normal: n_bottom },
            Vertex { pos: v2, normal: n_bottom },
            Vertex { pos: v3, normal: n_bottom },
        ]
    }
}
