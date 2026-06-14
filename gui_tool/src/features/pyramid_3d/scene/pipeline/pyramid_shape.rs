use super::vertex::Vertex;
use glam::{Vec3, vec3};

#[derive(Clone, Copy, Debug)]
pub struct PyramidShape {
    pub position: glam::Mat3,
    pub rotation: glam::Quat,
    pub scale: f32,
}

impl PyramidShape {
    pub fn new() -> Self {
        PyramidShape {
            position: glam::Mat3::IDENTITY,
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        }
    }
}

impl Default for PyramidShape {
    fn default() -> Self {
        PyramidShape::new()
    }
}

#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct PyramidRaw {
    /// 模型矩阵（Model Matrix）= 缩放 × 旋转 × 平移（4×4 矩阵），
    /// 将立方体顶点从局部坐标变换到世界坐标
    transformation: glam::Mat4,
}

impl PyramidRaw {
    pub fn vertices() -> [Vertex; 16] {
        [
            // 斜面1
            Vertex {
                pos: vec3(0f32, 0f32, 0.5f32),
            },
            Vertex {
                pos: vec3(0.5f32, 0f32, 0f32),
            },
            Vertex {
                pos: vec3(0f32, 0.5f32, 0f32),
            },
            // 斜面2
            Vertex {
                pos: vec3(0.5f32, 0f32, 0f32),
            },
            Vertex {
                pos: vec3(0f32, 0f32, -0.5f32),
            },
            Vertex {
                pos: vec3(0f32, 0.5f32, 0f32),
            },
            // 斜面3
            Vertex {
                pos: vec3(0f32, 0f32, -0.5f32),
            },
            Vertex {
                pos: vec3(-0.5f32, 0f32, 0f32),
            },
            Vertex {
                pos: vec3(0f32, 0.5f32, 0f32),
            },
            // 斜面4
            Vertex {
                pos: vec3(-0.5f32, 0f32, 0f32),
            },
            Vertex {
                pos: vec3(0f32, 0f32, 0.5f32),
            },
            Vertex {
                pos: vec3(0f32, 0.5f32, 0f32),
            },
            // 底面
            Vertex {
                pos: vec3(0.5f32, 0f32, 0.5f32),
            },
            Vertex {
                pos: vec3(0.5f32, 0f32, -0.5f32),
            },
            Vertex {
                pos: vec3(-0.5f32, 0f32, 0.5f32),
            },
            Vertex {
                pos: vec3(-0.5f32, 0f32, 0.5f32),
            },
        ]
    }
}
