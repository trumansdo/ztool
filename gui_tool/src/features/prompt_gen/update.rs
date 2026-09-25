//! AI提示词生成器 —— 状态管理和消息处理
//!
//! 管理提示词列表的 CRUD 操作、编辑状态、模板预览。

use iced::widget::text_editor;
use iced::Task;

use super::model::{PromptStore, PromptTemplate};
use super::storage;
use super::template;

/// 提示词生成器状态
#[derive(Default)]
pub struct PromptGenerator {
    /// 所有提示词存储
    pub store: PromptStore,
    /// 当前选中的提示词 ID
    pub selected_id: Option<String>,
    /// 标题编辑内容
    pub title_input: String,
    /// 用户输入区编辑内容
    pub user_input: text_editor::Content,
    /// 固定模板编辑内容
    pub template_input: text_editor::Content,
    /// 预览结果
    pub preview: String,
    /// 存储错误信息
    pub error: Option<String>,
    /// 加载状态
    pub loaded: bool,
}

/// 消息枚举
#[derive(Debug, Clone)]
pub enum Msg {
    /// 初始化加载
    Load,
    /// 加载完成
    Loaded(Result<PromptStore, String>),
    /// 新建提示词
    NewPrompt,
    /// 选择提示词
    SelectPrompt(String),
    /// 删除当前提示词
    DeletePrompt,
    /// 标题变化
    TitleChanged(String),
    /// 用户输入区变化
    UserInputChanged(text_editor::Action),
    /// 固定模板变化
    TemplateInputChanged(text_editor::Action),
    /// 保存当前提示词
    Save,
    /// 保存完成
    Saved(Result<(), String>),
    /// 刷新预览
    RefreshPreview,
    /// 复制预览结果
    CopyPreview,
    /// 清除错误
    ClearError,
}

pub fn update(pg: &mut PromptGenerator, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::Load => {
            if pg.loaded {
                return Task::none();
            }
            pg.loaded = true;
            Task::perform(load_store(), Msg::Loaded)
        }
        Msg::Loaded(result) => {
            match result {
                Ok(store) => {
                    pg.store = store;
                    // 默认选中第一个
                    let first_id = pg.store.prompts.first().map(|p| p.id.clone());
                    if pg.selected_id.is_none() {
                        if let Some(ref id) = first_id {
                            pg.select_prompt(id);
                        }
                    }
                }
                Err(e) => {
                    pg.error = Some(format!("加载失败: {}", e));
                }
            }
            Task::none()
        }
        Msg::NewPrompt => {
            let prompt = PromptTemplate::new("新提示词".to_string());
            let id = prompt.id.clone();
            pg.store.prompts.push(prompt);
            pg.select_prompt(&id);
            let store = pg.store.clone();
            Task::perform(save_store(store), Msg::Saved)
        }
        Msg::SelectPrompt(id) => {
            pg.select_prompt(&id);
            Task::none()
        }
        Msg::DeletePrompt => {
            let id_to_delete = pg.selected_id.clone();
            if let Some(ref id) = id_to_delete {
                storage::delete_by_id(&mut pg.store, id);
                let new_selected = pg.store.prompts.first().map(|p| p.id.clone());
                pg.selected_id = new_selected;
                if let Some(ref sid) = pg.selected_id.clone() {
                    pg.select_prompt(sid);
                } else {
                    pg.clear_editor();
                }
                let store = pg.store.clone();
                return Task::perform(save_store(store), Msg::Saved);
            }
            Task::none()
        }
        Msg::TitleChanged(title) => {
            pg.title_input = title.clone();
            // 先取出需要的值，避免借用冲突
            if let Some(prompt) = pg.current_prompt_mut() {
                prompt.title = title;
                prompt.touch();
            }
            Task::none()
        }
        Msg::UserInputChanged(action) => {
            pg.user_input.perform(action);
            // 先解析 sections，再更新 prompt
            let sections = template::parse_user_sections(&pg.user_input.text());
            if let Some(prompt) = pg.current_prompt_mut() {
                prompt.user_sections = sections;
                prompt.touch();
            }
            Task::perform(std::future::ready(()), |_| Msg::RefreshPreview)
        }
        Msg::TemplateInputChanged(action) => {
            pg.template_input.perform(action);
            // 先取出文本，避免借用冲突
            let text = pg.template_input.text().to_string();
            if let Some(prompt) = pg.current_prompt_mut() {
                prompt.fixed_template = text;
                prompt.touch();
            }
            Task::perform(std::future::ready(()), |_| Msg::RefreshPreview)
        }
        Msg::Save => {
            // 先收集所有需要的数据
            let title = pg.title_input.clone();
            let sections = template::parse_user_sections(&pg.user_input.text());
            let template_text = pg.template_input.text().to_string();
            if let Some(prompt) = pg.current_prompt_mut() {
                prompt.title = title;
                prompt.user_sections = sections;
                prompt.fixed_template = template_text;
                prompt.touch();
            }
            let store = pg.store.clone();
            Task::perform(save_store(store), Msg::Saved)
        }
        Msg::Saved(result) => {
            if let Err(e) = result {
                pg.error = Some(format!("保存失败: {}", e));
            }
            Task::none()
        }
        Msg::RefreshPreview => {
            let sections = template::parse_user_sections(&pg.user_input.text());
            let raw_input = pg.user_input.text().to_string();
            pg.preview = template::render_template(&pg.template_input.text(), &sections, &raw_input);
            Task::none()
        }
        Msg::CopyPreview => {
            iced::clipboard::write(pg.preview.clone())
        }
        Msg::ClearError => {
            pg.error = None;
            Task::none()
        }
    }
}

impl PromptGenerator {
    /// 选中指定 ID 的提示词并加载到编辑器
    fn select_prompt(&mut self, id: &str) {
        self.selected_id = Some(id.to_string());
        if let Some(prompt) = storage::find_by_id(&self.store, id) {
            self.title_input = prompt.title.clone();
            self.user_input =
                text_editor::Content::with_text(&template::sections_to_text(&prompt.user_sections));
            self.template_input =
                text_editor::Content::with_text(&prompt.fixed_template);
            let sections = template::parse_user_sections(&self.user_input.text());
            let raw_input = self.user_input.text().to_string();
            self.preview = template::render_template(&self.template_input.text(), &sections, &raw_input);
        }
    }

    /// 获取当前选中提示词的可变引用
    fn current_prompt_mut(&mut self) -> Option<&mut PromptTemplate> {
        self.selected_id
            .as_ref()
            .and_then(|id| storage::find_by_id_mut(&mut self.store, id))
    }

    /// 清空编辑器
    fn clear_editor(&mut self) {
        self.title_input.clear();
        self.user_input = text_editor::Content::default();
        self.template_input = text_editor::Content::default();
        self.preview.clear();
    }
}

/// 异步加载存储
async fn load_store() -> Result<PromptStore, String> {
    storage::load().map_err(|e| e.to_string())
}

/// 异步保存存储
async fn save_store(store: PromptStore) -> Result<(), String> {
    storage::save(&store).map_err(|e| e.to_string())
}
