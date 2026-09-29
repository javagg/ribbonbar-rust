use std::sync::Arc;
use gpui_kit::component::Icon;
use gpui_kit::component::StyledExt;
use gpui_kit::{div, img, px, rgba, App, AppContext as _, Bounds, Context, Image, ImageFormat, IntoElement, Render, Styled as _, ParentElement as _, Window,
               WindowBounds, WindowOptions, TitlebarOptions, size};
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/icons_manifest.rs"));

fn icon(rel: &str) -> Arc<Image> {
    let bytes = CAD_ICONS
        .iter()
        .find(|(name, _)| *name == rel)
        .map(|(_, b)| *b)
        .expect(rel);
    let mut out = bytes.to_vec();
    out = replace_ci(&out, b"#b4b6b9", b"#e3e5e7");
    out = replace_ci(&out, b"#B4B6B9", b"#e3e5e7");
    Arc::new(Image::from_bytes(ImageFormat::Svg, out))
}

fn replace_ci(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let w = &bytes[i..];
        if w.len() >= from.len() && w[..from.len()].eq_ignore_ascii_case(from) {
            out.extend_from_slice(to);
            i += from.len();
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

struct Test;
impl Render for Test {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let names = [
            "point.svg",
            "line.svg",
            "constrain/auto.svg",
            "constrain/distance.svg",
            "constrain/coincident.svg",
            "circle/circle_cr.svg",
            "layers/panel.svg",
        ];
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(rgba(0x1a1a1aff))
            .size_full()
            .text_color(rgba(0xffffffff))
            .child(
                div().flex().flex_row().gap_3().children(names.map(|n| {
                    div().flex().flex_col().items_center().gap_1()
                        .child(div().text_size(px(9.)).child(n))
                        .child(img(icon(n)).size(px(32.)))
                })),
            )
            .child(
                div().flex().flex_row().gap_3().children([
                    svg_row("svg cad/dim_linear", "cad/dim_linear.svg"),
                    svg_row("svg cad/line", "cad/line.svg"),
                    svg_row("svg lucide check", "icons/check.svg"),
                ]),
            )
    }
}

fn svg_row(label: &'static str, path: &'static str) -> gpui_kit::AnyElement {
    div().flex().flex_col().items_center().gap_1()
        .child(div().text_size(px(9.)).child(label))
        .child(Icon::empty().path(path).size(px(20.)))
        .into_any_element()
}

struct TestAssets;

impl gpui_kit::AssetSource for TestAssets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(CAD_ICONS
            .iter()
            .find(|(name, _)| *name == path.strip_prefix("cad/").unwrap_or(path))
            .map(|(_, b)| std::borrow::Cow::Borrowed(*b)))
    }

    fn list(&self, _path: &str) -> gpui_kit::Result<Vec<gpui_kit::SharedString>> {
        Ok(vec![])
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(TestAssets)
        .run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(700.), px(160.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions { title: Some("icon test".into()), appears_transparent: false, traffic_light_position: None }),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Test)).unwrap();
        });
}
