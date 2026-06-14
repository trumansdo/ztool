use std::time::Instant;

use iced::Task;

use crate::features::pyramid_3d::PyramidScene;

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Tick(Instant),
}

pub fn update(libs: &mut PyramidScene, msg: Msg) -> Task<Msg> {
    Task::none()
}
