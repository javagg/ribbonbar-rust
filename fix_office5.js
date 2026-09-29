const fs = require('fs');
// office_widgets 清理
let o = fs.readFileSync('src/office_widgets.rs', 'utf8');
o = o.replace('    div, img, px, AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement,\n    ParentElement, SharedString, Stateful, StatefulInteractiveElement as _, Styled, Window,',
              '    div, img, px, AnyElement, App, ElementId, InteractiveElement as _, IntoElement,\n    ParentElement, SharedString, Stateful, StatefulInteractiveElement as _, Styled,');
o = o.replace('use crate::state::{self, RibbonSnapshot};', 'use crate::state;');
// dd 未使用
o = o.replace('                    let dd = *cmd;\n', '');
fs.writeFileSync('src/office_widgets.rs', o);

// ribbon.rs：主入口 init 主题（main.rs 里调用），render_tab_strip 未用 snap —— 去掉参数太麻烦，加下划线
let r = fs.readFileSync('src/ribbon.rs', 'utf8');
r = r.replace(`    fn render_tab_strip(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
        snap: &RibbonSnapshot,
    ) -> Stateful<gpui_kit::Div> {`, `    fn render_tab_strip(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
        _snap: &RibbonSnapshot,
    ) -> Stateful<gpui_kit::Div> {`);
r = r.replace(`use crate::office_widgets::{
    office_theme, render_action_row, render_char_flow, render_combo_row, render_paste_group,
    render_spin_row,
};`, `use crate::office_widgets::{office_theme, render_action_row, render_char_flow, render_combo_row, render_paste_group, render_spin_row};`);
fs.writeFileSync('src/ribbon.rs', r);

// main.rs：启动时初始化 Office 主题
let m = fs.readFileSync('src/main.rs', 'utf8');
m = m.replace('        state::init(cx);', `        state::init(cx);
        office_widgets::init_office_theme(theme::OfficePreset::Blue, cx);`);
fs.writeFileSync('src/main.rs', m);
console.log('OK');
