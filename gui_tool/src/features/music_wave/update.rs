//! 更新逻辑

use super::types::{MusicWave, Pitch, PitchClass, Octave, Msg};
use iced::Task;

pub fn update(state: &mut MusicWave, message: Msg) -> Task<Msg> {
    match message {
        Msg::Tick(now) => {
            if let Some(l) = state.last_tick {
                state.time += now.duration_since(l).as_secs_f64();
            }
            state.last_tick = Some(now);
            Task::none()
        }
        Msg::ToggleCheck(p) => {
            if state.checked.contains(&p) { state.checked.remove(&p); }
            else { state.checked.insert(p); state.solo = Some(p); }
            Task::none()
        }
        Msg::SelectSolo(p) => { state.solo = Some(p); Task::none() }
        Msg::Clear => { state.checked.clear(); Task::none() }
        Msg::SelectChord(ct) => {
            let root = Pitch { class: PitchClass::C, octave: Octave::Four };
            state.solo = Some(root);
            state.checked.clear();
            for n in ct.notes(root) { state.checked.insert(n); }
            Task::none()
        }
        Msg::SoloSpeedChanged(s) => { state.solo_speed = s; Task::none() }
        Msg::SoloZoom(z) => { state.solo_zoom = z; Task::none() }
        Msg::ComboSpeedChanged(s) => { state.combo_speed = s; Task::none() }
        Msg::ComboZoom(z) => { state.combo_zoom = z; Task::none() }
    }
}
