use glam::vec3;
use iced::wgpu;

use super::vertex::Vertex;

/// CPU端的金字塔实体——持有线性变换参数和颜色
///
/// 每个PyramidShape对应场景中的一个金字塔实例，通过实例化渲染(instancing)共享同一份顶点缓冲。
/// 位置/旋转/大小构成模型矩阵 M，颜色通过实例数据传入着色器，基于顶点高度生成渐变。
#[derive(Debug, Copy, Clone)]
pub struct PyramidShape {
    pub rotation: glam::Quat,
    pub position: glam::Vec3,
    pub size: f32,
    /// 实例的基础颜色（RGB），着色器根据高度对此值做渐变
    pub color: glam::Vec3,
}

impl Default for PyramidShape {
    fn default() -> Self {
        Self {
            rotation: glam::Quat::IDENTITY,
            position: glam::Vec3::ZERO,
            size: 0.5f32,
            color: vec3(0.5, 0.5, 0.5),
        }
    }
}

// ========== GPU端的实例数据格式 ==========
// 每个金字塔实例一份，直接作为字节上传到实例缓冲区
// bytemuck traits 保证安全的零拷贝序列化
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct PyramidRaw {
    /// 模型矩阵（缩放×旋转×平移），用于将顶点从局部空间变换到世界空间
    transformation: glam::Mat4,
    /// 实例的基础颜色(RGB) + 填充(w)
    color: glam::Vec4,
}

impl PyramidRaw {
    /// 实例数据的顶点属性布局：4×4矩阵（4个vec4）+ 颜色（1个vec4）
    ///
    /// location 2-5: 模型矩阵的4行（每行一个vec4）
    /// location 6: 颜色（vec4）
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        2 => Float32x4,  // 模型矩阵第1行
        3 => Float32x4,  // 模型矩阵第2行
        4 => Float32x4,  // 模型矩阵第3行
        5 => Float32x4,  // 模型矩阵第4行
        6 => Float32x4,  // 实例颜色 (r,g,b,w)
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl PyramidRaw {
    /// 将CPU端PyramidShape转换为GPU端实例数据
    ///
    /// `world_angle`: 全局Y轴旋转角（UI滑块控制），在实例自身变换之上再旋转整个场景
    /// `world_scale`: 全局缩放因子，叠加上实例自身 size
    pub fn from_shape(pyramid_shape: &PyramidShape, world_angle: f32, world_scale: f32) -> PyramidRaw {
        let model = glam::Mat4::from_scale_rotation_translation(
            glam::vec3(
                pyramid_shape.size * world_scale,
                pyramid_shape.size * world_scale,
                pyramid_shape.size * world_scale,
            ),
            pyramid_shape.rotation,
            pyramid_shape.position,
        );
        // 在模型矩阵外再叠一层全局Y旋转——拖动angle条时所有金字塔绕Y轴一起转
        let transformation = glam::Mat4::from_rotation_y(world_angle) * model;
        let color = glam::Vec4::from((
            pyramid_shape.color,
            1.0f32, // w = 1.0，着色器中可用于透明度
        ));
        Self {
            transformation,
            color,
        }
    }

    /// 返回金字塔的18个顶点（4个三角侧面 × 3 + 底面四边形 × 6 = 18）
    ///
    /// 每个面使用独立的顶点副本（不共享顶点），以便每个面有独立的法线方向。
    /// 这是"平直着色"(flat shading)的经典做法，每个三角形的三个顶点使用相同的法线。
    ///
    /// 法线计算方式：每个三角面的两条边向量叉积，归一化后得到法线。
    /// 在顶点着色器中，我们通过模型矩阵的3×3部分变换法线到世界空间。
    pub fn vertices() -> [Vertex; 18] {
        let base_y = -0.5f32;
        let peak_y = 0.5f32;

        // 底面的4个顶点 + 顶点
        let v0 = vec3(0.5, base_y, 0.0);
        let v1 = vec3(0.0, base_y, -0.5);
        let v2 = vec3(-0.5, base_y, 0.0);
        let v3 = vec3(0.0, base_y, 0.5);
        let peak = vec3(0.0, peak_y, 0.0);

        // 每个三角面的法线（右手定则，逆时针方向）
        // 面1 (v0→v1→peak): 前侧
        let n0 = {
            let e1 = v1 - v0;
            let e2 = peak - v0;
            e1.cross(e2).normalize()
        };
        // 面2 (v1→v2→peak): 右侧
        let n1 = {
            let e1 = v2 - v1;
            let e2 = peak - v1;
            e1.cross(e2).normalize()
        };
        // 面3 (v2→v3→peak): 后侧
        let n2 = {
            let e1 = v3 - v2;
            let e2 = peak - v2;
            e1.cross(e2).normalize()
        };
        // 面4 (v3→v0→peak): 左侧
        let n3 = {
            let e1 = v0 - v3;
            let e2 = peak - v3;
            e1.cross(e2).normalize()
        };
        // 底面 (v0→v1→v3 + v1→v2→v3): 朝下
        let n4 = vec3(0.0, -1.0, 0.0);

        [
            // === 侧面1 (前侧): v0→v1→peak ===
            Vertex { pos: v0, normal: n0 },
            Vertex { pos: v1, normal: n0 },
            Vertex { pos: peak, normal: n0 },
            // === 侧面2 (右侧): v1→v2→peak ===
            Vertex { pos: v1, normal: n1 },
            Vertex { pos: v2, normal: n1 },
            Vertex { pos: peak, normal: n1 },
            // === 侧面3 (后侧): v2→v3→peak ===
            Vertex { pos: v2, normal: n2 },
            Vertex { pos: v3, normal: n2 },
            Vertex { pos: peak, normal: n2 },
            // === 侧面4 (左侧): v3→v0→peak ===
            Vertex { pos: v3, normal: n3 },
            Vertex { pos: v0, normal: n3 },
            Vertex { pos: peak, normal: n3 },
            // === 底面: v0→v1→v3 + v1→v2→v3 (两个三角形) ===
            Vertex { pos: v0, normal: n4 },
            Vertex { pos: v1, normal: n4 },
            Vertex { pos: v3, normal: n4 },
            Vertex { pos: v1, normal: n4 },
            Vertex { pos: v2, normal: n4 },
            Vertex { pos: v3, normal: n4 },
        ]
    }
}
