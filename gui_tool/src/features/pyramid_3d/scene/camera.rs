// ========== 相机模块：3D视图与投影矩阵计算 ==========
// 相机定义了从3D世界空间到2D屏幕空间的变换方式
// 核心概念：
//   - 视图矩阵(view)：将世界坐标转换为相机视角坐标（从相机看世界）
//   - 投影矩阵(proj)：将3D相机空间投影到2D裁剪空间（透视效果：近大远小）
//   - 两者相乘 = 视图投影矩阵，GPU每帧用它把每个顶点从世界坐标变换到屏幕坐标

use glam::{mat4, vec3, vec4}; // glam数学库的函数宏（方便创建矩阵和向量）
use iced::Rectangle; // 矩形类型（用于获取宽高比）

// 相机结构体（定义观察者的位置和投影参数）
#[derive(Debug, Copy, Clone)] // Copy + Clone：轻量数据，可快速复制
pub struct Camera {
    // 相机位置（观察者眼睛在哪）
    eye: glam::Vec3,
    // 注视目标点（相机看向哪里）
    target: glam::Vec3,
    // 向上方向矢量（定义相机的"上"是什么方向，通常为Y轴正方向）
    up: glam::Vec3,
    // 垂直视场角（Field of View），单位：度。越大视野越宽
    fov_y: f32,
    // 近裁剪面：离相机多近的物体不再渲染
    near: f32,
    // 远裁剪面：离相机多远的物体不再渲染
    far: f32,
}

// 默认相机配置
impl Default for Camera {
    fn default() -> Self {
        Self {
            // 相机在(0, 2, 3)：稍微抬高的前方位置
            eye: vec3(0.0, 2.0, 3.0),
            // 看向原点(0,0,0)：场景中心
            target: glam::Vec3::ZERO,
            // Y轴向上（标准右手坐标系）
            up: glam::Vec3::Y,
            // 45度视场角（中等广角）
            fov_y: 45.0,
            // 近裁剪面0.1单位（非常近）
            near: 0.1,
            // 远裁剪面100单位（足够远）
            far: 100.0,
        }
    }
}

// OpenGL 到 wgpu 的坐标系转换矩阵
// ========== 重要概念：为什么需要这个矩阵？ ==========
// OpenGL的裁剪空间（clip space）Z轴范围是[-1, 1]（NDC坐标从左到右）
// wgpu/DirectX/Metal的裁剪空间Z轴范围是[0, 1]
// 这个矩阵将OpenGL风格矩阵的结果映射到wgpu可接受的范围
pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = mat4(
    // 4x4变换矩阵
    // 第1列：X轴不变
    vec4(1.0, 0.0, 0.0, 0.0),
    // 第2列：Y轴不变
    vec4(0.0, 1.0, 0.0, 0.0),
    // 第3列：Z轴缩放0.5（[-1,1] → [-0.5,0.5]）
    vec4(0.0, 0.0, 0.5, 0.0),
    // 第4列：Z轴平移0.5（[-0.5,0.5] → [0,1]）
    vec4(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    // 相机的方法实现
    // 构建视图投影矩阵（View-Projection Matrix，简称VP矩阵）
    // VP矩阵 = openGL_to_wgpu * projection * view
    // 作用：将世界空间中的点变换到wgpu裁剪空间
    pub fn build_view_proj_matrix(&self, bounds: Rectangle) -> glam::Mat4 {
        let aspect_ratio = bounds.width / bounds.height; // 宽高比 = 宽度 / 高度（避免画面拉伸）
        // 视图矩阵：从世界空间变换到相机空间
        // look_at_rh: 右手坐标系的"看向"矩阵（rh = right-hand，右手坐标系）
        // 参数：相机位置、目标点、向上方向
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        // 投影矩阵：从相机空间投影到裁剪空间（透视投影，近大远小）
        // perspective_rh: 右手坐标系透视投影
        // 参数：垂直视场角、宽高比、近平面、远平面
        let proj = glam::Mat4::perspective_rh(self.fov_y, aspect_ratio, self.near, self.far);

        OPENGL_TO_WGPU_MATRIX * proj * view // 链式相乘：先view再proj再坐标系适配
    }

    // 获取相机位置（以齐次坐标vec4形式，w=0表示方向/位置向量）
    // w=0.0表示这是一个方向向量（不受平移影响），用于着色器中的方向计算
    pub fn position(&self) -> glam::Vec4 {
        // 返回4D向量
        glam::Vec4::from((self.eye, 0.0)) // 将3D位置扩展为4D（w=0.0，方向语义）
    }
}
