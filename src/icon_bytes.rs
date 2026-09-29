//! 图标字节处理（纯逻辑，UI 无关）：编译期嵌入 assets/icons 的 SVG 清单、
//! 语义色重映射（灰→文字色、蓝→强调色）、栅格尺寸统一。
//! gpui 版在其上缓存 `Image`，egui 版用它 rasterize 成纹理。

use std::borrow::Cow;

include!(concat!(env!("OUT_DIR"), "/cad_icons.rs"));

pub static LIGHT_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_light_mode(light: bool) {
    LIGHT_MODE.store(light, std::sync::atomic::Ordering::SeqCst);
}

pub fn is_light() -> bool {
    LIGHT_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// 大图标最多 39px，HiDPI 150% 下约需 59 物理像素，统一重绘到 64px。
const RASTER_SIZE: &str = "64";

pub fn find_icon(rel: &str) -> Option<&'static [u8]> {
    CAD_ICONS
        .iter()
        .find(|(name, _)| *name == rel)
        .map(|(_, bytes)| *bytes)
}

/// 取已重映射、已统一尺寸的 SVG 字节（带缺失兜底）。
pub fn prepared(rel: &str) -> Cow<'static, [u8]> {
    let bytes = find_icon(rel).unwrap_or_else(|| {
        eprintln!("[icons] missing icon: {rel}");
        b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"64\" height=\"64\"></svg>" as &[u8]
    });
    Cow::Owned(ensure_size(&recolor(bytes)))
}

/// 把第一个 <svg …> 标签里的 width/height 统一改成 64（缺失则注入）。
fn ensure_size(svg: &[u8]) -> Vec<u8> {
    const OPEN: &[u8] = b"<svg";
    let Some(start) = svg.windows(OPEN.len()).position(|w| w == OPEN) else {
        return svg.to_vec();
    };
    let Some(off) = svg[start..].iter().position(|&b| b == b'>') else {
        return svg.to_vec();
    };
    let end = start + off;
    let tag = String::from_utf8_lossy(&svg[start..end]).to_string();
    let tag = set_attr(&tag, "width");
    let tag = set_attr(&tag, "height");

    let mut out = Vec::with_capacity(svg.len() + 32);
    out.extend_from_slice(&svg[..start]);
    out.extend_from_slice(tag.as_bytes());
    out.push(b'>');
    out.extend_from_slice(&svg[end + 1..]);
    out
}

fn set_attr(tag: &str, attr: &str) -> String {
    // 前导空格防止误匹配 stroke-width 等属性
    let needle = format!(" {attr}=\"");
    if let Some(i) = tag.find(&needle) {
        if let Some(j) = tag[i + needle.len()..].find('"') {
            let close = i + needle.len() + j + 1;
            // 替换区间含 needle 的前导空格，故补回
            return format!("{} {attr}=\"{RASTER_SIZE}\"{}", &tag[..i], &tag[close..]);
        }
    }
    format!("{tag} {attr}=\"{RASTER_SIZE}\"")
}

/// (原图色, 暗色主题映射, 亮色主题映射)
const COLOR_MAP: &[(&str, &str, &str)] = &[
    ("#b4b6b9", "#e3e5e7", "#3a3a3a"),
    ("#e0e0e0", "#e3e5e7", "#3a3a3a"),
    ("#eeeeee", "#d4d6d8", "#4a4a4a"),
    ("#d1d1d1", "#c2c4c6", "#565656"),
    ("#6db7ed", "#4db3e8", "#0277bd"),
    ("#4cc9f0", "#4db3e8", "#0277bd"),
    ("#0099e5", "#3ba3dd", "#0268a8"),
    ("#2496ed", "#3ba3dd", "#0268a8"),
    ("#1565c0", "#2f8ec9", "#02559b"),
    ("#505050", "#9aa0a6", "#6b7075"),
    ("#888888", "#9aa0a6", "#6b7075"),
    ("#777777", "#8a9096", "#6b7075"),
    ("#526b80", "#93a8bc", "#5f7484"),
    ("#718c9e", "#93a8bc", "#5f7484"),
    ("#1a1a1a", "#1a1a1a", "#ffffff"),
    ("#000000", "#1a1a1a", "#ffffff"),
];

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

fn recolor(svg: &[u8]) -> Vec<u8> {
    let light = is_light();
    let mut out = svg.to_vec();
    for &(from, dark, light_to) in COLOR_MAP {
        let to = if light { light_to } else { dark };
        if from.eq_ignore_ascii_case(to) {
            continue;
        }
        out = replace_ci(&out, from.as_bytes(), to.as_bytes());
    }
    out
}
