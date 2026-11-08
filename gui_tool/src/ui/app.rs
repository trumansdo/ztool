//! 应用状态管理和顶部菜单栏 + 内容区布局渲染

use crate::features::json_fmt::JsonFormatter;
use crate::features::music_wave::MusicWave;
use crate::features::net_capture::PacketCapture;
use crate::features::net_port_scan::NetScanner;
use crate::features::prompt_gen::PromptGenerator;
use crate::features::pyramid_3d::Pyramid;
use crate::features::ui_libs::UiLibs;
use crate::ui::state::UiState;
use crate::ui::widgets::menu::{MenuBar, MenuTree};
use iced::widget::{column, container};
use iced::{Element, Length, Task};

use super::{Message, Tab};

pub struct App {
    pub selected_tab: Tab,
    pub ui_state: UiState,
    pub json_formatter: JsonFormatter,
    pub net_port_scan: NetScanner,
    pub packet_capture: PacketCapture,
    pub ui_libs: UiLibs,
    pub pyramid: Pyramid,
    pub music_wave: MusicWave,
    pub prompt_gen: PromptGenerator,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected_tab: Tab::default(),
            ui_state: UiState::default(),
            json_formatter: JsonFormatter::default(),
            net_port_scan: NetScanner::default(),
            packet_capture: PacketCapture::default(),
            ui_libs: UiLibs::default(),
            pyramid: Pyramid::default(),
            music_wave: MusicWave::default(),
            prompt_gen: PromptGenerator::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                let prev = self.selected_tab;
                self.selected_tab = tab;
                // 首次切换到提示词生成时触发加载
                if tab == Tab::PromptGen && prev != Tab::PromptGen {
                    return crate::features::prompt_gen::update(&mut self.prompt_gen, crate::features::prompt_gen::Msg::Load)
                        .map(Message::PromptGen);
                }
                Task::none()
            }
            Message::JsonFmt(m) => {
                crate::features::json_fmt::update(&mut self.json_formatter, m).map(Message::JsonFmt)
            }
            Message::NetPortScan(m) => {
                crate::features::net_port_scan::update(&mut self.net_port_scan, m).map(Message::NetPortScan)
            }
            Message::NetCapture(m) => {
                crate::features::net_capture::update(&mut self.packet_capture, m).map(Message::NetCapture)
            }
            Message::UiLibs(m) => crate::features::ui_libs::update(&mut self.ui_libs, m).map(Message::UiLibs),
            Message::ShaderPyramid(m) => {
                crate::features::pyramid_3d::update(&mut self.pyramid, m).map(Message::ShaderPyramid)
            }
            Message::MusicWave(m) => {
                crate::features::music_wave::update(&mut self.music_wave, m).map(Message::MusicWave)
            }
            Message::PromptGen(m) => {
                crate::features::prompt_gen::update(&mut self.prompt_gen, m).map(Message::PromptGen)
            }
            Message::SetTheme(id) => {
                self.set_theme(&id);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // 顶部菜单栏（Windows 经典菜单风格，替代原左侧树形导航）
        // 根菜单：功能分类；下拉菜单：具体功能页面
        let menu_bar = MenuBar::new(vec![
            MenuTree::folder(
                "网络工具",
                vec![
                    MenuTree::item("端口扫描", Message::TabSelected(Tab::NetPortScan)),
                    MenuTree::item("网络抓包", Message::TabSelected(Tab::NetCapture)),
                ],
            ),
            MenuTree::folder(
                "数据工具",
                vec![MenuTree::item("JSON格式化", Message::TabSelected(Tab::JsonFmt))],
            ),
            MenuTree::folder(
                "组件库",
                vec![MenuTree::item("组件示例", Message::TabSelected(Tab::UiLibs))],
            ),
            MenuTree::folder(
                "3D展示",
                vec![MenuTree::item("金字塔", Message::TabSelected(Tab::Pyramid3d))],
            ),
            MenuTree::folder(
                "音乐波形",
                vec![MenuTree::item("音阶波形", Message::TabSelected(Tab::MusicWave))],
            ),
            MenuTree::folder(
                "AI工具",
                vec![MenuTree::item("提示词生成", Message::TabSelected(Tab::PromptGen))],
            ),
            MenuTree::folder(
                "主题",
                crate::theme::ThemeId::all()
                    .iter()
                    .map(|id| {
                        let label = if id.dark { format!("🌙 {}", id.name) } else { format!("☀ {}", id.name) };
                        MenuTree::item(label, Message::SetTheme(id.clone()))
                    })
                    .collect(),
            ),
        ]);

        // 顶部菜单栏容器（令牌层颜色）
        let menu_bg = self.ui_state.styles.tokens().surface;
        let menu_panel = container(menu_bar)
            .width(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(menu_bg)),
                ..Default::default()
            });

        let content: Element<'_, Message> = match self.selected_tab {
            Tab::JsonFmt => {
                crate::features::json_fmt::view(&self.json_formatter).map(Message::JsonFmt)
            }
            Tab::NetPortScan => {
                crate::features::net_port_scan::view(&self.net_port_scan).map(Message::NetPortScan)
            }
            Tab::NetCapture => {
                crate::features::net_capture::view(&self.packet_capture).map(Message::NetCapture)
            }
            Tab::UiLibs => {
                crate::features::ui_libs::view(&self.ui_libs).map(Message::UiLibs)
            }
            Tab::Pyramid3d => {
                crate::features::pyramid_3d::view(&self.pyramid).map(Message::ShaderPyramid)
            }
            Tab::MusicWave => {
                crate::features::music_wave::view(&self.music_wave).map(Message::MusicWave)
            }
            Tab::PromptGen => {
                crate::features::prompt_gen::view(&self.prompt_gen, self.ui_state.styles.as_ref()).map(Message::PromptGen)
            }
        };

        let content_panel = container(content)
            .width(Length::Fill)
            .height(Length::Fill);

        // 布局：顶部菜单栏 + 内容区（垂直排列）
        column![menu_panel, content_panel]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn title() -> String {
        "综合工具".to_string()
    }

    pub fn theme(&self) -> iced::Theme {
        self.ui_state.styles.iced_theme()
    }

    /// 切换主题
    pub fn set_theme(&mut self, id: &crate::theme::ThemeId) {
        let styles = Box::new(crate::theme::kit::KitStyles::new(id));
        self.ui_state = crate::ui::state::UiState::new(id.clone(), styles);
    }
}
