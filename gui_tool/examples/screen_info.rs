//! 屏幕信息快照 —— 基于 uiautomation-rs 的简单示例
//!
//! 启动时枚举当前桌面所有顶层窗口，输出其 Name、ClassName、
//! BoundingRectangle、ProcessId、FrameworkId、IsOffscreen 等属性。
//!
//! # 用法
//!
//! ```bash
//! cargo run --example screen_info
//! ```

use uiautomation::UIAutomation;
use uiautomation::UIElement;

fn main() -> uiautomation::Result<()> {
    println!("═══════════════════════════════════════════════════");
    println!("  屏幕信息快照 (Screen Info Snapshot)");
    println!("═══════════════════════════════════════════════════\n");

    // 1. 初始化 UIAutomation
    let automation = UIAutomation::new()?;
    let root = automation.get_root_element()?;

    // 2. 打印桌面根元素信息
    println!("━━━ Desktop (Root Element) ━━━");
    print_element_details(&root, "  ");

    // 3. 获取 Control View Walker 并遍历顶层子元素
    //    也可以直接用 get_control_view_walker()
    let walker = automation.get_control_view_walker()?;

    println!("\n━━━ 顶层窗口列表 (Top-Level Windows) ━━━");

    let mut count = 0u32;

    // 取第一个子元素
    let mut current = match walker.get_first_child(&root) {
        Ok(child) => child,
        Err(_) => {
            println!("  (没有找到任何子元素)");
            return Ok(());
        }
    };

    loop {
        count += 1;
        print_window_info(&current, count);

        // 尝试移动到下一个兄弟元素
        match walker.get_next_sibling(&current) {
            Ok(next) => current = next,
            Err(_) => break,
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  共发现 {} 个顶层窗口/元素", count);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 打印单个元素的核心属性（调试用）
fn print_element_details(elem: &UIElement, indent: &str) {
    let name = elem.get_name().unwrap_or_default();
    let class = elem.get_classname().unwrap_or_default();
    let ctrl_type = elem.get_localized_control_type().unwrap_or_default();
    let rect = elem.get_bounding_rectangle().unwrap_or_default();
    let pid = elem.get_process_id().unwrap_or(-1);
    let fwid = elem.get_framework_id().unwrap_or_default();
    let enabled = elem.is_enabled().unwrap_or(false);
    let offscreen = elem.is_offscreen().unwrap_or(true);
    let handle = elem.get_native_window_handle().unwrap_or_default();

    println!("{}Name             : {}", indent, name);
    println!("{}ClassName        : {}", indent, class);
    println!("{}ControlType      : {}", indent, ctrl_type);
    println!("{}BoundingRect     : {}", indent, rect);
    if rect.get_width() > 0 && rect.get_height() > 0 {
        println!("{}  ├─ Size        : {} x {}", indent, rect.get_width(), rect.get_height());
    }
    println!("{}ProcessId        : {}", indent, pid);
    println!("{}FrameworkId      : {}", indent, fwid);
    println!("{}IsEnabled        : {}", indent, enabled);
    println!("{}IsOffscreen      : {}", indent, offscreen);
    println!("{}NativeWindowHandle: {}", indent, handle);
}

/// 简洁输出一条窗口信息
fn print_window_info(elem: &UIElement, index: u32) {
    let name = elem.get_name().unwrap_or_else(|_| String::from("(unnamed)"));
    let class = elem.get_classname().unwrap_or_else(|_| String::from("(unknown)"));
    let rect = elem.get_bounding_rectangle().unwrap_or_default();
    let pid = elem.get_process_id().unwrap_or(-1);

    // 截断过长的名称（安全处理多字节字符）
    let display_name = if name.chars().count() > 60 {
        let truncated: String = name.chars().take(57).collect();
        format!("{}...", truncated)
    } else {
        name
    };

    println!(
        "  [{:2}] {:32} | Class={:24} | Rect={} | PID={}",
        index,
        display_name,
        class,
        rect,
        pid,
    );
}
