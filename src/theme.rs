//! Office Fluent 风格主题（对照 Qitan Ribbon Office 截图，
//! 详见 docs/office-ribbon-style-notes.md）。
//! 蓝 / 绿两套预设；icon_bytes 的重映射同步切亮/暗（工具区为白底 → 亮色映射）。

use gpui_kit::Hsla;

pub fn rgb(hex: u32) -> Hsla {
    gpui_kit::rgb(hex).into()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OfficePreset {
    Blue,
    #[allow(dead_code)]
    Green,
}

#[derive(Clone, Copy)]
pub struct OfficeTheme {
    /// 标题栏深色
    pub title_bg: Hsla,
    /// 标签行主题色（中调）
    pub tab_bar_bg: Hsla,
    /// File 块背景（比标签行更深）
    pub file_bg: Hsla,
    /// 激活标签背景（白，点亮工具区）
    pub tab_active_bg: Hsla,
    pub tab_active_text: Hsla,
    pub tab_text: Hsla,
    /// 上下文标签（第二主题色）背景与文字
    pub ctx_bg: Hsla,
    pub ctx_text: Hsla,
    pub ctx_bar: Hsla,
    /// 工具区（白）
    pub tool_bg: Hsla,
    /// 按钮三态
    pub hover: Hsla,
    pub hover_border: Hsla,
    pub pressed: Hsla,
    pub stuck: Hsla,
    /// 文字
    pub text: Hsla,
    pub text_muted: Hsla,
    pub disabled: Hsla,
    /// 分隔线 / 组名
    pub group_line: Hsla,
    pub group_name: Hsla,
    /// 组合框/输入框
    pub input_bg: Hsla,
    pub input_border: Hsla,
    /// 状态栏
    pub status_bg: Hsla,
    pub status_text: Hsla,
    /// 强调（选中/链接）
    pub accent: Hsla,
}

pub fn office_blue() -> OfficeTheme {
    OfficeTheme {
    title_bg: rgb(0x1e3a75),
    tab_bar_bg: rgb(0x4a6fbf),
    file_bg: rgb(0x16305f),
    tab_active_bg: rgb(0xf5f5f5),
    tab_active_text: rgb(0x1e1e1e),
    tab_text: rgb(0xeef2fa),
    ctx_bg: rgb(0xe8b4c8),
    ctx_text: rgb(0x6b1f3a),
    ctx_bar: rgb(0xd89ab5),
    tool_bg: rgb(0xffffff),
    hover: rgb(0xcde3f7),
    hover_border: rgb(0x7eb4ea),
    pressed: rgb(0xa8cdf0),
    stuck: rgb(0xb5d8f5),
    text: rgb(0x1e1e1e),
    text_muted: rgb(0x666666),
    disabled: rgb(0xa0a0a0),
    group_line: rgb(0xd9d9d9),
    group_name: rgb(0x555555),
    input_bg: rgb(0xffffff),
    input_border: rgb(0x7a8a99),
    status_bg: rgb(0xf5f5f5),
    status_text: rgb(0x333333),
    accent: rgb(0x2a6db5),
    }
}

pub fn office_green() -> OfficeTheme {
    OfficeTheme {
    title_bg: rgb(0x4c8a2c),
    tab_bar_bg: rgb(0x7cb84e),
    file_bg: rgb(0x3d7022),
    tab_active_bg: rgb(0xf5f5f5),
    tab_active_text: rgb(0x1e1e1e),
    tab_text: rgb(0xf2f8ec),
    ctx_bg: rgb(0xf0c060),
    ctx_text: rgb(0x6b4a10),
    ctx_bar: rgb(0xe0b050),
    tool_bg: rgb(0xffffff),
    hover: rgb(0xe0f0c8),
    hover_border: rgb(0x9cc86a),
    pressed: rgb(0xc8e0a0),
    stuck: rgb(0xd4e8b0),
    text: rgb(0x1e1e1e),
    text_muted: rgb(0x666666),
    disabled: rgb(0xa0a0a0),
    group_line: rgb(0xd9d9d9),
    group_name: rgb(0x555555),
    input_bg: rgb(0xffffff),
    input_border: rgb(0x7a9960),
    status_bg: rgb(0x9cc86a),
    status_text: rgb(0x1e3a10),
    accent: rgb(0x4c8a2c),
    }
}

pub fn preset(p: OfficePreset) -> OfficeTheme {
    match p {
        OfficePreset::Blue => office_blue(),
        OfficePreset::Green => office_green(),
    }
}
