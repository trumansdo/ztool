//! 截屏 → ASCII 艺术转换
//!
//! 完全参考 image-to-ascii 0.7.2 设计 + 四项增强：
//! 1. Alpha 阈值处理（透明像素→空格）
//! 2. 颜色缓存（HashMap 避免重复 ANSI 字符串分配）
//! 3. 对比度拉伸（min-max 拉伸到 0..1）
//! 4. 边缘检测叠加（Sobel 算子，edge-augmented 风格）

use std::collections::HashMap;
use image::{DynamicImage, GenericImageView, RgbaImage};
use xcap::Monitor;

// ── 灰度计算（与 image-to-ascii 完全一致）────────────────────────

#[inline]
fn linearize(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

#[inline]
fn rgba_to_luma(r: u8, g: u8, b: u8, a: u8) -> f32 {
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
        * (a as f32 / 255.0)
}

// ── 字符 ramp ────────────────────────────────────────────────────

const RAMP: &[u8] = b" .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";

// ── 边缘检测（Sobel 3×3）─────────────────────────────────────────

fn sobel_edge(luma_grid: &[f32], w: u32, h: u32, x: u32, y: u32) -> f32 {
    if x == 0 || y == 0 || x >= w - 1 || y >= h - 1 {
        return 0.0;
    }
    let g = |dx: i32, dy: i32| luma_grid[((y as i32 + dy) as u32 * w + (x as i32 + dx) as u32) as usize];

    let gx = -1.0 * g(-1,-1) + 1.0 * g(1,-1)
           + -2.0 * g(-1, 0) + 2.0 * g(1, 0)
           + -1.0 * g(-1, 1) + 1.0 * g(1, 1);

    let gy = -1.0 * g(-1,-1) + -2.0 * g(0,-1) + -1.0 * g(1,-1)
           +  1.0 * g(-1, 1) +  2.0 * g(0, 1) +  1.0 * g(1, 1);

    (gx * gx + gy * gy).sqrt() / 4.0 // 归一化到 ~0..1
}

// ── 主函数 ───────────────────────────────────────────────────────

fn main() {
    // 1. 截屏
    let monitors = Monitor::all().expect("无法获取显示器列表");
    let primary = monitors.iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .expect("未找到主显示器");
    let rgba_img: RgbaImage = primary.capture_image().expect("截屏失败");
    let img = DynamicImage::from(rgba_img);
    let (w, h) = img.dimensions();

    // 2. 字体参数（bitocra-13: 8×13）
    let font_w: u32 = 8;
    let font_h: u32 = 13;
    let char_ratio = font_w as f32 / font_h as f32;

    // 3. 输出字符网格
    let cols = 120u32;
    let rows = (h as f32 * cols as f32 / w as f32 * char_ratio) as u32;

    // 4. 颜色网格（用于着色）
    let color_grid = img.resize_exact(cols, rows, image::imageops::FilterType::Lanczos3);
    let color_pixels = color_grid.to_rgba8();

    // 5. 构建灰度数组 + 边缘检测
    let brightness_scale: f32 = 0.25;
    let edge_scale: f32 = 0.3; // 边缘叠加权重
    let alpha_threshold: u8 = 64;
    let total = (cols * rows) as usize;
    let mut lumas: Vec<f32> = Vec::with_capacity(total);

    for y in 0..rows {
        for x in 0..cols {
            let cp = color_pixels.get_pixel(x, y);
            let (r, g, b, a) = (cp[0], cp[1], cp[2], cp[3]);
            if a < alpha_threshold {
                lumas.push(f32::NAN); // 标记透明
            } else {
                lumas.push(rgba_to_luma(r, g, b, a));
            }
        }
    }

    // 6. 对比度拉伸（跳过透明像素）
    let valid_lumas: Vec<f32> = lumas.iter().filter(|v| !v.is_nan()).copied().collect();
    if !valid_lumas.is_empty() {
        let (min_l, max_l) = valid_lumas.iter().fold((f32::MAX, f32::MIN), |(min, max), &v| {
            (min.min(v), max.max(v))
        });
        let range = max_l - min_l;
        if range > 0.001 {
            for v in lumas.iter_mut() {
                if !v.is_nan() {
                    *v = (*v - min_l) / range;
                }
            }
        }
    }

    // 7. 边缘检测（在拉伸后的亮度上做 Sobel）
    let mut edges: Vec<f32> = vec![0.0; total];
    for y in 0..rows {
        for x in 0..cols {
            let idx = (y * cols + x) as usize;
            if lumas[idx].is_nan() { continue; }
            edges[idx] = sobel_edge(&lumas, cols, rows, x, y);
        }
    }

    // 8. 颜色缓存
    let mut color_cache: HashMap<(u8, u8, u8), String> = HashMap::new();
    let ramp_len = RAMP.len() as f32 - 1.0;
    let mut output = String::with_capacity((cols as usize + 1) * rows as usize * 30);

    for y in 0..rows {
        for x in 0..cols {
            let idx = (y * cols + x) as usize;
            let cp = color_pixels.get_pixel(x, y);
            let (r, g, b, a) = (cp[0], cp[1], cp[2], cp[3]);

            // Alpha 阈值：透明→空格
            if a < alpha_threshold {
                output.push(' ');
                continue;
            }

            // 亮度 + 边缘叠加 → 字符索引
            let luma = lumas[idx];
            let edge = edges[idx];
            let combined = (luma * brightness_scale + edge * edge_scale).clamp(0.0, 1.0);
            let char_idx = (combined * ramp_len).round() as usize;
            let ch = RAMP[char_idx.min(RAMP.len() - 1)] as char;

            // 颜色缓存
            let code = color_cache.entry((r, g, b)).or_insert_with(|| {
                let alpha = a as f32 / 255.0;
                let intensity = alpha * 255.0;
                format!(
                    "\x1b[38;2;{};{};{}m",
                    (r as f32 * intensity / 255.0) as u8,
                    (g as f32 * intensity / 255.0) as u8,
                    (b as f32 * intensity / 255.0) as u8,
                )
            });
            output.push_str(code);
            output.push(ch);
        }
        output.push_str("\x1b[0m\n");
    }

    print!("{}", output);
}
