//! 主窗口装配：Office 风格 RibbonBar + DockArea + Office 状态栏。
//! 状态栏：左侧命令状态/坐标，右侧 捕捉开关 + 视图切换 + 缩放滑块 + 主题切换。

use gpui_kit::component::dock::DockArea;
use gpui_kit::component::StyledExt;
use gpui_kit::component::slider::{Slider, SliderState};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    div, img, px, rgb, AppContext as _, Context, ElementId, Entity, Focusable as _, FocusHandle,
    IntoElement, InteractiveElement as _, ParentElement, Render, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled, Subscription, Window,
};

use crate::icons;
use crate::keys::{self, CloseAppMenu, ToggleRibbonMinimize, ToolAction};
use crate::office_widgets::office_theme;
use crate::ribbon::RibbonBar;
use crate::state::{self, AppEvent};
use crate::theme::OfficePreset;

pub struct Workspace {
    ribbon: Entity<RibbonBar>,
    dock_area: Entity<DockArea>,
    zoom: Entity<SliderState>,
    /// 命令行面板的焦点句柄（单键工具：焦点在其子树时按键归输入）。
    cmd_input_focus: FocusHandle,
    _sub: Option<Subscription>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (canvas, command_line, properties, layers, palette) =
            crate::panels::build_panels(window, cx);
        let cmd_input_focus = command_line.focus_handle(cx);

        let dock_area = cx.new(|cx| {
            DockArea::new("cad-main", Some(1), window, cx)
                .with_renderer(gpui_kit::component::dock::DockSkin::new(cx))
        });
        dock_area.update(cx, |area, cx| {
            crate::panels::install_docks(
                area,
                canvas.clone(),
                command_line,
                properties,
                layers,
                palette,
                window,
                cx,
            );
        });

        let sub = state::app_state(cx).map(|app| {
            cx.subscribe(&app, |_, _, event: &AppEvent, cx| {
                if *event == AppEvent::Updated {
                    cx.notify();
                }
            })
        });

        let zoom = cx.new(|_| SliderState::new());

        Self {
            ribbon: cx.new(|cx| RibbonBar::new(window, cx)),
            dock_area,
            zoom,
            cmd_input_focus,
            _sub: sub,
        }
    }

    /// 单键工具绑定：焦点在命令行面板子树内时按键归输入，不触发工具；
    /// 应用菜单展开时按键不驱动工具。
    /// Input 元素实际聚焦的是其内部 frame 句柄（非 InputState::focus_handle），
    /// 因此用 contains_focused 沿焦点树判断，而非句柄相等。
    fn on_tool_key(&mut self, action: &ToolAction, window: &Window, cx: &mut Context<Self>) {
        if state::ribbon_snapshot(cx).app_menu_open
            || self.cmd_input_focus.contains_focused(window, cx)
        {
            return;
        }
        state::run_tool(action.0, cx);
    }

    /// Office 风格状态栏：状态文字 | 坐标 | 捕捉开关 | 视图 | 缩放 | 主题切换。
    fn render_status_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let ot = office_theme(cx);
        let Some(app) = state::app_state(cx) else {
            return div().h(px(26.0));
        };
        let s = &app.read(cx).model;

        let coords = s
            .cursor
            .map(|(x, y)| format!("X {:>10.4}   Y {:>10.4}", x, y))
            .unwrap_or_else(|| "X           -           Y           -".to_string());
        let last_cmd = s
            .undo_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "就绪".to_string());
        let toggles: [(&'static str, &'static str, &'static str, bool); 4] = [
            ("OSNAP", "status/osnap.svg", "osnap", s.osnap),
            ("ORTHO", "status/ortho.svg", "ortho", s.ortho),
            ("POLAR", "status/polar.svg", "polar", s.polar),
            ("LWT", "status/lwt.svg", "lwt", s.lwt),
        ];
        let zoom = self.zoom.clone();

        div()
            .h_flex()
            .w_full()
            .h(px(26.0))
            .flex_shrink_0()
            .items_center()
            .bg(ot.status_bg)
            .border_t_1()
            .border_color(ot.group_line)
            .px_2()
            .gap_2()
            // 左：最近命令 + 实时坐标
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(ot.status_text)
                    .child(SharedString::from(format!("命令: {last_cmd}"))),
            )
            .child(
                div()
                    .font_family("Consolas")
                    .text_size(px(11.0))
                    .text_color(ot.accent)
                    .child(SharedString::from(coords)),
            )
            .child(div().flex_1())
            // 右：捕捉开关
            .children(toggles.map(|(label, icon, kind, on)| {
                status_toggle(label, icon, kind, on, &ot)
            }))
            .child(
                div()
                    .w(px(1.0))
                    .h(px(14.0))
                    .mx_1()
                    .bg(ot.group_line)
                    .flex_shrink_0(),
            )
            // 视图切换（演示按钮组）
            .child(
                div()
                    .h_flex()
                    .gap_0p5()
                    .child(
                        div()
                            .px_1p5()
                            .h(px(18.0))
                            .flex().items_center()
                            .rounded_xs()
                            .bg(rgb(0xffffff))
                            .text_size(px(10.0))
                            .text_color(ot.text)
                            .child("视图"),
                    )
                    .child(
                        div()
                            .px_1p5()
                            .h(px(18.0))
                            .flex().items_center()
                            .rounded_xs()
                            .text_size(px(10.0))
                            .text_color(ot.status_text)
                            .child("布局"),
                    ),
            )
            // 缩放滑块 + 百分比
            .child(
                div().h_flex().items_center().gap_1().w(px(150.0)).child(
                    div().flex_1().child(Slider::new(&zoom)),
                ),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(ot.status_text)
                    .child("100%"),
            )
            // 主题切换（蓝 / 绿）
            .child(
                div()
                    .id("theme-blue")
                    .size(px(18.0))
                    .rounded_xs()
                    .border_1()
                    .border_color(ot.group_line)
                    .bg(rgb(0x4a6fbf))
                    .cursor_pointer()
                    .hover(move |s| s.opacity(0.8))
                    .on_click(|_, _, cx| {
                        crate::office_widgets::set_office_theme(OfficePreset::Blue, cx)
                    }),
            )
            .child(
                div()
                    .id("theme-green")
                    .size(px(18.0))
                    .rounded_xs()
                    .border_1()
                    .border_color(ot.group_line)
                    .bg(rgb(0x7cb84e))
                    .cursor_pointer()
                    .hover(move |s| s.opacity(0.8))
                    .on_click(|_, _, cx| {
                        crate::office_widgets::set_office_theme(OfficePreset::Green, cx)
                    }),
            )
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ot = office_theme(cx);
        let ribbon = self.ribbon.clone();
        let ribbon_el = ribbon.update(cx, |r, cx| r.render(window, cx).into_any_element());
        // 应用菜单（Backstage）展开时接管功能区以下的全部区域
        let app_menu_open = state::ribbon_snapshot(cx).app_menu_open;
        let body: gpui_kit::AnyElement = if app_menu_open {
            self.ribbon
                .update(cx, |r, cx| r.render_app_menu(cx).into_any_element())
        } else {
            // dock 区点击收回最小化临时展开层（FR：点外部收回）
            div()
                .id("dock-area-host")
                .flex_1()
                .min_h_0()
                .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| {
                    let snap = state::ribbon_snapshot(cx);
                    if snap.ribbon_transient_open {
                        state::set_ribbon_transient_open(false, cx);
                    }
                })
                .child(self.dock_area.clone())
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(ot.tool_bg)
            .text_color(ot.text)
            .on_action(cx.listener(|this, action: &ToolAction, window, cx| {
                this.on_tool_key(action, window, cx);
                // KeyTip 会话中按工具键 → 直达并退出会话（FR Forward click）
                if state::ribbon_snapshot(cx).keytips_active {
                    state::set_keytips_active(false, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &CloseAppMenu, _, cx| {
                this.ribbon.update(cx, |r, cx| r.close_app_menu(cx));
                // Esc 兼职退出 KeyTip 会话（FR Back/Terminate）
                if state::ribbon_snapshot(cx).keytips_active {
                    state::set_keytips_active(false, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ToggleRibbonMinimize, _, cx| {
                let minimized = state::ribbon_snapshot(cx).ribbon_minimized;
                state::set_ribbon_minimized(!minimized, cx);
                let _ = this;
            }))
            .on_action(cx.listener(|this, _: &keys::ShowKeyTips, _, cx| {
                // 再按 Alt 终止会话（FR Terminate）
                let active = state::ribbon_snapshot(cx).keytips_active;
                state::set_keytips_active(!active, cx);
                let _ = this;
            }))
            .on_action(cx.listener(|this, action: &keys::SwitchTab, _, cx| {
                // 标签键只在 KeyTip 会话中生效（键位避开工具键，仍双保险）
                if state::ribbon_snapshot(cx).keytips_active {
                    this.ribbon.update(cx, |r, _| {
                        r.active = action.0;
                    });
                    // KeyTip 激活时最小化态切标签即临时展开，随后退出会话
                    if state::ribbon_snapshot(cx).ribbon_minimized {
                        state::set_ribbon_transient_open(true, cx);
                    }
                    state::set_keytips_active(false, cx);
                }
            }))
            .child(ribbon_el)
            .child(body)
            .child(self.render_status_bar(cx))
    }
}

fn status_toggle(
    label: &'static str,
    icon: &'static str,
    kind: &'static str,
    on: bool,
    ot: &crate::theme::OfficeTheme,
) -> Stateful<gpui_kit::Div> {
    div()
        .id(ElementId::Name(format!("sb-{kind}").into()))
        .h_flex()
        .h(px(20.0))
        .px_1p5()
        .ml_1()
        .gap_1()
        .items_center()
        .rounded_xs()
        .cursor_pointer()
        .when(on, |el| {
            el.bg(ot.stuck).border_1().border_color(ot.hover_border)
        })
        .when(!on, |el| el.hover(|s| s.bg(ot.hover)))
        .on_click(move |_, _, cx| state::toggle(kind, cx))
        .child(
            div()
                .opacity(if on { 1.0 } else { 0.35 })
                .child(img(icons::icon(icon)).size(px(13.0)).flex_shrink_0()),
        )
        .child(
            div()
                .text_size(px(9.5))
                .text_color(if on { ot.text } else { ot.disabled })
                .child(label),
        )
}
