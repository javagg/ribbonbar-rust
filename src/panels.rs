//! 停靠面板集合：模型视口（画布）、命令行、特性、图层、工具选项板。
//! 每个面板实现 gpui-component 的 `Panel`（表现层）+ `BasePanel`（行为层），
//! 通过 `panel_handle` 包装修入 DockArea。

use gpui_kit::component::dock::{BasePanel, DockArea, DockLayout, DockPlacement, Panel, PanelEvent, panel_handle};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::theme::Theme;
use gpui_kit::component::{ActiveTheme, StyledExt};
use gpui_kit::{InteractiveElement as _, StatefulInteractiveElement as _};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    canvas, div, hsla, img, px, rgba, rgb, size, App, AppContext as _, Bounds, ClickEvent, Context,
    ElementId, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement,
    MouseMoveEvent, ParentElement, PathBuilder, Pixels, Point, Render, SharedString, Stateful,
    Styled, Subscription, Window,
};

use crate::icons;
use crate::state;

// 1 模型单位 = 10 px
const UNIT: f32 = 10.0;

// ---------------------------------------------------------------------------
// 模型视口
// ---------------------------------------------------------------------------

pub struct CanvasPanel {
    cursor_window: Option<Point<Pixels>>,
    bounds: Bounds<Pixels>,
    focus_handle: FocusHandle,
}

impl CanvasPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            cursor_window: None,
            bounds: Bounds::default(),
            focus_handle: cx.focus_handle(),
        }
    }

    fn model_coords(&self) -> Option<(f64, f64)> {
        let c = self.cursor_window?;
        let center = self.bounds.center();
        let dx = (c.x - center.x) / px(UNIT);
        let dy = (center.y - c.y) / px(UNIT);
        Some((dx as f64, dy as f64))
    }
}

impl BasePanel for CanvasPanel {
    fn panel_name(&self) -> &'static str {
        "CanvasPanel"
    }
}

impl Panel for CanvasPanel {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "模型"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("模型".into())
    }

    fn title_bar(&self, _cx: &App) -> bool {
        false
    }

    fn inner_padding(&self, _cx: &App) -> bool {
        false
    }
}

impl Focusable for CanvasPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for CanvasPanel {}

impl Render for CanvasPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let tracker = cx.entity();

        div()
            .id("canvas-viewport")
            .size_full()
            .relative()
            .bg(hsla(0.0, 0.0, 0.965, 1.0))
            .overflow_hidden()
            .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
                this.cursor_window = Some(e.position);
                let coords = this.model_coords();
                state::set_cursor(coords, cx);
                cx.notify();
            }))
            .on_hover(cx.listener(|this, over: &bool, _, cx| {
                if !over && this.cursor_window.take().is_some() {
                    state::set_cursor(None, cx);
                    cx.notify();
                }
            }))
            .child(
                // 布局期记录视口 bounds 供坐标换算（不触发重绘）
                canvas(
                    move |bounds, _, cx| {
                        tracker.update(cx, |this, _| this.bounds = bounds);
                    },
                    move |bounds, _, window, cx| {
                        paint_viewport(bounds, entity.read(cx).cursor_window, window);
                    },
                )
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .top_2()
                    .left_2()
                    .px_2()
                    .py_0p5()
                    .rounded_xs()
                    .bg(rgba(0x1a1a1ad0))
                    .text_color(rgba(0xf2f2f2e0))
                    .text_size(px(10.5))
                    .child("模型空间"),
            )
    }
}

fn quad_line(
    window: &mut Window,
    origin: Point<Pixels>,
    x: Pixels,
    y: Pixels,
    ww: Pixels,
    hh: Pixels,
    color: impl Into<gpui_kit::Background>,
) {
    window.paint_quad(gpui_kit::fill(
        Bounds {
            origin: Point { x: origin.x + x, y: origin.y + y },
            size: size(ww, hh),
        },
        color,
    ));
}

fn paint_viewport(bounds: Bounds<Pixels>, cursor: Option<Point<Pixels>>, window: &mut Window) {
    let w = bounds.size.width;
    let h = bounds.size.height;
    if w <= px(0.0) || h <= px(0.0) {
        return;
    }
    let origin = bounds.origin;

    // 栅格：小格 10px、大格 50px
    let minor = rgba(0x00000012);
    let major = rgba(0x00000026);
    let mut x = px(0.0);
    while x <= w {
        let color = if ((x / px(UNIT)) as i32) % 5 == 0 { major } else { minor };
        quad_line(window, origin, x, px(0.0), px(1.0), h, color);
        x += px(UNIT);
    }
    let mut y = px(0.0);
    while y <= h {
        let color = if ((y / px(UNIT)) as i32) % 5 == 0 { major } else { minor };
        quad_line(window, origin, px(0.0), y, w, px(1.0), color);
        y += px(UNIT);
    }

    // 坐标轴（过原点）
    let cxp = w / 2.0;
    let cyp = h / 2.0;
    quad_line(window, origin, px(0.0), cyp, w, px(1.5), rgba(0xc0392b60));
    quad_line(window, origin, cxp, px(0.0), px(1.5), h, rgba(0x27ae6060));

    // 示例几何：L 形板轮廓（主色）+ 圆（钢色）+ 中心线
    let m2p = |mx: f32, my: f32| Point {
        x: cxp + px(mx * UNIT),
        y: cyp - px(my * UNIT),
    };
    let mut outline = PathBuilder::stroke(px(2.0));
    outline.move_to(m2p(0.0, 0.0));
    outline.line_to(m2p(12.0, 0.0));
    outline.line_to(m2p(12.0, 4.0));
    outline.line_to(m2p(4.0, 4.0));
    outline.line_to(m2p(4.0, 10.0));
    outline.line_to(m2p(0.0, 10.0));
    outline.line_to(m2p(0.0, 0.0));
    if let Ok(p) = outline.build() {
        window.paint_path(p, rgb(0x0696d7));
    }
    let (ccx, ccy, r) = (17.0_f32, 2.0_f32, 2.5_f32);
    let mut circle = PathBuilder::stroke(px(1.5));
    for i in 0..=48 {
        let t = (i as f32) / 48.0 * std::f32::consts::TAU;
        let pt = m2p(ccx + r * t.cos(), ccy + r * t.sin());
        if i == 0 {
            circle.move_to(pt);
        } else {
            circle.line_to(pt);
        }
    }
    if let Ok(p) = circle.build() {
        window.paint_path(p, rgb(0x718c9e));
    }
    for seg in [
        (ccx - r - 1.5, ccy, ccx + r + 1.5, ccy),
        (ccx, ccy - r - 1.5, ccx, ccy + r + 1.5),
    ] {
        let mut cl = PathBuilder::stroke(px(1.0));
        cl.move_to(m2p(seg.0, seg.1));
        cl.line_to(m2p(seg.2, seg.3));
        if let Ok(p) = cl.build() {
            window.paint_path(p, rgba(0x9aa0a690));
        }
    }

    // 十字光标 + 拾取框
    if let Some(c) = cursor {
        let x = c.x - origin.x;
        let y = c.y - origin.y;
        quad_line(window, origin, px(0.0), y, w, px(1.0), rgba(0x1a1a1ab8));
        quad_line(window, origin, x, px(0.0), px(1.0), h, rgba(0x1a1a1ab8));
        let s = px(5.0);
        quad_line(window, origin, x - s, y - s, s * 2.0, px(1.0), rgba(0x1a1a1ae0));
        quad_line(window, origin, x - s, y + s, s * 2.0, px(1.0), rgba(0x1a1a1ae0));
        quad_line(window, origin, x - s, y - s, px(1.0), s * 2.0, rgba(0x1a1a1ae0));
        quad_line(window, origin, x + s, y - s, px(1.0), s * 2.0, rgba(0x1a1a1ae0));
    }
}

// ---------------------------------------------------------------------------
// 命令行
// ---------------------------------------------------------------------------

pub struct CommandLinePanel {
    input: Entity<InputState>,
    focus_handle: FocusHandle,
    _sub: Subscription,
}

impl CommandLinePanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入命令（LINE / CIRCLE / MOVE …），回车执行")
        });
        let sub = cx.subscribe_in(&input, window, |this, _, event: &InputEvent, window, cx| {
            if let InputEvent::PressEnter { .. } = event {
                let value = this.input.read(cx).value().to_string();
                if !value.trim().is_empty() {
                    state::run_command(&value, cx);
                    this.input.update(cx, |s, cx| s.set_value("", window, cx));
                }
            }
        });
        Self {
            input,
            focus_handle: cx.focus_handle(),
            _sub: sub,
        }
    }
}

impl BasePanel for CommandLinePanel {
    fn panel_name(&self) -> &'static str {
        "CommandLinePanel"
    }
}

impl Panel for CommandLinePanel {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "命令行"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("命令行".into())
    }
}

impl Focusable for CommandLinePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for CommandLinePanel {}

impl Render for CommandLinePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let Some(app) = state::app_state(cx) else {
            return div().child("");
        };
        let log = app.read(cx).model.log.clone();

        div()
            .v_flex()
            .size_full()
            .bg(theme.background)
            // 让面板句柄成为输入框（内部 frame 焦点句柄）的焦点树祖先：
            // 单键工具绑定用 contains_focused 判断焦点是否在命令行内
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .id("cmd-log")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_2()
                    .py_1()
                    .child(div().v_flex().gap_0p5().children(log.iter().rev().map(|line| {
                        div()
                            .font_family("Consolas")
                            .text_size(px(11.5))
                            .text_color(theme.muted_foreground)
                            .child(SharedString::from(line.clone()))
                    }))),
            )
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .border_t_1()
                    .border_color(theme.border)
                    .px_2()
                    .py_1()
                    .gap_1p5()
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.accent)
                            .child("命令:"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.0))
                            .child(Input::new(&self.input)),
                    ),
            )
    }
}

// ---------------------------------------------------------------------------
// 特性面板
// ---------------------------------------------------------------------------

pub struct PropertiesPanel {
    focus_handle: FocusHandle,
}

impl PropertiesPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self { focus_handle: cx.focus_handle() }
    }
}

impl BasePanel for PropertiesPanel {
    fn panel_name(&self) -> &'static str {
        "PropertiesPanel"
    }
}

impl Panel for PropertiesPanel {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "特性"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("特性".into())
    }
}

impl Focusable for PropertiesPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for PropertiesPanel {}

const PROP_ROWS: &[(&str, &str)] = &[
    ("颜色", "ByLayer"),
    ("图层", "Walls"),
    ("线型", "ByLayer"),
    ("线宽", "0.30 毫米"),
    ("透明度", "ByLayer"),
    ("打印样式", "ByLayer"),
];

const GEOM_ROWS: &[(&str, &str)] = &[
    ("起点 X", "0.0000"),
    ("起点 Y", "0.0000"),
    ("终点 X", "12.0000"),
    ("终点 Y", "4.0000"),
    ("长度", "12.0000"),
    ("角度", "18.435"),
    ("面积", "96.0000"),
];

impl Render for PropertiesPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .id("props-scroll")
            .v_flex()
            .size_full()
            .overflow_y_scroll()
            .bg(theme.background)
            .child(section_header("常规", &theme))
            .children(PROP_ROWS.iter().map(|(k, v)| prop_row(k, v, &theme)))
            .child(section_header("几何图形", &theme))
            .children(GEOM_ROWS.iter().map(|(k, v)| prop_row(k, v, &theme)))
    }
}

fn section_header(name: &str, theme: &Theme) -> impl IntoElement {
    div()
        .h_flex()
        .h(px(26.0))
        .px_2()
        .items_center()
        .border_b_1()
        .border_color(theme.border)
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.foreground)
        .child(SharedString::from(name))
}

fn prop_row(key: &str, value: &str, theme: &Theme) -> impl IntoElement {
    div()
        .h_flex()
        .items_center()
        .px_2()
        .py_0p5()
        .gap_1()
        .child(
            div()
                .w(px(64.0))
                .text_size(px(11.0))
                .text_color(theme.muted_foreground)
                .child(SharedString::from(key)),
        )
        .child(
            div()
                .flex_1()
                .h(px(20.0))
                .h_flex()
                .items_center()
                .px_1p5()
                .rounded_xs()
                .border_1()
                .border_color(theme.border)
                .bg(theme.input)
                .text_size(px(11.0))
                .text_color(theme.foreground)
                .child(SharedString::from(value)),
        )
}

// ---------------------------------------------------------------------------
// 图层面板
// ---------------------------------------------------------------------------

pub struct LayersPanel {
    selected: usize,
    focus_handle: FocusHandle,
}

const LAYERS: &[(u32, &str, bool)] = &[
    (0xf2f2f2, "0", true),
    (0x808080, "Defpoints", false),
    (0xe53935, "Walls", true),
    (0x1e88e5, "Doors", true),
    (0x43a047, "Dimensions", true),
    (0xfdd835, "Text", true),
    (0x8e24aa, "Furniture", false),
];

impl LayersPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self { selected: 2, focus_handle: cx.focus_handle() }
    }
}

impl BasePanel for LayersPanel {
    fn panel_name(&self) -> &'static str {
        "LayersPanel"
    }
}

impl Panel for LayersPanel {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "图层"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("图层".into())
    }
}

impl Focusable for LayersPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for LayersPanel {}

impl Render for LayersPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let selected = self.selected;
        div()
            .v_flex()
            .size_full()
            .bg(theme.background)
            .py_1()
            .children(LAYERS.iter().enumerate().map(|(ix, (color, name, on))| {
                let sel = ix == selected;
                div()
                    .id(ElementId::Name(format!("layer-{ix}").into()))
                    .h_flex()
                    .h(px(24.0))
                    .px_2()
                    .gap_2()
                    .items_center()
                    .when(sel, |el| {
                        el.bg(rgba(0x0696d72e))
                            .border_l_2()
                            .border_color(theme.accent)
                    })
                    .when(!sel, |el| el.hover(|s| s.bg(rgba(0xffffff0d))))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.selected = ix;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .size(px(10.0))
                            .rounded_xs()
                            .border_1()
                            .border_color(theme.border)
                            .bg(rgb(*color))
                            .flex_shrink_0(),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(11.5))
                            .text_color(if *on {
                                theme.foreground
                            } else {
                                theme.muted_foreground
                            })
                            .child(SharedString::from(*name)),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(if *on {
                                theme.foreground
                            } else {
                                theme.muted_foreground
                            })
                            .opacity(if *on { 1.0 } else { 0.45 })
                            .child(if *on { "●" } else { "○" }),
                    )
            }))
    }
}

// ---------------------------------------------------------------------------
// 工具选项板
// ---------------------------------------------------------------------------

pub struct ToolPalettePanel {
    focus_handle: FocusHandle,
}

const PALETTE_BLOCKS: &[&str] = &["Chair", "Door 900", "Window 1200", "Tree Plan", "Sink", "Toilet"];
const PALETTE_HATCHES: &[&str] = &["ANSI31", "ANSI37", "SOLID", "GRAVEL"];

impl ToolPalettePanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self { focus_handle: cx.focus_handle() }
    }
}

impl BasePanel for ToolPalettePanel {
    fn panel_name(&self) -> &'static str {
        "ToolPalettePanel"
    }
}

impl Panel for ToolPalettePanel {
    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "工具选项板"
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some("工具选项板".into())
    }
}

impl Focusable for ToolPalettePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for ToolPalettePanel {}

impl Render for ToolPalettePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let mut root = div()
            .id("palette-scroll")
            .v_flex()
            .size_full()
            .overflow_y_scroll()
            .bg(theme.background)
            .child(section_header("块", &theme));

        for name in PALETTE_BLOCKS {
            root = root.child(palette_row(name, "blocks/block.svg", &theme));
        }
        root = root.child(section_header("图案填充", &theme));
        for name in PALETTE_HATCHES {
            root = root.child(palette_row(name, "hatch/hatch_solid.svg", &theme));
        }
        root
    }
}

fn palette_row(name: &str, icon: &'static str, theme: &Theme) -> Stateful<gpui_kit::Div> {
    let cmd = format!("insert {name}");
    div()
        .id(ElementId::Name(format!("pal-{name}").into()))
        .h_flex()
        .h(px(24.0))
        .px_2()
        .gap_2()
        .items_center()
        .cursor_pointer()
        .hover(|s| s.bg(rgba(0x0696d71e)))
        .on_click(move |_, _, cx| state::run_command(&cmd, cx))
        .child(img(icons::icon(icon)).size(px(14.0)).flex_shrink_0())
        .child(
            div()
                .text_size(px(11.5))
                .text_color(theme.foreground)
                .child(SharedString::from(name)),
        )
}

// ---------------------------------------------------------------------------
// 组装：把全部面板装修入 DockArea（在 DockArea 实体构造闭包内调用）
// ---------------------------------------------------------------------------

pub fn build_panels(
    window: &mut Window,
    cx: &mut App,
) -> (
    Entity<CanvasPanel>,
    Entity<CommandLinePanel>,
    Entity<PropertiesPanel>,
    Entity<LayersPanel>,
    Entity<ToolPalettePanel>,
) {
    let canvas = cx.new(|cx| CanvasPanel::new(cx));
    let command_line = cx.new(|cx| CommandLinePanel::new(window, cx));
    let properties = cx.new(|cx| PropertiesPanel::new(cx));
    let layers = cx.new(|cx| LayersPanel::new(cx));
    let palette = cx.new(|cx| ToolPalettePanel::new(cx));
    (canvas, command_line, properties, layers, palette)
}

pub fn install_docks(
    area: &mut DockArea,
    canvas: Entity<CanvasPanel>,
    command_line: Entity<CommandLinePanel>,
    properties: Entity<PropertiesPanel>,
    layers: Entity<LayersPanel>,
    palette: Entity<ToolPalettePanel>,
    window: &mut Window,
    cx: &mut Context<DockArea>,
) {
    area.set_center(
        DockLayout::tabs().panel_view(panel_handle(canvas), cx),
        window,
        cx,
    );
    area.add_panel_view(panel_handle(palette), DockPlacement::Left, Some(px(240.0)), window, cx);
    area.add_panel_view(panel_handle(layers), DockPlacement::Left, None, window, cx);
    area.add_panel_view(panel_handle(properties), DockPlacement::Right, Some(px(300.0)), window, cx);
    area.add_panel_view(panel_handle(command_line), DockPlacement::Bottom, Some(px(150.0)), window, cx);
}
