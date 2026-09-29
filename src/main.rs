//! OpenCAD Demo — 基于 gpui-kit 0.7.0（GPUI 社区生态）的 CAD 桌面应用 UI 框架演示。
//!
//! 特性：
//! - RibbonBar（标签页 / 分组 / 大工具 / 分裂按钮 / 小工具列），图标取自 OpenCADStudio
//!   并在加载时做主题色重映射（用户后期可整体替换图标资源）。
//! - DockPanel 布局：左（工具选项板 / 图层）、右（特性）、下（命令行）四个可拖拽
//!   停靠面板 + 中央模型视口，支持拖动标签、分合、缩放与尺寸调整。

mod icons;
mod panels;
mod ribbon;
mod state;
mod workspace;

use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{px, rgb, rgba, size, App, AppContext as _, Bounds, TitlebarOptions, WindowBounds, WindowOptions};

fn main() {
    let app = gpui_kit::application().with_assets(icons::CadAssetSource);
    app.run(move |cx: &mut App| {
        gpui_kit::init(cx);
        state::init(cx);
        setup_theme(cx);

        let bounds = Bounds::centered(None, size(px(1520.0), px(940.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("OpenCAD Demo — gpui-kit".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| workspace::Workspace::new(window, cx))
        })
        .expect("Failed to open window");
    });
}

/// 仿 OpenCADStudio "Fusion Black" 主题：#1A1A1A 铬底、#0696D7 主色。
fn setup_theme(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    icons::set_light_mode(false);
    Theme::update(cx, |theme| {
        theme.radius = px(3.0);
        theme.radius_lg = px(6.0);
        theme.font_size = px(13.0);
        theme.focus_ring = false;
        theme.shadow = false;

        let c = &mut theme.colors;
        c.background = rgb(0x1a1a1a).into();
        c.foreground = rgb(0xe8e8e8).into();
        c.border = rgb(0x2e2e2e).into();

        c.accent = rgb(0x0696d7).into();
        c.accent_foreground = rgb(0xffffff).into();
        c.primary = rgb(0x0696d7).into();
        c.primary_foreground = rgb(0xffffff).into();
        c.primary_hover = rgb(0x1ba7e5).into();
        c.primary_active = rgb(0x0584bd).into();

        c.secondary = rgb(0x262626).into();
        c.secondary_foreground = rgb(0xe8e8e8).into();
        c.secondary_hover = rgb(0x303030).into();
        c.muted = rgb(0x232323).into();
        c.muted_foreground = rgb(0x9c9c9c).into();

        c.popover = rgb(0x242424).into();
        c.popover_foreground = rgb(0xe8e8e8).into();
        c.input = rgb(0x212121).into();
        c.ring = rgb(0x0696d7).into();
        c.selection = rgba(0x0696d74d).into();

        c.list = rgb(0x1f1f1f).into();
        c.list_hover = rgb(0x2a2a2a).into();
        c.list_active = rgb(0x0696d7).into();
        c.list_active_border = rgb(0x0696d7).into();

        c.sidebar = rgb(0x1f1f1f).into();
        c.sidebar_border = rgb(0x2e2e2e).into();
        c.sidebar_accent = rgb(0x2a2a2a).into();
        c.sidebar_accent_foreground = rgb(0xe8e8e8).into();
        c.sidebar_foreground = rgb(0xd8d8d8).into();
        c.sidebar_primary = rgb(0x0696d7).into();
        c.sidebar_primary_foreground = rgb(0xffffff).into();

        c.tab = rgb(0x202020).into();
        c.tab_active = rgb(0x2c2c2c).into();
        c.tab_active_foreground = rgb(0xf2f2f2).into();
        c.tab_bar = rgb(0x1a1a1a).into();
        c.tab_bar_segmented = rgb(0x232323).into();
        c.tab_foreground = rgb(0xa8a8a8).into();

        c.title_bar = rgb(0x1a1a1a).into();
        c.title_bar_border = rgb(0x2e2e2e).into();
        c.status_bar = rgb(0x1f1f1f).into();
        c.status_bar_border = rgb(0x2e2e2e).into();

        c.button = rgb(0x232323).into();
        c.button_foreground = rgb(0xe0e0e0).into();
        c.button_hover = rgb(0x2c2c2c).into();
        c.button_active = rgb(0x333333).into();

        c.drop_target = rgba(0x0696d755).into();
        c.drag_border = rgb(0x0696d7).into();
        c.scrollbar = rgb(0x242424).into();
        c.scrollbar_thumb = rgb(0x3a3a3a).into();
        c.scrollbar_thumb_hover = rgb(0x4a4a4a).into();
    });
}
