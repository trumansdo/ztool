// ========== 统一变量（Uniforms）模块：CPU 到 GPU 的全局参数 ==========
// Uniforms 是所有立方体共享的全局数据，存储在统一变量缓冲区中
// 在 GPU 着色器中通过 @group(0) @binding(0) 访问
// 内容：相机投影矩阵（VP矩阵）、相机世界坐标、光源颜色
// 这些数据每帧更新一次

// Iced颜色和矩形类型
use iced::{Color, Rectangle};

use crate::features::pyramid_3d::scene::camera::Camera;

// 统一变量结构体（直接映射到 GPU 内存）
// bytemuck::Pod: 可安全序列化为字节
// bytemuck::Zeroable: 可用零值初始化
// #[repr(C)]: C语言内存布局，与GPU着色器中Uniforms结构体内存布局一致
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // 多个derive
#[repr(C)] // C布局（保证GPU兼容性）
pub struct Uniforms {
    // 视图投影矩阵（VP矩阵）：将世界坐标变换到裁剪空间。在着色器中名为projection
    camera_proj: glam::Mat4,
    // 相机在世界空间中的位置（齐次坐标，w=0表示方向向量）
    camera_pos: glam::Vec4,
    // 光源颜色（RGB + A，其中A通道未使用）
    light_color: glam::Vec4,
}

impl Uniforms {
    // Uniforms构造方法
    // 创建统一变量对象
    // camera: 当前相机状态（用于计算视图投影矩阵）
    // bounds: 视口矩形（用于计算宽高比，确保投影正确）
    // light_color: Iced的Color类型（将转换为线性颜色空间 + Vec4格式）
    pub fn new(camera: &Camera, bounds: Rectangle, light_color: Color) -> Self {
        // 计算视图投影矩阵（组合view和projection并适配 wgpu 坐标系统）
        let camera_proj = camera.build_view_proj_matrix(bounds);

        Self {
            camera_proj,                   // 传入着色器用于顶点变换
            camera_pos: camera.position(), // 传入着色器用于方向计算（光照/反射等）
            // into_linear(): 将sRGB颜色空间转换为线性颜色空间
            // 原因：GPU光照计算在线性空间下才物理准确，sRGB是经过gamma校正的非线性空间
            light_color: glam::Vec4::from(light_color.into_linear()),
        }
    }
}
