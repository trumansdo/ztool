use std::f32::consts::PI;
use std::time::Instant;

use glam;
use super::scene;
use iced::Task;

#[derive(Debug, Clone)]
pub enum Msg {
    /// X 轴旋转角度
    RotationXChanged(f32),
    /// Y 轴旋转角度
    RotationYChanged(f32),
    /// Z 轴旋转角度
    RotationZChanged(f32),
    /// 缩放比例
    ScaleChanged(f32),
    /// 每帧时钟信号，驱动连续旋转动画
    Tick(Instant),
}

#[derive(Debug)]
pub struct Pyramid {
    pub start: Instant,
    pub scene: scene::Scene,
}

impl Default for Pyramid {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            scene: scene::Scene::new(),
        }
    }
}

/// 根据三个轴的欧拉角更新金字塔旋转
fn update_rotation(scene: &mut scene::Scene) {
    scene.pyramid_shape.rotation = glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        scene.angle_x,
        scene.angle_y,
        scene.angle_z,
    );
}

pub fn update(pyramid: &mut Pyramid, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::Tick(_t) => {
            // 连续旋转动画：随时间绕Y轴旋转
            let elapsed = pyramid.start.elapsed().as_secs_f32();
            let angle = (elapsed * 0.8) % (2.0 * PI);
            pyramid.scene.angle_y = angle;
            update_rotation(&mut pyramid.scene);
            Task::none()
        }
        Msg::RotationXChanged(angle) => {
            pyramid.scene.angle_x = angle;
            update_rotation(&mut pyramid.scene);
            Task::none()
        }
        Msg::RotationYChanged(angle) => {
            pyramid.scene.angle_y = angle;
            update_rotation(&mut pyramid.scene);
            Task::none()
        }
        Msg::RotationZChanged(angle) => {
            pyramid.scene.angle_z = angle;
            update_rotation(&mut pyramid.scene);
            Task::none()
        }
        Msg::ScaleChanged(scale) => {
            pyramid.scene.scale = scale;
            pyramid.scene.pyramid_shape.size = scale;
            Task::none()
        }
    }
}
