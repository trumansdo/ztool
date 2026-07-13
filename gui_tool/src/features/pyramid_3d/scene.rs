mod camera;
mod pipeline;
mod primitive;

use glam::vec3;
use pipeline::pyramid_shape::PyramidShape;
use primitive::PyramidPrimitive;

use iced::{Color, widget::shader::Program};

use crate::features::pyramid_3d::scene::camera::Camera;

/// 3D 场景——持有所有金字塔实例、相机和光照参数
///
/// 通过实例化渲染，所有金字塔共享同一份顶点缓冲区，每帧上传变换矩阵+颜色即可。
/// 实现 shader::Program trait 后可作为 iced 的 shader widget 嵌入 UI。
#[derive(Debug, Clone)]
pub struct Scene {
    /// 场景全局Y轴旋转角度（UI滑块控制）
    pub angle: f32,
    /// 全局缩放因子（UI滑块控制），叠加上实例自身的 size
    pub scale: f32,
    /// 所有金字塔实例的集合
    pub pyramids: Vec<PyramidShape>,
    /// 3D 相机（位置/朝向/投影）
    pub camera: Camera,
    /// 光照颜色
    pub light_color: Color,
}

impl Scene {
    /// 创建场景，生成多个随机颜色和位置的金字塔以展示 instancing 效果
    pub fn new() -> Self {
        let colors = [
            vec3(0.82, 0.45, 0.38), // 暖珊瑚
            vec3(0.55, 0.60, 0.42), // 鼠尾草绿
            vec3(0.38, 0.50, 0.60), // 雾蓝
            vec3(0.78, 0.62, 0.48), // 暖杏
            vec3(0.28, 0.24, 0.20), // 深棕灰
            vec3(0.90, 0.70, 0.30), // 金黄
            vec3(0.50, 0.30, 0.70), // 紫
            vec3(0.20, 0.70, 0.50), // 翠绿
        ];

        // 在网格上布置 24 个金字塔（4 行 × 6 列），每个带不同颜色和旋转
        let pyramids = (0..24)
            .map(|i| {
                let row = (i / 6) as f32 - 1.5;
                let col = (i % 6) as f32 - 2.5;
                let color = colors[i % colors.len()];
                PyramidShape {
                    rotation: glam::Quat::from_rotation_y(i as f32 * 0.5),
                    position: vec3(col * 0.7, 0.0, row * 0.7),
                    size: 0.25,
                    color,
                }
            })
            .collect();

        Scene {
            angle: 0f32,
            scale: 1.0,
            pyramids,
            camera: Camera::default(),
            light_color: Color::WHITE,
        }
    }
}

impl<Message> Program<Message> for Scene {
    type State = ();
    type Primitive = PyramidPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        PyramidPrimitive::new(
            &self.pyramids,
            &self.camera,
            bounds,
            self.light_color,
            self.angle,
            self.scale,
        )
    }
}
