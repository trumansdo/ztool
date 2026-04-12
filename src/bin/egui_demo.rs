//! # Egui wgpu demo - 最小化 egui 示例
//!
//! 本文件展示了 egui 最基本的用法，是学习 egui 的入门示例。
//! - 使用 eframe 框架
//! - 学习 App trait 的 ui() 方法
//! - 了解 egui 常用控件
//!
//! ============================================================================
//! 📚 推荐阅读顺序
//! ============================================================================
//! 1. main 函数 (行 126) → 了解如何启动 egui 应用
//! 2. App 实现 (行 31)  → 了解 UI 构建入口
//! 3. 控件使用 (行 33-60) → 了解常用 widget
//! 4. 字体加载 (行 64)  → 了解中文支持
//!
//! ============================================================================

// ============================================================================
// 第一部分：导入
// ============================================================================

use anyhow::Result; // anyhow: 统一错误处理，简化错误传播
use eframe::egui; // egui: 核心 GUI 库，提供所有控件
use std::path::PathBuf; // PathBuf: 路径处理，跨平台路径表示
use thiserror::Error; // thiserror: 自定义错误派生宏

// ============================================================================
// 第二部分：错误定义
// ============================================================================

// #[derive(Debug, Error)] 自动派生 Debug, Error trait
#[derive(Debug, Error)]
pub enum AppError {
    // #[error(...)] thiserror 宏自动生成错误 Display 实现
    #[error("未找到中文字体")]
    FontNotFound,

    // #[from] 自动实现 From<std::io::Error> 转换
    #[error("读取字体文件失败: {0}")]
    ReadFontFailed(#[from] std::io::Error),
}

// ============================================================================
// 第三部分：应用状态结构
// ============================================================================

// MyApp 结构体：存储应用状态，实现 Default trait
struct MyApp {
    name: String,         // name: 姓名，字符串类型，用于单行文本输入
    age: i32,             // age: 年龄，整数类型，用于 Slider 滑动条
    backend_info: String, // backend_info: wgpu 渲染适配器信息
}

// impl Default: 提供默认状态构造
impl Default for MyApp {
    fn default() -> Self {
        // Self: 当前类型别名
        Self {
            // "World".to_string(): 创建非空字符串
            name: "World".to_string(),
            age: 30, // 默认年龄30
            backend_info: "点击按钮获取渲染适配器信息".to_string(),
        }
    }
}

// ============================================================================
// 第四部分：App trait 实现 ⭐ 核心
// ============================================================================
//
// eframe::App: egui 应用的入口 trait
// ui(): 每帧被调用的方法，构建即时模式 UI
// 参数:
// - ui: UI 构建上下文，用于添加控件
// - frame: 框架控制 (窗口管理、退出程序等)
//

impl eframe::App for MyApp {
    // egui 0.34 使用 ui() 方法 (旧版本 0.27 用 update())
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // egui::CentralPanel::default(): 中央面板，占据窗口中央区域
        // .show(ui, |ui| {}): 在面板内构建 UI 的闭包
        egui::CentralPanel::default().show(ui, |ui| {
            // ui.heading(): 大标题控件，显示文本
            ui.heading("Hello egui with wgpu!");

            // ui.horizontal(): 水平布局容器，内部控件水平排列
            ui.horizontal(|ui| {
                // ui.label(): 文本标签，只读显示文本
                ui.label("Your name: ");
                // ui.text_edit_singleline(): 单行文本输入框
                // &mut self.name: 可变引用，实时同步文本
                ui.text_edit_singleline(&mut self.name);
            });

            // ui.add(): 添加返回 Response 的控件的通用方法
            // egui::Slider::new(): 滑动条控件
            // (&mut self.age, 0..=120): 绑定值，范围0到120
            // .text("Age"): 显示标签文本
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("Age"));

            // ui.button(): 按钮控件
            // .clicked(): 立即模式检测，返回bool
            if ui.button("Click me").clicked() {
                // println!: 标准输出打印
                println!("Hello, {}!", self.name);
            }

            // ui.separator(): 水平分隔线
            ui.separator();

            // egui::CollapsingHeader: 可折叠的标题区域
            // .new("标题文本"): 构造函数
            egui::CollapsingHeader::new("渲染信息").show(ui, |ui| {
                ui.label("渲染后端: eframe + wgpu");
                ui.label("软件渲染: 通过 WGPU_POWER_PREF=low 启用");
                ui.label("支持的API: Vulkan/Metal/DirectX12/OpenGL/WebGPU");
            });

            // 再次使用 separator() 分隔区域
            ui.separator();

            // ui.label(): 显示存储的字符串
            ui.label(&self.backend_info);

            // button 点击检测，更新状态
            if ui
                .button("获取渲染适配器信息")
                .clicked()
            {
                // 调用函数获取信息并赋值给状态
                self.backend_info = get_wgpu_adapter_info();
            }
        });
    }
}

// ============================================================================
// 第五部分：字体查找
// ============================================================================

// find_chinese_font_path(): 查找中文字体文件路径
// 返回 Result<PathBuf, AppError> 类型
fn find_chinese_font_path() -> Result<PathBuf, AppError> {
    // PathBuf::from(): 从 &str 创建路径
    let font_dir = PathBuf::from("C:/Windows/Fonts");
    // 数组字面量：字体名称列表
    let font_names = ["msyh.ttc", "simhei.ttf", "simsun.ttc", "msjh.ttc"];

    // for 循环遍历数组
    for name in &font_names {
        // PathBuf::join(): 拼接路径段
        let path = font_dir.join(name);
        // .exists(): 检查路径是否存在
        if path.exists() {
            // Ok(): 成功返回
            return Ok(path);
        }
    }
    // Err(): 错误返回
    Err(AppError::FontNotFound)
}

// ============================================================================
// 第六部分：字体加载
// ============================================================================

// load_chinese_fonts(): 加载字体定义为 egui 格式
fn load_chinese_fonts() -> Result<egui::FontDefinitions, AppError> {
    // ? 运算符：自动传播错误
    let font_path = find_chinese_font_path()?;
    // std::fs::read(): 读取文件到 Vec<u8>
    let data = std::fs::read(&font_path)?;

    // egui::FontDefinitions::default(): 创建默认字体定义
    let mut fonts = egui::FontDefinitions::default();

    // .font_data: HashMap 字段，存储字体二进制
    // .insert(): 插入键值对
    // .to_owned(): 创建 owned 的 String
    // egui::FontData::from_owned(): 从 bytes 创建 FontData
    fonts
        .font_data
        .insert("chinese".to_owned(), egui::FontData::from_owned(data).into());

    // .families: HashMap 字段，存储字体族
    // .entry(): 获取或插入条目
    // FontFamily::Proportional: 等宽字体
    // .or_default(): 获取默认值或插入空 Vec
    // .insert(0, "chinese"): 插入字体名称到索引0
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

    // Ok(): 返回成功
    Ok(fonts)
}

// ============================================================================
// 渲染模式配置 (通过环境变量控制)
// ============================================================================
//
// 运行前设置环境变量控制渲染后端:
//   $env:WGPU_POWER_PREF="low"; cargo run --bin egui_demo
//
// ============================================================================

// get_wgpu_adapter_info(): 获取 wgpu 适配器信息
fn get_wgpu_adapter_info() -> String {
    // 字符串字面量：多行字符串
    "=== wgpu 渲染适配器信息 ===\n\n\
     渲染模式: 默认 GPU (通过环境变量 WGPU_POWER_PREF 控制)\n\n\
     配置方式:\n\
     1. WGPU_POWER_PREF=high  -> GPU渲染\n\
     2. WGPU_POWER_PREF=low   -> 软件渲染 (WARP on Windows)\n\n\
     在 Windows 上: 软件渲染使用 WARP (D3D12)\n\
     在 Linux 上: 软件渲染使用 llvmpipe (OpenGL)"
        // .to_string(): 转换为 String
        .to_string()
}

// ============================================================================
// 第七部分：main 函数 (应用入口)
// ============================================================================

// main(): 程序入口函数
fn main() -> Result<()> {
    // eframe::NativeOptions: 原生窗口配置
    // .default(): 获取默认值
    let options = eframe::NativeOptions {
        // .viewport: 视口配置
        // egui::ViewportBuilder: 视口构建器
        // .default(): 获取默认值
        // .with_inner_size([550.0, 550.0]): 设置窗口尺寸 [宽, 高]
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 550.0]),
        // ..Default::default(): 其余字段使用默认值
        ..Default::default()
    };

    // eframe::run_native(): 启动原生窗口应用
    // 参数1: 窗口标题
    // 参数2: NativeOptions 配置
    // 参数3: 闭包，创建 App 实例
    eframe::run_native(
        "Egui Wgpu Demo", // title: 窗口标题
        options,          // NativeOptions 配置
        Box::new(|cc| {
            // Box::new() 装箱闭包
            // |cc| 闭包参数: CreationContext，包含了 egui 上下文
            // load_chinese_fonts(): 加载中文字体
            if let Ok(fonts) = load_chinese_fonts() {
                // cc.egui_ctx: egui 上下文
                // .set_fonts(): 设置字体定义
                cc.egui_ctx.set_fonts(fonts);
            }
            // Ok(Box::new()): 成功返回装箱的 App 实例
            Ok(Box::new(MyApp::default()))
        }),
    )
    // .map_err(): 转换错误类型
    // anyhow::anyhow!(): 包装错误信息
    .map_err(|e| anyhow::anyhow!("运行 egui 应用失败: {}", e))?;

    // Ok(()): 成功退出
    Ok(())
}
