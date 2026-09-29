//! Office Fluent 风格渲染原语与新形态组件（对照 docs/office-ribbon-style-notes.md）。
//! 主题色经 gpui Global 全局读取，渲染函数无需逐层传参。

use gpui_kit::prelude::FluentBuilder;
use gpui_kit::component::StyledExt;
use gpui_kit::{
    div, img, px, AnyElement, App, ElementId, InteractiveElement as _, IntoElement,
    ParentElement, SharedString, Stateful, StatefulInteractiveElement as _, Styled,
};

use crate::icons;
use crate::state;
use crate::theme::{preset, OfficePreset, OfficeTheme};

// ── 主题全局 ────────────────────────────────────────────────────────────────

pub struct OfficeThemeGlobal(pub OfficeTheme);

impl gpui_kit::Global for OfficeThemeGlobal {}

pub fn init_office_theme(p: OfficePreset, cx: &mut App) {
    cx.set_global(OfficeThemeGlobal(preset(p)));
    // 工具区为白底：图标重映射切到"亮色"（深灰笔画 + Office 蓝强调）
    cad_demo::icon_bytes::set_light_mode(true);
}

pub fn office_theme(cx: &App) -> OfficeTheme {
    cx.try_global::<OfficeThemeGlobal>()
        .map(|g| g.0)
        .unwrap_or(crate::theme::office_blue())
}

#[allow(dead_code)]
pub fn set_office_theme(p: OfficePreset, cx: &mut App) {
    init_office_theme(p, cx);
    cx.refresh_windows();
}

/// 无 cx 环境读取（init 后可用）。
static OT_CACHED: std::sync::OnceLock<OfficeTheme> = std::sync::OnceLock::new();

pub fn ot_cached() -> OfficeTheme {
    *OT_CACHED.get_or_init(|| crate::theme::office_blue())
}

// ── 按钮三态（hover 圆角浅填充+边框 / 按下更深 / 贴住保持）──────────────────

pub fn office_btn(
    el: Stateful<gpui_kit::Div>,
    active: bool,
    disabled: bool,
    ot: &OfficeTheme,
) -> Stateful<gpui_kit::Div> {
    if disabled {
        return el;
    }
    let hover = ot.hover;
    let hover_border = ot.hover_border;
    let pressed = ot.pressed;
    el.when(active, |el| el.bg(ot.stuck))
        .when(!active, |el| {
            el.hover(move |s| s.bg(hover).border_color(hover_border))
        })
        .active(move |s| s.bg(pressed))
}

/// 图标按钮（Office 三态，圆角 3px）。
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn icon_button(
    ui_key: &str,
    w: f32,
    h: f32,
    icon: &'static str,
    icon_size: f32,
    active: bool,
    disabled: bool,
    tip: SharedString,
    ot: &OfficeTheme,
    model_cmd: &'static str,
) -> Stateful<gpui_kit::Div> {
    let base = div()
        .id(ElementId::Name(ui_key.into()))
        .w(px(w))
        .h(px(h))
        .rounded_sm()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer();
    let base = office_btn(base, active, disabled, ot);
    let base = if disabled {
        base.opacity(0.45)
    } else {
        base.on_click(move |_, _, cx| state::run_tool(model_cmd, cx))
    };
    attach_tip(base.child(img(icons::icon(icon)).size(px(icon_size)).flex_shrink_0()), tip)
}

/// 字符按钮（B / I / U …）：自绘文字。
pub fn char_button(
    ui_key: &str,
    w: f32,
    h: f32,
    ch: &'static str,
    active: bool,
    ot: &OfficeTheme,
    cmd: &'static str,
) -> Stateful<gpui_kit::Div> {
    let base = div()
        .id(ElementId::Name(ui_key.into()))
        .w(px(w))
        .h(px(h))
        .rounded_sm()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_size(px(12.0));
    let text_color = if matches!(ch, "B") {
        ot.text
    } else {
        ot.text_muted
    };
    let base = office_btn(base, active, false, ot).child(
        div()
            .text_color(text_color)
            .child(SharedString::from(ch)),
    );
    attach_tip(base, format!("{ch}\n命令: {cmd}"))
}

fn attach_tip(el: Stateful<gpui_kit::Div>, text: impl Into<SharedString>) -> Stateful<gpui_kit::Div> {
    let text = text.into();
    el.tooltip(move |window, cx| {
        gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window, cx)
    })
}

// ── 新形态渲染 ──────────────────────────────────────────────────────────────

/// Office 剪贴板形态：左侧大按钮，右侧竖排横排小按钮（图标+文字，可带▾）。
pub fn render_paste_group(
    key: &str,
    icon: &'static str,
    label: &'static str,
    cmd: &'static str,
    sides: &'static [(&'static str, &'static str, &'static str, bool)],
    ot: &OfficeTheme,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .gap_1()
        .child(
            // 大按钮：图标 + 两行文字
            div()
                .id(ElementId::Name(format!("{key}-big").into()))
                .w(px(52.0))
                .h_full()
                .rounded_sm()
                .cursor_pointer()
                .hover(move |s| s.bg(ot.hover).border_color(ot.hover_border))
                .active(move |s| s.bg(ot.pressed))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_0p5()
                .child(img(icons::icon(icon)).size(px(30.0)))
                .child(
                    div().text_size(px(10.5)).text_color(ot.text).child(SharedString::from(label)),
                )
                .on_click(move |_, _, cx| state::run_tool(cmd, cx)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .justify_center()
                .children(sides.iter().map(|(icon, label, cmd, caret)| {
                    let has_menu = *caret;
                    let base = div()
                        .id(ElementId::Name(format!("{key}-s{label}").into()))
                        .h(px(18.0))
                        .rounded_sm()
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(ot.hover).border_color(ot.hover_border))
                        .child(
                            div().pl_0p5().child(
                                img(icons::icon(icon)).size(px(15.0)).flex_shrink_0(),
                            ),
                        )
                        .child(
                            div().px_0p5().text_size(px(10.5)).text_color(ot.text).child(
                                SharedString::from(*label),
                            ),
                        );
                    let base = if has_menu {
                        base.child(
                            div()
                                .pr_0p5()
                                .text_size(px(7.0))
                                .text_color(ot.text_muted)
                                .child("▾"),
                        )
                    } else {
                        base
                    };
                    base.on_click(move |_, _, cx| state::run_tool(cmd, cx))
                })),
        )
        .into_any_element()
}

/// Office 组合框行：一体式（值 + ▾）。
pub fn render_combo_row(
    key: &str,
    items: &'static [cad_demo::ribbon_data::Combo],
    ot: &OfficeTheme,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .gap_1()
        .items_center()
        .children(items.iter().enumerate().map(|(k, combo)| {
            let id = ElementId::Name(format!("{key}-combo-{k}").into());
            let value = SharedString::from(combo.value);
            div()
                .id(id)
                .w(px(combo.width))
                .h(px(20.0))
                .rounded_xs()
                .border_1()
                .border_color(ot.input_border)
                .bg(ot.input_bg)
                .flex()
                .items_center()
                .px_1()
                .cursor_pointer()
                .hover(move |s| s.border_color(ot.accent))
                .on_click(move |_, _, cx| state::run_command(combo.cmd, cx))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.0))
                        .text_color(ot.text)
                        .child(value),
                )
                .child(
                    div().text_size(px(7.0)).text_color(ot.text_muted).child("▾"),
                )
        }))
        .into_any_element()
}

/// Office 流式字符按钮行。
pub fn render_char_flow(
    key: &str,
    buttons: &'static [cad_demo::ribbon_data::CharBtn],
    ot: &OfficeTheme,
) -> AnyElement {
    div().flex().flex_row().gap_0p5().children(
        buttons.iter().enumerate().map(|(k, b)| {
            char_button(
                &format!("{key}-c{k}"),
                20.0,
                20.0,
                b.ch,
                false,
                ot,
                b.cmd,
            )
        }),
    ).into_any_element()
}

/// Office 数值微调组：标签 + 值框（▴▾）。
pub fn render_spin_row(
    key: &str,
    items: &'static [cad_demo::ribbon_data::SpinEdit],
    ot: &OfficeTheme,
) -> AnyElement {
    div().flex().flex_row().gap_2().children(items.iter().enumerate().map(|(k, sp)| {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .child(
                div().text_size(px(10.5)).text_color(ot.text_muted).child(
                    SharedString::from(sp.label),
                ),
            )
            .child(
                div()
                    .id(ElementId::Name(format!("{key}-spin-{k}").into()))
                    .flex()
                    .items_center()
                    .w(px(sp.width))
                    .h(px(18.0))
                    .rounded_xs()
                    .border_1()
                    .border_color(ot.input_border)
                    .bg(ot.input_bg)
                    .cursor_pointer()
                    .hover(move |s| s.border_color(ot.accent))
                    .on_click(move |_, _, cx| state::run_command(sp.cmd, cx))
                    .child(
                        div().flex_1().pl_1().text_size(px(10.5)).text_color(ot.text).child(
                            SharedString::from(sp.value),
                        ),
                    )
                    .child(
                        div()
                            .v_flex()
                            .items_center()
                            .px_0p5()
                            .text_size(px(5.0))
                            .text_color(ot.text_muted)
                            .child("▴")
                            .child("▾"),
                    ),
            )
    })).into_any_element()
}

/// Office 横排带文字按钮组（右箭头），可禁用。
pub fn render_action_row(
    key: &str,
    items: &'static [cad_demo::ribbon_data::ActionBtn],
    ot: &OfficeTheme,
) -> AnyElement {
    div().flex().flex_row().gap_1().children(items.iter().enumerate().map(|(k, a)| {
        let base = div()
            .id(ElementId::Name(format!("{key}-a{k}").into()))
            .h(px(20.0))
            .rounded_sm()
            .flex()
            .items_center()
            .px_1()
            .gap_0p5()
            .text_size(px(10.5))
            .child(SharedString::from(a.label))
            .child(div().text_size(px(6.0)).child("▾"));
        let base = if a.disabled {
            base.text_color(ot.disabled).opacity(0.8)
        } else {
            let b = office_btn(base, false, false, ot).text_color(ot.text);
            b.on_click(move |_, _, cx| state::run_command(a.cmd, cx))
        };
        base
    })).into_any_element()
}
