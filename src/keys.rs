//! 单键快捷键体系：带命令参数的 [`ToolAction`] + 逐键 KeyBinding 注册。
//!
//! gpui 的按键绑定按 action 类型分发，实例携带 payload，因此一个 action 类型
//! 即可承载全部工具命令（`#[action(no_json)]` 免去 serde 依赖）。
//! 键位表在共享核心 `ribbon_data::TOOL_KEYS`，tooltip 第三行与之联动。

use gpui_kit::{actions, App, KeyBinding};

use cad_demo::ribbon_data::TOOL_KEYS;

/// 键盘触发的工具命令：payload 为命令 id（如 "line"）。
#[derive(Clone, PartialEq, gpui_kit::Action)]
#[action(namespace = cad, no_json)]
pub struct ToolAction(pub &'static str);

actions!(cad, [CloseAppMenu, ToggleRibbonMinimize]);

/// 启动时把键位表绑到窗口（App 级 keymap）。
pub fn bind(cx: &mut App) {
    cx.bind_keys(TOOL_KEYS.iter().map(|(key, cmd)| {
        KeyBinding::new(&key.to_lowercase(), ToolAction(cmd), None)
    }));
    cx.bind_keys([KeyBinding::new("escape", CloseAppMenu, None)]);
    cx.bind_keys([KeyBinding::new("ctrl-f1", ToggleRibbonMinimize, None)]);
}
