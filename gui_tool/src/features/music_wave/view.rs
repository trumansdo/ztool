//! 视图

use super::types::{Pitch, PitchClass, Octave, ChordType, MusicWave, Msg};
use super::canvas::{SoloWave, CombinedWave};
use iced::widget::{button, checkbox, column, container, row, scrollable, slider, text, Canvas};
use iced::{Color, Element, Length, Theme};
use iced::border;

pub fn view(state: &MusicWave) -> Element<'_, Msg> {
    let solo_phase = state.solo_phase();
    let combo_phase = state.combo_phase();

    // ============================================================
    // 顶部工具栏：和弦按钮
    // ============================================================
    let chords = [
        ChordType::Major, ChordType::Minor, ChordType::Augmented, ChordType::Diminished,
        ChordType::Dominant7, ChordType::Major7, ChordType::Minor7,
        ChordType::Major9, ChordType::Minor9, ChordType::Sus4, ChordType::Add9,
    ];
    let mut cr = row![].spacing(3).align_y(iced::Alignment::Center);
    for &c in &chords {
        cr = cr.push(
            button(text(c.label()).size(13))
                .style(sbtn(Color::from_rgb8(40,40,65), Color::from_rgb8(70,70,110)))
                .height(Length::Fixed(29.0))
                .on_press(Msg::SelectChord(c)),
        );
    }
    let toolbar = container(
        row![cr, text("根音 C4").size(15).color(Color::from_rgb8(200,180,100))]
            .spacing(12).align_y(iced::Alignment::Center),
    )
    .style(tbar)
    .padding(4)
    .width(Length::Fill);

    // ============================================================
    // 左侧：音阶列表
    // ============================================================
    let mut cc = column![].spacing(6).padding(4);
    for octave in [Octave::Three, Octave::Four, Octave::Five] {
        cc = cc.push(
            text(format!("— 八度 {} —", octave.number()))
                .size(16).color(Color::from_rgb8(110,120,150))
        );
        for chunk in PitchClass::ALL.chunks(3) {
            let mut row_items = row![].spacing(6);
            for pc in chunk {
                let p = Pitch { class: *pc, octave };
                let checked = state.checked.contains(&p);
                let cb = checkbox(checked)
                    .label(pc.label())
                    .text_size(20)
                    .size(24)
                    .spacing(4)
                    .on_toggle(move |_| Msg::ToggleCheck(p));
                row_items = row_items.push(cb);
            }
            cc = cc.push(row_items);
        }
        cc = cc.push(column![].height(6));
    }
    let left = container(column![
        text("音阶").size(18).color(Color::from_rgb8(140,150,170)),
        scrollable(cc).height(Length::Fill).width(Length::Fill),
        button(text("清除").size(16))
            .style(sbtn(Color::from_rgb8(50,25,25), Color::from_rgb8(90,40,40)))
            .height(Length::Fixed(32.0)).width(Length::Fill)
            .on_press(Msg::Clear),
    ].spacing(6))
    .style(pan(Color::from_rgb8(20,20,35)))
    .padding(8)
    .width(Length::Fixed(185.0))
    .height(Length::Fill);

    // ============================================================
    // 右上：单音波形
    // ============================================================
    let solo_speed_row = row![
        text("速度").size(11).color(Color::from_rgb8(140,160,190)),
        text(format!("{:.2}x", state.solo_speed)).size(11).color(Color::from_rgb8(150,160,180)),
        slider(0.01..=5.0, state.solo_speed, Msg::SoloSpeedChanged).step(0.01).width(Length::Fixed(100.0)),
        text("缩放").size(11).color(Color::from_rgb8(140,160,190)),
        text(format!("{:.1}x", state.solo_zoom)).size(11).color(Color::from_rgb8(150,160,180)),
        slider(0.1..=5.0, state.solo_zoom, Msg::SoloZoom).step(0.1).width(Length::Fixed(80.0)),
    ].spacing(4).align_y(iced::Alignment::Center);

    let solo_area = container(column![
        solo_speed_row,
        Canvas::new(SoloWave { phase: solo_phase, pitch: state.solo, zoom: state.solo_zoom })
            .width(Length::Fill).height(Length::Fill),
    ].spacing(2))
    .style(pan(Color::from_rgb8(15,15,30)))
    .padding(4)
    .width(Length::Fill)
    .height(Length::Fill);

    // ============================================================
    // 右下：组合波形
    // ============================================================
    let combo_ctrl = row![
        text("速度").size(11).color(Color::from_rgb8(140,160,190)),
        text(format!("{:.2}x", state.combo_speed)).size(11).color(Color::from_rgb8(150,160,180)),
        slider(0.01..=5.0, state.combo_speed, Msg::ComboSpeedChanged).step(0.01).width(Length::Fixed(100.0)),
        text("缩放").size(11).color(Color::from_rgb8(140,160,190)),
        text(format!("{:.1}x", state.combo_zoom)).size(11).color(Color::from_rgb8(150,160,180)),
        slider(0.1..=10.0, state.combo_zoom, Msg::ComboZoom).step(0.1).width(Length::Fixed(80.0)),
    ].spacing(4).align_y(iced::Alignment::Center);

    let combo_area = container(column![
        combo_ctrl,
        Canvas::new(CombinedWave {
            phase: combo_phase,
            pitches: state.checked_pitches(),
            zoom: state.combo_zoom,
        })
        .width(Length::Fill).height(Length::Fill),
    ].spacing(2))
    .style(pan(Color::from_rgb8(18,18,35)))
    .padding(4)
    .width(Length::Fill)
    .height(Length::Fill);

    // ============================================================
    // 右侧边栏：单音在上 + 组合在下
    // ============================================================
    let right_side = column![
        solo_area,
        combo_area,
    ]
    .spacing(3)
    .width(Length::Fill)
    .height(Length::Fill);

    // ============================================================
    // 主区域
    // ============================================================
    let main = row![left, right_side]
        .spacing(3)
        .width(Length::Fill)
        .height(Length::Fill);

    column![toolbar, main]
        .spacing(3)
        .padding(4)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ============================================================
// 样式
// ============================================================

fn pan(bg: Color) -> impl Fn(&Theme) -> container::Style {
    move |_: &Theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border { color: Color::from_rgb8(40,40,60), width: 1.0, radius: 6.0.into() },
        ..container::Style::default()
    }
}

fn tbar(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(22, 22, 38))),
        border: iced::Border { color: Color::from_rgb8(50,50,70), width: 1.0, radius: 6.0.into() },
        ..container::Style::default()
    }
}

fn sbtn(bg: Color, h: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, s| button::Style {
        background: Some(iced::Background::Color(match s {
            button::Status::Hovered => h,
            _ => bg,
        })),
        border: border::rounded(3.0),
        text_color: Color::from_rgb8(200,210,230),
        ..button::Style::default()
    }
}
