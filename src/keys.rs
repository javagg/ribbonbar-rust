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

actions!(cad, [CloseAppMenu, ToggleRibbonMinimize, ShowKeyTips]);

/// 键盘切标签：payload 为标签下标。
#[derive(Clone, PartialEq, gpui_kit::Action)]
#[action(namespace = cad, no_json)]
pub struct SwitchTab(pub usize);

/// 启动时把键位表绑到窗口（App 级 keymap）。
pub fn bind(cx: &mut App) {
    cx.bind_keys(TOOL_KEYS.iter().map(|(key, cmd)| {
        KeyBinding::new(&key.to_lowercase(), ToolAction(cmd), None)
    }));
    cx.bind_keys([KeyBinding::new("escape", CloseAppMenu, None)]);
    cx.bind_keys([KeyBinding::new("ctrl-f1", ToggleRibbonMinimize, None)]);
    // KeyTip 激活键（FR DefaultKeyTipKeys：Alt/F10）；标签页固定键串
    // （避开 TOOL_KEYS：绘图 D/参数化 Q/模型 3/视图 V/管理 G）
    cx.bind_keys([
        KeyBinding::new("alt", ShowKeyTips, None),
        KeyBinding::new("f10", ShowKeyTips, None),
    ]);
    for (key, tab) in [("d", 0usize), ("q", 1), ("3", 2), ("v", 3), ("g", 4)] {
        cx.bind_keys([KeyBinding::new(key, SwitchTab(tab), None)]);
    }
}
