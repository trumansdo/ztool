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

// ========== GPU端的立方体数据格式 ==========
// 这个结构体会直接作为字节数据上传到GPU的实例缓冲区
// bytemuck::Pod: 表明数据是"纯旧数据"(Plain Old Data)，可以直接安全序列化
// bytemuck::Zeroable: 表明可以用零值安全初始化
// #[repr(C)]: 使用C语言内存布局，保证与GPU期望的内存布局一致
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct PyramidRaw {
    // 4x4变换矩阵（缩放+旋转+平移的组合，用于顶点位置变换）
    transformation: glam::Mat4,
    // 3x3法线矩阵（只含旋转，用于法线方向变换，不含平移和缩放）
    // normal: glam::Mat3,
    // 填充字段，确保内存对齐（glam::Mat3实际占12个f32，但GPU按16字节对齐）
    // _padding: [f32; 3],
}

// Raw的GPU布局定义
impl PyramidRaw {
    // 顶点属性数组：定义实例缓冲区中每个字段如何映射到WGSL着色器
    // 着色器中@location(4-10)对应这些属性
    // vertex_attr_array宏: 便捷创建顶点属性描述数组
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        // ---- 4x4变换矩阵（4个vec4，占据location 4-7） ----
        // 矩阵无法直接在wgsl中接收，必须每个向量接收
        4 => Float32x4,                        // 变换矩阵第1行 (location 4)
        5 => Float32x4,                        // 变换矩阵第2行 (location 5)
        6 => Float32x4,                        // 变换矩阵第3行 (location 6)
        7 => Float32x4,                        // 变换矩阵第4行 (location 7)
        // ---- 3x3法线矩阵（3个vec3，占据location 8-10） ----
        // 8 => Float32x3,                        // 法线矩阵第1行 (location 8)
        // 9 => Float32x3,                        // 法线矩阵第2行 (location 9)
        // 10 => Float32x3,                       // 法线矩阵第3行 (location 10)
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
                // 缩放：统一缩放为cube.size
                glam::vec3(pyramid_shape.size, pyramid_shape.size, pyramid_shape.size),
                // 旋转：当前的四元数旋转
                pyramid_shape.rotation,
                // 平移：在3D空间中的位置
                pyramid_shape.position,
            ),
        }
    }

    pub fn vertices() -> [Vertex; 18] {
        let base_y = -0.5f32;
        let peak_y = 0.5f32;
        let v0 = vec3(0.5, base_y, 0.0);
        let v1 = vec3(0.0, base_y, -0.5);
        let v2 = vec3(-0.5, base_y, 0.0);
        let v3 = vec3(0.0, base_y, 0.5);
        let peak = vec3(0.0, peak_y, 0.0);

        // 五个面的颜色
        let c_front = vec3(0.82, 0.45, 0.38); // 暖珊瑚 - 侧面1
        let c_right = vec3(0.55, 0.60, 0.42); // 鼠尾草绿 - 侧面2
        let c_back = vec3(0.38, 0.50, 0.60); // 雾蓝 - 侧面3
        let c_left = vec3(0.78, 0.62, 0.48); // 暖杏 - 侧面4
        let c_bottom = vec3(0.28, 0.24, 0.20); // 深棕灰 - 底面

        // 渐变：底部顶点深，尖顶浅
        let peak_front = c_front * 1.2;
        let peak_right = c_right * 1.2;
        let peak_back = c_back * 1.2;
        let peak_left = c_left * 1.2;

        [
            // 侧面1: v0→v1→peak (暖珊瑚)
            Vertex {
                pos: v0,
                color: c_front,
            },
            Vertex {
                pos: v1,
                color: c_front,
            },
            Vertex {
                pos: peak,
                color: peak_front,
            },
            // 侧面2: v1→v2→peak (鼠尾草绿)
            Vertex {
                pos: v1,
                color: c_right,
            },
            Vertex {
                pos: v2,
                color: c_right,
            },
            Vertex {
                pos: peak,
                color: peak_right,
            },
            // 侧面3: v2→v3→peak (雾蓝)
            Vertex {
                pos: v2,
                color: c_back,
            },
            Vertex {
                pos: v3,
                color: c_back,
            },
            Vertex {
                pos: peak,
                color: peak_back,
            },
            // 侧面4: v3→v0→peak (暖杏)
            Vertex {
                pos: v3,
                color: c_left,
            },
            Vertex {
                pos: v0,
                color: c_left,
            },
            Vertex {
                pos: peak,
                color: peak_left,
            },
            // 底面: v0→v1→v3 + v1→v2→v3 (深棕灰)
            Vertex {
                pos: v0,
                color: c_bottom,
            },
            Vertex {
                pos: v1,
                color: c_bottom,
            },
            Vertex {
                pos: v3,
                color: c_bottom,
            },
            Vertex {
                pos: v1,
                color: c_bottom,
            },
            Vertex {
                pos: v2,
                color: c_bottom,
            },
            Vertex {
                pos: v3,
                color: c_bottom,
            },
        ]
    }
}
