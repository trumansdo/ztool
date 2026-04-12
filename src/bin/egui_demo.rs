use anyhow::Result;
use eframe::egui;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("未找到中文字体")]
    FontNotFound,

    #[error("读取字体文件失败: {0}")]
    ReadFontFailed(#[from] std::io::Error),
}

struct MyApp {
    name: String,
    age: i32,
    backend_info: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "World".to_string(),
            age: 30,
            backend_info: "点击按钮获取渲染适配器信息".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Hello egui with wgpu!");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("Age"));
            if ui.button("Click me").clicked() {
                println!("Hello, {}!", self.name);
            }
            ui.separator();

            egui::CollapsingHeader::new("渲染信息").show(ui, |ui| {
                ui.label("渲染后端: eframe + wgpu");
                ui.label("软件渲染: 通过 WgpuSetup 配置 CPU 渲染");
                ui.label("支持的API: Vulkan/Metal/DirectX12/OpenGL/WebGPU");
            });

            ui.separator();
            ui.label(&self.backend_info);

            if ui
                .button("获取渲染适配器信息")
                .clicked()
            {
                self.backend_info = get_wgpu_adapter_info();
            }
        });
    }
}

fn find_chinese_font_path() -> Result<PathBuf, AppError> {
    let font_dir = PathBuf::from("C:/Windows/Fonts");
    let font_names = ["msyh.ttc", "simhei.ttf", "simsun.ttc", "msjh.ttc"];

    for name in &font_names {
        let path = font_dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(AppError::FontNotFound)
}

fn load_chinese_fonts() -> Result<egui::FontDefinitions, AppError> {
    let font_path = find_chinese_font_path()?;
    let data = std::fs::read(&font_path)?;

    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert("chinese".to_owned(), egui::FontData::from_owned(data).into());
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "chinese".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "chinese".to_owned());

    Ok(fonts)
}

// ============================================================================
// 渲染模式配置 (通过环境变量控制)
// ============================================================================
//
// eframe/wgpu 通过环境变量控制渲染后端:
// | 环境变量 | 值 | 说明 |
// |---------|-----|------|
// | WGPU_POWER_PREF | high | GPU渲染 (默认) |
// | WGPU_POWER_PREF | low | 软件渲染 (WARP on Windows) |
// | WGPU_BACKEND | dx12 | DirectX 12 |
// | WGPU_BACKEND | vulkan | Vulkan |
// | WGPU_BACKEND | metal | Metal (macOS) |
//
// 运行示例:
//   $env:WGPU_POWER_PREF="low"; cargo run --bin egui_demo

fn get_wgpu_adapter_info() -> String {
    "=== wgpu 渲染适配器信息 ===\n\n\
     渲染模式: 默认 GPU (通过环境变量 WGPU_POWER_PREF 控制)\n\n\
     配置方式:\n\
     1. WGPU_POWER_PREF=high  -> GPU渲染\n\
     2. WGPU_POWER_PREF=low   -> 软件渲染 (WARP on Windows)\n\n\
     在 Windows 上: 软件渲染使用 WARP (D3D12)\n\
     在 Linux 上: 软件渲染使用 llvmpipe (OpenGL)"
        .to_string()
}

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Egui Wgpu Demo",
        options,
        Box::new(|cc| {
            if let Ok(fonts) = load_chinese_fonts() {
                cc.egui_ctx.set_fonts(fonts);
            }
            Ok(Box::new(MyApp::default()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("运行 egui 应用失败: {}", e))?;

    Ok(())
}
