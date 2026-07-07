use std::time::Instant;

use super::scene;
use iced::Task;

#[derive(Debug, Clone)]
pub enum Msg {
    RotationChanged(f32),
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
            Task::none()
        }
        Msg::RotationChanged(t) => {
            pyramid.scene.angle = t;
            Task::none()
        }
    }
}
