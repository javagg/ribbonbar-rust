//! gpui 版的全局状态封装：把纯逻辑的 [`CadModel`] 挂到 gpui 的
//! Entity/Global 体系上（事件驱动的重绘）。命令入口保持不变，
//! 供 ribbon / panels / workspace 调用。

use gpui_kit::{App, AppContext as _, Entity, EventEmitter, Global};

pub use cad_demo::model::{CadModel, LayerInfo, RibbonSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Updated,
}

pub struct AppState {
    pub model: CadModel,
}

impl EventEmitter<AppEvent> for AppState {}

pub struct AppModel(pub Entity<AppState>);

impl Global for AppModel {}

pub fn init(cx: &mut App) -> Entity<AppState> {
    let state = cx.new(|_| AppState { model: CadModel::new() });
    cx.set_global(AppModel(state.clone()));
    state
}

pub fn app_state(cx: &App) -> Option<Entity<AppState>> {
    cx.try_global::<AppModel>().map(|m| m.0.clone())
}

/// 渲染期取快照（owned）。
pub fn ribbon_snapshot(cx: &App) -> RibbonSnapshot {
    app_state(cx)
        .map(|e| e.read(cx).model.snapshot())
        .unwrap_or_else(|| CadModel::new().snapshot())
}

// ---------------------------------------------------------------------------
// 命令入口（包装 CadModel，并触发重绘）
// ---------------------------------------------------------------------------

macro_rules! forward {
    ($pub:ident ($($arg:ident : $ty:ty),*) $method:ident) => {
        pub fn $pub($($arg: $ty),*, cx: &mut App) {
            let Some(entity) = app_state(cx) else { return };
            entity.update(cx, |state, cx| {
                state.model.$method($($arg),*);
                cx.emit(AppEvent::Updated);
            });
        }
    };
}

forward!(run_command(cmd: &str) run_command);
forward!(run_tool(cmd: &'static str) run_tool);
forward!(toggle(kind: &'static str) toggle);
forward!(layer_toggle(index: usize, kind: &'static str) layer_toggle);
forward!(set_cursor(pos: Option<(f64, f64)>) set_cursor);
forward!(set_ext_sub(sub: Option<String>) set_ext_sub);
forward!(set_ext_open(key: Option<String>) set_ext_open);
forward!(set_app_menu_open(open: bool) set_app_menu_open);

pub fn select_dropdown(dd: &'static str, cmd: &'static str, cx: &mut App) {
    let Some(entity) = app_state(cx) else { return };
    entity.update(cx, |state, cx| {
        state.model.select_dropdown(dd, cmd);
        cx.emit(AppEvent::Updated);
    });
}

#[allow(dead_code)]
pub fn set_dropdown_current(dd: &'static str, cmd: &'static str, cx: &mut App) {
    let Some(entity) = app_state(cx) else { return };
    entity.update(cx, |state, cx| {
        state.model.set_dropdown_current(dd, cmd);
        cx.emit(AppEvent::Updated);
    });
}

pub fn layer_select(name: String, cx: &mut App) {
    let Some(entity) = app_state(cx) else { return };
    entity.update(cx, |state, cx| {
        state.model.layer_select(name);
        cx.emit(AppEvent::Updated);
    });
}

pub fn undo(cx: &mut App) {
    let Some(entity) = app_state(cx) else { return };
    entity.update(cx, |state, cx| {
        state.model.undo();
        cx.emit(AppEvent::Updated);
    });
}

pub fn redo(cx: &mut App) {
    let Some(entity) = app_state(cx) else { return };
    entity.update(cx, |state, cx| {
        state.model.redo();
        cx.emit(AppEvent::Updated);
    });
}

pub fn undo_many(n: usize, cx: &mut App) {
    let Some(entity) = app_state(cx) else { return };
    entity.update(cx, |state, cx| {
        state.model.undo_many(n);
        cx.emit(AppEvent::Updated);
    });
}

pub fn redo_many(n: usize, cx: &mut App) {
    let Some(entity) = app_state(cx) else { return };
    entity.update(cx, |state, cx| {
        state.model.redo_many(n);
        cx.emit(AppEvent::Updated);
    });
}
