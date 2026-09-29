//! 主窗口装配：RibbonBar + DockArea + 自绘状态栏。
//! 状态栏模仿 CAD：左侧实时坐标，右侧对象捕捉开关（OSNAP/ORTHO/POLAR/LWT）。

use gpui_kit::component::dock::DockArea;
use gpui_kit::component::theme::Theme;
use gpui_kit::component::{ActiveTheme, StyledExt};
use gpui_kit::{InteractiveElement as _, StatefulInteractiveElement as _};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    div, img, px, rgba, AppContext as _, Context, ElementId, Entity, IntoElement, ParentElement,
    Render, SharedString, Stateful, Styled, Subscription, Window,
};

use crate::icons;
use crate::ribbon::RibbonBar;
use crate::state::{self, AppEvent};

pub struct Workspace {
    ribbon: Entity<RibbonBar>,
    dock_area: Entity<DockArea>,
    _sub: Option<Subscription>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (canvas, command_line, properties, layers, palette) =
            crate::panels::build_panels(window, cx);

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

        Self {
            ribbon: cx.new(|cx| RibbonBar::new(window, cx)),
            dock_area,
            _sub: sub,
        }
    }

    fn render_status_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let Some(app) = state::app_state(cx) else {
            return div().h(px(26.0));
        };
        let s = &app.read(cx).model;

        let coords = s
            .cursor
            .map(|(x, y)| format!("X {:>10.4}   Y {:>10.4}", x, y))
            .unwrap_or_else(|| "X           -           Y           -".to_string());
        let toggles: [(&'static str, &'static str, &'static str, bool); 4] = [
            ("OSNAP", "status/osnap.svg", "osnap", s.osnap),
            ("ORTHO", "status/ortho.svg", "ortho", s.ortho),
            ("POLAR", "status/polar.svg", "polar", s.polar),
            ("LWT", "status/lwt.svg", "lwt", s.lwt),
        ];

        div()
            .h_flex()
            .w_full()
            .h(px(26.0))
            .flex_shrink_0()
            .items_center()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.status_bar)
            .px_2()
            .child(
                div()
                    .font_family("Consolas")
                    .text_size(px(11.0))
                    .text_color(theme.accent)
                    .child(SharedString::from(coords)),
            )
            .child(div().flex_1())
            .children(toggles.map(|(label, icon, kind, on)| {
                status_toggle(label, icon, kind, on, &theme)
            }))
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let ribbon = self.ribbon.clone();
        let ribbon_el = ribbon.update(cx, |r, cx| r.render(window, cx).into_any_element());

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(ribbon_el)
            .child(div().flex_1().min_h_0().child(self.dock_area.clone()))
            .child(self.render_status_bar(cx))
    }
}

fn status_toggle(
    label: &'static str,
    icon: &'static str,
    kind: &'static str,
    on: bool,
    theme: &Theme,
) -> Stateful<gpui_kit::Div> {
    div()
        .id(ElementId::Name(format!("sb-{kind}").into()))
        .h_flex()
        .h(px(20.0))
        .px_1p5()
        .ml_1()
        .gap_1()
        .items_center()
        .rounded_sm()
        .cursor_pointer()
        .when(on, |el| {
            el.bg(rgba(0x0696d72e))
                .border_1()
                .border_color(rgba(0x0696d760))
        })
        .when(!on, |el| el.hover(|s| s.bg(rgba(0xffffff0d))))
        .on_click(move |_, _, cx| state::toggle(kind, cx))
        .child(
            div()
                .opacity(if on { 1.0 } else { 0.35 })
                .child(img(icons::icon(icon)).size(px(13.0)).flex_shrink_0()),
        )
        .child(
            div()
                .text_size(px(9.5))
                .text_color(if on {
                    theme.foreground
                } else {
                    theme.muted_foreground
                })
                .child(label),
        )
}
