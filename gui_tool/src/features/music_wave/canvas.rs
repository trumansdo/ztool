//! Canvas 绘制：单音波形、组合波形、键盘指示条

use super::types::{Pitch, PitchClass, Octave, AMPLITUDE, harmonic_base};
use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{Color, Rectangle, Theme, Point};
use std::f64::consts::TAU;

// ============================================================
// 常量
// ============================================================

const KEYBOARD_H: f64 = 30.0;       // 键盘高度
const WHITE_PER_OCTAVE: usize = 7;  // 每八度白键数
const OCTAVE_COUNT: usize = 3;      // 显示八度数

// ============================================================
// 辅助函数
// ============================================================

/// 波形绘制：通用路径生成器
fn make_wave_path(w: f64, mid: f64, n: usize, f: impl Fn(f64) -> f64) -> Path {
    Path::new(|b| {
        let mut first = true;
        for i in 0..=n {
            let x = i as f64 / n as f64 * w;
            let val = f(x);
            let py = (mid - val * mid) as f32;
            if first { b.move_to(Point::new(x as f32, py)); first = false; }
            else { b.line_to(Point::new(x as f32, py)); }
        }
    })
}

/// 键盘指示条：在 canvas 底部绘制钢琴键，高亮勾选音
fn draw_keyboard(f: &mut Frame, w: f64, y_base: f64, checked: &[Pitch]) {
    let wk_w = w / (OCTAVE_COUNT * WHITE_PER_OCTAVE) as f64;  // 白键宽
    let bk_w = wk_w * 0.55;                                    // 黑键宽
    let bk_h = KEYBOARD_H * 0.62;                              // 黑键高

    // 背景
    f.fill_rectangle(
        Point::new(0.0, y_base as f32),
        iced::Size::new(w as f32, KEYBOARD_H as f32),
        Color::from_rgb8(10, 10, 18),
    );

    // ── 白键 ──
    let white_order = [
        PitchClass::C, PitchClass::D, PitchClass::E, PitchClass::F,
        PitchClass::G, PitchClass::A, PitchClass::B,
    ];
    for (oi, octave) in [Octave::Three, Octave::Four, Octave::Five].iter().enumerate() {
        for (wi, &pc) in white_order.iter().enumerate() {
            let p = Pitch { class: pc, octave: *octave };
            let idx = (oi * WHITE_PER_OCTAVE + wi) as f64;
            let x = idx * wk_w;
            let is_on = checked.contains(&p);
            f.fill_rectangle(
                Point::new(x as f32, y_base as f32),
                iced::Size::new(wk_w as f32, KEYBOARD_H as f32),
                if is_on { p.color() } else { Color::from_rgb8(30, 30, 45) },
            );
            // 白键边框
            f.stroke(&Path::rectangle(
                Point::new(x as f32, y_base as f32),
                iced::Size::new(wk_w as f32, KEYBOARD_H as f32),
            ), Stroke::default().with_color(Color::from_rgba8(50, 50, 70, 0.4)).with_width(0.5));
        }
    }

    // ── 黑键 ──
    let black_positions: [(PitchClass, f64); 5] = [
        (PitchClass::Cs, 0.5),   // C# after C
        (PitchClass::Ds, 1.5),   // D# after D
        (PitchClass::Fs, 3.5),   // F# after F
        (PitchClass::Gs, 4.5),   // G# after G
        (PitchClass::As, 5.5),   // A# after A
    ];
    for (oi, octave) in [Octave::Three, Octave::Four, Octave::Five].iter().enumerate() {
        for &(pc, offset) in &black_positions {
            let p = Pitch { class: pc, octave: *octave };
            let idx = oi as f64 * WHITE_PER_OCTAVE as f64 + offset;
            let x = idx * wk_w - bk_w / 2.0;
            let is_on = checked.contains(&p);
            f.fill_rectangle(
                Point::new(x as f32, y_base as f32),
                iced::Size::new(bk_w as f32, bk_h as f32),
                if is_on { p.color() } else { Color::from_rgb8(20, 20, 32) },
            );
        }
    }
}

/// 虚线绘制：在画布上画一条水平虚线
fn draw_dashed_line(f: &mut Frame, x1: f32, x2: f32, y: f32, color: Color, width: f32, dash_len: f32, gap_len: f32) {
    let mut px = x1;
    while px < x2 {
        let ex = (px + dash_len).min(x2);
        f.stroke(
            &Path::line(Point::new(px, y), Point::new(ex, y)),
            Stroke::default().with_color(color).with_width(width),
        );
        px = ex + gap_len;
    }
}

// ============================================================
// 单音 Canvas
// ============================================================

#[derive(Debug, Clone)]
pub struct SoloWave {
    pub phase: f64,
    pub pitch: Option<Pitch>,
    pub zoom: f32,
}

impl<Message> Program<Message> for SoloWave {
    type State = ();

    fn draw(&self, _: &(), r: &iced::Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(r, b.size());
        let (w, h) = (b.width as f64, b.height as f64);
        let mid = h / 2.0;

        // 背景
        f.fill_rectangle(Point::ORIGIN, b.size(), Color::from_rgb8(15, 15, 30));

        // 中线（X轴）
        f.stroke(
            &Path::line(Point::new(0.0, mid as f32), Point::new(w as f32, mid as f32)),
            Stroke::default().with_color(Color::from_rgba8(60, 60, 80, 0.5)).with_width(1.0),
        );
        // Y轴
        f.stroke(
            &Path::line(Point::new(w as f32 / 2.0, 0.0), Point::new(w as f32 / 2.0, h as f32)),
            Stroke::default().with_color(Color::from_rgba8(60, 60, 80, 0.5)).with_width(1.0),
        );

        if let Some(p) = self.pitch {
            let freq = p.frequency();
            let cycles = 2.0 + 4.0 * (self.zoom as f64 / 5.0).clamp(0.0, 1.0);
            let t_scale = cycles / (freq * w);
            let t_offset = -cycles / (2.0 * freq);

            let path = make_wave_path(w, mid, 600, |x| {
                let t = x * t_scale + t_offset;
                (TAU * freq * t + self.phase).sin() * AMPLITUDE
            });
            f.stroke(&path, Stroke::default().with_color(p.color()).with_width(2.0));

            // 标签
            f.fill_text(Text {
                content: format!("{}  {:.1} Hz", p.label(), freq),
                position: Point::new(8.0, 14.0),
                color: Color::from_rgb8(180, 200, 220),
                size: 12.0.into(),
                ..Text::default()
            });
        }

        vec![f.into_geometry()]
    }
}

// ============================================================
// 组合 Canvas
// ============================================================

#[derive(Debug, Clone)]
pub struct CombinedWave {
    pub phase: f64,
    pub pitches: Vec<Pitch>,
    pub zoom: f32,
}

impl<Message> Program<Message> for CombinedWave {
    type State = ();

    fn draw(&self, _: &(), r: &iced::Renderer, _: &Theme, b: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let mut f = Frame::new(r, b.size());
        let (w, h_all) = (b.width as f64, b.height as f64);
        let h_wave = h_all - KEYBOARD_H;  // 波形区域高度
        let mid = h_wave / 2.0;

        // 背景
        f.fill_rectangle(Point::ORIGIN, b.size(), Color::from_rgb8(18, 18, 35));

        // 中线（X轴）
        f.stroke(
            &Path::line(Point::new(0.0, mid as f32), Point::new(w as f32, mid as f32)),
            Stroke::default().with_color(Color::from_rgba8(80, 80, 100, 0.6)).with_width(1.5),
        );
        // Y轴
        f.stroke(
            &Path::line(Point::new(w as f32 / 2.0, 0.0), Point::new(w as f32 / 2.0, h_wave as f32)),
            Stroke::default().with_color(Color::from_rgba8(80, 80, 100, 0.6)).with_width(1.5),
        );

        if !self.pitches.is_empty() {
            let base_freq = harmonic_base(&self.pitches);
            let n = self.pitches.len() as f64;

            // 动态周期数：zoom 0.1→2周期, zoom 10→6周期
            let cycles = 2.0 + 4.0 * (self.zoom as f64 / 10.0).clamp(0.0, 1.0);
            let t_scale = cycles / (base_freq * w);
            let t_offset = -cycles / (2.0 * base_freq);

            // 各音单独波形（透明度按音高渐变）
            for (pi, p) in self.pitches.iter().enumerate() {
                let c = p.color();
                let alpha = 0.15 + 0.20 * (pi as f64 / (n - 1.0).max(1.0));  // 0.15 ~ 0.35
                let amp = AMPLITUDE * 0.5;  // 单独波形小一点
                let path = make_wave_path(w, mid, 600, |x| {
                    let t = x * t_scale + t_offset;
                    (TAU * p.frequency() * t + self.phase).sin() * amp
                });
                f.stroke(
                    &path,
                    Stroke::default()
                        .with_color(Color::from_rgba8(
                            (c.r * 255.) as u8, (c.g * 255.) as u8, (c.b * 255.) as u8,
                            alpha as f32,
                        ))
                        .with_width(0.7),
                );
            }

            // 组合波形（振幅自动缩放防削波）
            let combo_amp = AMPLITUDE / n.sqrt().max(1.0);
            let combo_path = make_wave_path(w, mid, 600, |x| {
                let t = x * t_scale + t_offset;
                let sum: f64 = self.pitches.iter()
                    .map(|p| (TAU * p.frequency() * t + self.phase).sin())
                    .sum();
                (sum / n) * combo_amp
            });
            f.stroke(
                &combo_path,
                Stroke::default().with_color(Color::from_rgb8(100, 200, 255)).with_width(2.5),
            );

            // ── 基频虚线 ──
            let base_y = mid - (mid * combo_amp * 0.3);
            draw_dashed_line(
                &mut f, 0.0, w as f32, base_y as f32,
                Color::from_rgba8(255, 200, 100, 0.3), 1.0, 6.0, 4.0,
            );
            f.fill_text(Text {
                content: format!("基频 {:.1} Hz", base_freq),
                position: Point::new(w as f32 - 155.0, 12.0),
                color: Color::from_rgba8(255, 200, 100, 0.7),
                size: 14.0.into(),
                ..Text::default()
            });

            // 标签
            let label: String = self.pitches.iter().map(|p| p.label()).collect::<Vec<_>>().join(" + ");
            f.fill_text(Text {
                content: label,
                position: Point::new(8.0, 14.0),
                color: Color::from_rgb8(180, 200, 220),
                size: 11.0.into(),
                ..Text::default()
            });
        }

        // ── 键盘指示条 ──
        draw_keyboard(&mut f, w, h_wave, &self.pitches);

        vec![f.into_geometry()]
    }
}
