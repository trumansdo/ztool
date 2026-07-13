use std::time::Instant;

use super::scene;
use iced::Task;

#[derive(Debug, Clone)]
pub enum Msg {
    RotationChanged(f32),
    ScaleChanged(f32),
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

pub fn update(pyramid: &mut Pyramid, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::Tick(t) => {
            // 每帧更新所有金字塔的旋转，展示 instancing 实行动画的能力
            let elapsed = (t - pyramid.start).as_secs_f32();
            for (i, p) in pyramid.scene.pyramids.iter_mut().enumerate() {
                // 每个金字塔以不同速度绕 Y 轴旋转，产生波浪效果
                let speed = 0.5 + (i as f32 * 0.07);
                p.rotation = glam::Quat::from_rotation_y(elapsed * speed);
            }
            Task::none()
        }
        Msg::RotationChanged(t) => {
            pyramid.scene.angle = t;
            Task::none()
        }
        Msg::ScaleChanged(s) => {
            pyramid.scene.scale = (s * 10.0).round() / 10.0; // 保留一位小数
            Task::none()
        }
    }
}
