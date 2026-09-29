//! gpui 版图标缓存：在共享的 [`cad_demo::icon_bytes`]（重映射 + 尺寸统一）
//! 之上缓存 gpui `Image`，并提供 `cad/` 前缀的 AssetSource。

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use gpui_kit::{AssetSource, Image, ImageFormat, SharedString};

use cad_demo::icon_bytes::{self, prepared};

static ICON_CACHE: OnceLock<Mutex<HashMap<String, Arc<Image>>>> = OnceLock::new();

/// 取一个 CAD 图标（已按主题重映射、重设栅格尺寸），带缓存。
pub fn icon(rel: &str) -> Arc<Image> {
    let mut cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
    if let Some(img) = cache.get(rel) {
        return img.clone();
    }
    let processed = prepared(rel).into_owned();
    let img = Arc::new(Image::from_bytes(ImageFormat::Svg, processed));
    cache.insert(rel.to_string(), img.clone());
    img
}

/// 主题切换后清空 gpui Image 缓存（字节级重映射由 icon_bytes 处理）。
pub fn set_light_mode(light: bool) {
    icon_bytes::set_light_mode(light);
    ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap().clear();
}

/// 应用资源源：`cad/` 前缀走内置图标（供 svg().path() 的单色用法），
/// 其余路径回退到 gpui-kit 自带资源（Lucide 图标等）。
pub struct CadAssetSource;

impl AssetSource for CadAssetSource {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<Cow<'static, [u8]>>> {
        if let Some(rel) = path.strip_prefix("cad/") {
            return Ok(Some(prepared(rel)));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui_kit::Result<Vec<SharedString>> {
        let mut own: Vec<SharedString> = icon_bytes::CAD_ICONS
            .iter()
            .map(|(name, _)| SharedString::from(format!("cad/{name}")))
            .collect();
        own.extend(gpui_kit::assets::Assets.list(path)?);
        Ok(own)
    }
}
