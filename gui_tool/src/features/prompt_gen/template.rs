//! AI提示词生成器 —— 模板解析与变量替换
//!
//! 支持两种输入格式：
//! 1. 用户输入区：`#变量名\n内容` 定义变量
//! 2. 固定模板区：`${变量名}` 引用变量
//!
//! 内置变量 `${user_prompt}` 等价于所有用户变量内容的拼接。

use super::model::UserSection;

/// 解析用户输入区文本，提取变量定义
///
/// 格式：
/// ```text
/// #var1
/// 这是 var1 的内容
/// 可以多行
/// #var2
/// 这是 var2 的内容
/// ```
///
/// 规则：
/// - 以 `#` 开头的行开始一个新变量定义
/// - `#` 后的文本（到行尾）为变量名
/// - 后续行（直到下一个 `#` 或文本结束）为变量内容
/// - 变量名不能为空
/// - 空行被保留在内容中
pub fn parse_user_sections(input: &str) -> Vec<UserSection> {
    let mut sections = Vec::new();
    let mut current_name = String::new();
    let mut current_content = String::new();
    let mut in_section = false;

    for line in input.lines() {
        if line.starts_with('#') {
            // 保存上一个变量
            if in_section && !current_name.is_empty() {
                sections.push(UserSection {
                    var_name: current_name.clone(),
                    content: trim_trailing_newline(&current_content),
                });
            }
            // 开始新变量
            current_name = line[1..].trim().to_string();
            current_content = String::new();
            in_section = !current_name.is_empty();
        } else if in_section {
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        }
    }

    // 保存最后一个变量
    if in_section && !current_name.is_empty() {
        sections.push(UserSection {
            var_name: current_name,
            content: trim_trailing_newline(&current_content),
        });
    }

    sections
}

/// 将变量列表序列化为用户输入区文本
pub fn sections_to_text(sections: &[UserSection]) -> String {
    let mut text = String::new();
    for (i, section) in sections.iter().enumerate() {
        if i > 0 {
            text.push('\n');
        }
        text.push_str(&format!("#{}\n{}", section.var_name, section.content));
    }
    text
}

/// 渲染模板：将固定模板中的 `${var}` 替换为对应变量内容
///
/// 特殊变量：
/// - `${input}`：替换为用户输入区的完整原始文本
/// - `${user_prompt}`：替换为所有用户变量内容的拼接（用换行分隔）
///
/// 未找到的变量保留原样 `${var}`。
pub fn render_template(template: &str, sections: &[UserSection], raw_input: &str) -> String {
    let mut result = template.to_string();

    // 先处理 ${input} —— 整个原始输入
    result = result.replace("${input}", raw_input);

    // 处理 ${user_prompt}
    let user_prompt = sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    result = result.replace("${user_prompt}", &user_prompt);

    // 处理自定义变量 ${var_name}
    for section in sections {
        let placeholder = format!("${{{}}}", section.var_name);
        result = result.replace(&placeholder, &section.content);
    }

    result
}

/// 去除末尾多余换行
fn trim_trailing_newline(s: &str) -> String {
    s.trim_end_matches('\n').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_section() {
        let input = "#role\n你是一个助手";
        let sections = parse_user_sections(input);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].var_name, "role");
        assert_eq!(sections[0].content, "你是一个助手");
    }

    #[test]
    fn test_parse_multiple_sections() {
        let input = "#a\n内容A\n#b\n内容B";
        let sections = parse_user_sections(input);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].var_name, "a");
        assert_eq!(sections[0].content, "内容A");
        assert_eq!(sections[1].var_name, "b");
        assert_eq!(sections[1].content, "内容B");
    }

    #[test]
    fn test_parse_multiline_content() {
        let input = "#desc\n第一行\n第二行\n第三行\n#next\n下一段";
        let sections = parse_user_sections(input);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].content, "第一行\n第二行\n第三行");
        assert_eq!(sections[1].content, "下一段");
    }

    #[test]
    fn test_render_template() {
        let sections = vec![
            UserSection {
                var_name: "role".into(),
                content: "助手".into(),
            },
            UserSection {
                var_name: "task".into(),
                content: "翻译".into(),
            },
        ];
        let raw_input = "#role\n助手\n#task\n翻译";
        let template = "你是${role}，请${task}：\n${user_prompt}\n---\n原始输入：${input}";
        let result = render_template(template, &sections, raw_input);
        assert!(result.contains("你是助手"));
        assert!(result.contains("请翻译"));
        assert!(result.contains("助手\n翻译"));
        assert!(result.contains(raw_input));
    }

    #[test]
    fn test_sections_to_text_roundtrip() {
        let input = "#a\n内容A\n#b\n内容B";
        let sections = parse_user_sections(input);
        let text = sections_to_text(&sections);
        let sections2 = parse_user_sections(&text);
        assert_eq!(sections.len(), sections2.len());
        for (a, b) in sections.iter().zip(sections2.iter()) {
            assert_eq!(a.var_name, b.var_name);
            assert_eq!(a.content, b.content);
        }
    }
}
