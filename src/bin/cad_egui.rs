//! OpenCAD Demo — egui / eframe / egui_tiles 版本。
//!
//! 与 gpui 版（cad_demo）共享同一核心（cad_demo lib）：
//! - `model::CadModel`    应用状态模型（last_cmd / active_tool / undo-redo / 图层）
//! - `ribbon_data`        Ribbon 数据模型 + 自适应降级算法（decide_levels）
//! - `icon_bytes`         SVG 图标重映射（语义色 → 主题色）
//!
//! 布局：顶部 RibbonBar（自绘）+ egui_tiles 停靠区（可拖拽/分合/标签化）+ 底部状态栏。

use std::collections::HashMap;

use eframe::egui;
use egui::{Color32, Sense, TextureHandle, Vec2};

use cad_demo::model::{CadModel, RibbonSnapshot};
use cad_demo::ribbon_data::*;

// ── 主题常量（Fusion Black，同 gpui 版）────────────────────────────────────

fn c(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}
const ACCENT: Color32 = Color32::from_rgb(0x06, 0x96, 0xd7);
const PAPER: Color32 = Color32::from_rgb(0xf6, 0xf6, 0xf6);
const GRID_MINOR: Color32 = Color32::from_black_alpha(18);
const GRID_MAJOR: Color32 = Color32::from_black_alpha(38);

fn accent_active() -> Color32 {
    Color32::from_rgba_unmultiplied(6, 150, 215, 75)
}

// ── 停靠面板种类 ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Pane {
    Viewport,
    CommandLine,
    Properties,
    Layers,
    Palette,
}

impl Pane {
    fn title(self) -> &'static str {
        match self {
            Pane::Viewport => "模型",
            Pane::CommandLine => "命令行",
            Pane::Properties => "特性",
            Pane::Layers => "图层",
            Pane::Palette => "工具选项板",
        }
    }
}

// ── 即时 UI 状态 ────────────────────────────────────────────────────────────

#[derive(Default)]
struct UiState {
    tab: usize,
    collapsed: bool,
    cmd_input: String,
}

// ── 图标纹理（resvg 直接栅格化共享模块输出的重映射字节）──────────────────

fn icon_tex(
    ctx: &egui::Context,
    cache: &mut HashMap<String, TextureHandle>,
    rel: &str,
) -> TextureHandle {
    if let Some(t) = cache.get(rel) {
        return t.clone();
    }
    let bytes = cad_demo::icon_bytes::prepared(rel);
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&bytes, &opt).expect("svg parse failed");
    let size = tree.size();
    let mut pixmap = tiny_skia::Pixmap::new(64, 64).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(64.0 / size.width(), 64.0 / size.height()),
        &mut pixmap.as_mut(),
    );
    // tiny-skia 输出预乘 alpha，egui 需要非预乘
    let mut rgba = Vec::with_capacity(64 * 64 * 4);
    for p in pixmap.pixels() {
        let c = p.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    let handle = ctx.load_texture(
        format!("icon:{rel}"),
        egui::ColorImage::from_rgba_unmultiplied([64, 64], &rgba),
        egui::TextureOptions::LINEAR,
    );
    cache.insert(rel.to_string(), handle.clone());
    handle
}

fn img(tex: &TextureHandle, size: f32) -> egui::Image<'static> {
    egui::Image::from_texture((tex.id(), Vec2::splat(size)))
}

fn tip_text(label: &str, cmd: &str) -> String {
    format!("{label}\n命令: {cmd}")
}

// ── 应用 ────────────────────────────────────────────────────────────────────

struct CadEgui {
    model: CadModel,
    icons: HashMap<String, TextureHandle>,
    tree: egui_tiles::Tree<Pane>,
    ui: UiState,
}

impl CadEgui {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);

        let mut tiles = egui_tiles::Tiles::<Pane>::default();
        let viewport = tiles.insert_pane(Pane::Viewport);
        let command_line = tiles.insert_pane(Pane::CommandLine);
        let properties = tiles.insert_pane(Pane::Properties);
        let layers = tiles.insert_pane(Pane::Layers);
        let palette = tiles.insert_pane(Pane::Palette);

        let center =
            tiles.insert_container(egui_tiles::Container::new_vertical(vec![viewport, command_line]));
        let left = tiles.insert_container(egui_tiles::Container::new_tabs(vec![palette, layers]));
        let root = tiles.insert_container(egui_tiles::Container::new_horizontal(vec![
            left,
            center,
            properties,
        ]));
        let tree = egui_tiles::Tree::new("cad_dock", root, tiles);

        Self {
            model: CadModel::new(),
            icons: HashMap::new(),
            tree,
            ui: UiState::default(),
        }
    }

    // ── 顶部 RibbonBar ──────────────────────────────────────────────────────

    fn ribbon_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("ribbon")
            .frame(
                egui::Frame::new()
                    .fill(c(0x1a1a1a))
                    .stroke(egui::Stroke::new(1.0, c(0x2e2e2e)))
                    .inner_margin(egui::Margin::symmetric(6, 0)),
            )
            .show(ui, |ui| {
                let snap = self.model.snapshot();
                let tab_ix = self.ui.tab;

                // ── 标签条 ──
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.set_height(30.0);
                    {
                        let Self { model, icons, .. } = self;
                        quick_button(ui, icons, "qa-new", "ui/doc_new.svg", "新建", "new", model);
                        quick_button(ui, icons, "qa-open", "ui/folder_open.svg", "打开", "open", model);
                        quick_button(ui, icons, "qa-save", "ui/save.svg", "保存", "save", model);
                        quick_button(ui, icons, "qa-print", "ui/print.svg", "打印", "plot", model);
                        let undo_labels: Vec<String> = model.undo_stack.iter().rev().cloned().collect();
                        let redo_labels: Vec<String> = model.redo_stack.iter().rev().cloned().collect();
                        history_button(
                            ui, icons, "qa-undo", "ui/undo.svg", "放弃", &undo_labels, true, model,
                        );
                        history_button(
                            ui, icons, "qa-redo", "ui/redo.svg", "重做", &redo_labels, false, model,
                        );
                    }
                    ui.separator();
                    let tabs = tabs();
                    for (ix, t) in tabs.iter().enumerate() {
                        let active = ix == tab_ix;
                        let text = egui::RichText::new(t.name).size(12.5).color(if active {
                            c(0xf2f2f2)
                        } else {
                            c(0x9c9c9c)
                        });
                        if ui
                            .add(
                                egui::Button::new(text)
                                    .fill(if active { c(0x2c2c2c) } else { Color32::TRANSPARENT })
                                    .stroke(if active {
                                        egui::Stroke::new(1.5, ACCENT)
                                    } else {
                                        egui::Stroke::NONE
                                    }),
                            )
                            .clicked()
                        {
                            self.ui.tab = ix;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(
                                egui::RichText::new(if self.ui.collapsed { "▾" } else { "▴" }).size(10.0),
                            )
                            .clicked()
                        {
                            self.ui.collapsed = !self.ui.collapsed;
                        }
                    });
                });
                ui.separator();

                // ── 内容区：自适应降级 ──
                if self.ui.collapsed {
                    return;
                }
                let tabs = tabs();
                let tab = &tabs[tab_ix.min(tabs.len() - 1)];
                let available = ui.available_width() - 20.0;
                let widths: Vec<GroupWidths> = tab.groups.iter().map(group_widths).collect();
                let levels = decide_levels(&widths, available);

                egui::ScrollArea::horizontal()
                    .id_salt("ribbon-scroll")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for ((gi, group), level) in tab.groups.iter().enumerate().zip(levels) {
                        egui::Frame::new()
                            .inner_margin(egui::Margin::symmetric(5, 2))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    let key = format!("t{}g{}", tab_ix, gi);
                                    // 内容行：组内工具横排
                                    ui.horizontal(|ui| {
                                        match level {
                                            Level::Full => {
                                                for (ii, item) in group.items.iter().enumerate() {
                                                    let ikey = format!("{key}i{ii}");
                                                    render_item(
                                                        ui,
                                                        &mut self.model,
                                                        &mut self.icons,
                                                        &ikey,
                                                        item,
                                                        &snap,
                                                        &mut self.ui,
                                                    );
                                                }
                                            }
                                            Level::Compact => {
                                                for (ii, item) in group.items.iter().enumerate() {
                                                    render_compact(
                                                        ui,
                                                        &mut self.model,
                                                        &mut self.icons,
                                                        &format!("{key}c{ii}"),
                                                        item,
                                                    );
                                                }
                                            }
                                            Level::Flyout => {
                                                flyout_button(
                                                    ui,
                                                    &mut self.model,
                                                    &mut self.icons,
                                                    &key,
                                                    group,
                                                    &snap,
                                                    &mut self.ui,
                                                );
                                            }
                                        }
                                    });

                                    // 组标题（带扩展面板的组是"标题 ▾"触发器）
                                    ui.add_space(2.0);
                                    match (group.ext, level) {
                                        (Some(ext), _) => {
                                            ext_title(
                                                ui,
                                                &mut self.model,
                                                &mut self.icons,
                                                &key,
                                                group,
                                                ext,
                                                &snap,
                                            );
                                        }
                                        _ => {
                                            let title = egui::RichText::new(group.name)
                                                .size(10.0)
                                                .color(if level == Level::Flyout {
                                                    ACCENT
                                                } else {
                                                    c(0x9c9c9c)
                                                });
                                            ui.vertical_centered_justified(|ui| {
                                                ui.label(title);
                                            });
                                        }
                                    }
                                });
                            });
                        ui.separator();
                    }
                    });
                    });
                ui.add_space(3.0);
            });
    }

    // ── 底部状态栏 ──────────────────────────────────────────────────────────

    fn status_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(c(0x1f1f1f))
                    .stroke(egui::Stroke::new(1.0, c(0x2e2e2e)))
                    .inner_margin(egui::Margin::symmetric(8, 3)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_height(20.0);
                    let coords = self
                        .model
                        .cursor
                        .map(|(x, y)| format!("X {:>10.4}   Y {:>10.4}", x, y))
                        .unwrap_or_else(|| "X           -           Y           -".into());
                    ui.label(
                        egui::RichText::new(coords)
                            .font(egui::FontId::monospace(11.0))
                            .color(ACCENT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for (label, icon, kind, on) in [
                            ("LWT", "status/lwt.svg", "lwt", self.model.lwt),
                            ("POLAR", "status/polar.svg", "polar", self.model.polar),
                            ("ORTHO", "status/ortho.svg", "ortho", self.model.ortho),
                            ("OSNAP", "status/osnap.svg", "osnap", self.model.osnap),
                        ] {
                            let tex = icon_tex(ui.ctx(), &mut self.icons, icon);
                            let fill = if on { accent_active() } else { Color32::TRANSPARENT };
                            if ui
                                .add(
                                    egui::Button::image_and_text(
                                        img(&tex, 13.0),
                                        egui::RichText::new(label).size(9.5),
                                    )
                                    .fill(fill),
                                )
                                .clicked()
                            {
                                self.model.toggle(kind);
                            }
                        }
                    });
                });
            });
    }
}

impl eframe::App for CadEgui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ribbon_panel(ui);
        self.status_panel(ui);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(c(0x242424)))
            .show(ui, |ui| {
                let Self {
                    model,
                    icons,
                    tree,
                    ui: st,
                } = self;
                let mut behavior = DockBehavior { model, icons, ui: st };
                tree.ui(&mut behavior, ui);
            });
    }
}

// ── 停靠行为 ────────────────────────────────────────────────────────────────

struct DockBehavior<'a> {
    model: &'a mut CadModel,
    icons: &'a mut HashMap<String, TextureHandle>,
    ui: &'a mut UiState,
}

impl egui_tiles::Behavior<Pane> for DockBehavior<'_> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Viewport => viewport_ui(ui, self.model),
            Pane::CommandLine => command_line_ui(ui, self.model, &mut self.ui.cmd_input),
            Pane::Properties => properties_ui(ui),
            Pane::Layers => layers_ui(ui, self.model),
            Pane::Palette => palette_ui(ui, self.model, self.icons),
        }
        Default::default()
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        egui::RichText::new(pane.title()).size(11.5).into()
    }

    fn tab_bar_color(&self, _visuals: &egui::Visuals) -> Color32 {
        c(0x1f1f1f)
    }

    fn tab_bg_color(
        &self,
        _visuals: &egui::Visuals,
        _tiles: &egui_tiles::Tiles<Pane>,
        _tile_id: egui_tiles::TileId,
        state: &egui_tiles::TabState,
    ) -> Color32 {
        if state.active {
            c(0x242424)
        } else {
            Color32::TRANSPARENT
        }
    }

    fn tab_outline_stroke(
        &self,
        _visuals: &egui::Visuals,
        _tiles: &egui_tiles::Tiles<Pane>,
        _tile_id: egui_tiles::TileId,
        _state: &egui_tiles::TabState,
    ) -> egui::Stroke {
        egui::Stroke::new(1.0, c(0x2e2e2e))
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        1.0
    }
}

// ── Ribbon 小部件 ───────────────────────────────────────────────────────────

fn tool_fill(active: bool) -> Color32 {
    if active {
        accent_active()
    } else {
        Color32::TRANSPARENT
    }
}

fn quick_button(
    ui: &mut egui::Ui,
    icons: &mut HashMap<String, TextureHandle>,
    _id: &str,
    icon: &'static str,
    tip: &'static str,
    cmd: &'static str,
    model: &mut CadModel,
) {
    let tex = icon_tex(ui.ctx(), icons, icon);
    let r = ui
        .add(
            egui::Button::image(img(&tex, 17.0))
                .fill(Color32::TRANSPARENT)
                .min_size(Vec2::splat(26.0)),
        )
        .on_hover_text(tip_text(tip, cmd));
    if r.clicked() {
        model.run_command(cmd);
    }
}

fn history_button(
    ui: &mut egui::Ui,
    icons: &mut HashMap<String, TextureHandle>,
    id: &str,
    icon: &'static str,
    tip: &'static str,
    labels: &[String],
    is_undo: bool,
    model: &mut CadModel,
) {
    let tex = icon_tex(ui.ctx(), icons, icon);
    let face = ui
        .add(
            egui::Button::image(img(&tex, 17.0))
                .fill(Color32::TRANSPARENT)
                .min_size(Vec2::splat(26.0)),
        )
        .on_hover_text(tip_text(tip, if is_undo { "U" } else { "REDO" }));
    if face.clicked() {
        if is_undo {
            model.undo();
        } else {
            model.redo();
        }
    }
    // 历史箭头（栈空时隐藏）
    if !labels.is_empty() {
        let owned: Vec<String> = labels.to_vec();
        ui.push_id(id, |ui| {
            ui.menu_button(egui::RichText::new("▾").size(8.0), |ui| {
                for (i, label) in owned.iter().enumerate() {
                    let n = i + 1;
                    if ui.button(label.clone()).clicked() {
                        if is_undo {
                            model.undo_many(n);
                        } else {
                            model.redo_many(n);
                        }
                        ui.close();
                    }
                }
            });
        });
    }
}

fn large_tool(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    icon: &'static str,
    label: &str,
    cmd: &'static str,
    snap: &RibbonSnapshot,
) {
    let tex = icon_tex(ui.ctx(), icons, icon);
    let active = snap.is_active(cmd);
    ui.push_id(key, |ui| {
        ui.vertical(|ui| {
            ui.set_width(LARGE_W);
            let r = ui
                .add(
                    egui::Button::image(img(&tex, 30.0))
                        .min_size(Vec2::new(LARGE_W, 36.0))
                        .fill(tool_fill(active)),
                )
                .on_hover_text(tip_text(label, cmd));
            if r.clicked() {
                model.run_tool(cmd);
            }
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(label).size(10.5).color(c(0xe8e8e8)));
            });
        });
    });
}

/// 分裂下拉的菜单内容：图标项 + 勾选当前项。
fn menu_items_ui(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    menu: &'static [MenuEntry],
    dd: &'static str,
    snap: &RibbonSnapshot,
) {
    let current = snap.current_of(dd);
    for (icon, label, cmd) in menu {
        let tex = icon_tex(ui.ctx(), icons, icon);
        let mark = if current == Some(*cmd) { "✓ " } else { "   " };
        let text = format!("{mark}{label}");
        if ui
            .add(
                egui::Button::image_and_text(img(&tex, 16.0), egui::RichText::new(text).size(11.5))
                    .fill(Color32::TRANSPARENT),
            )
            .clicked()
        {
            model.select_dropdown(dd, cmd);
            ui.close();
        }
    }
}

fn current_face(
    dd: &str,
    icon: &'static str,
    label: &'static str,
    cmd: &'static str,
    menu: &'static [MenuEntry],
    snap: &RibbonSnapshot,
) -> (&'static str, &'static str, &'static str) {
    match snap.current_of(dd) {
        Some(cur) => menu
            .iter()
            .find(|(c, _, _)| *c == cur)
            .map(|(i, l, c)| (*i, *l, *c))
            .unwrap_or((icon, label, cmd)),
        None => (icon, label, cmd),
    }
}

fn split_large(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    dd: &'static str,
    icon: &'static str,
    label: &'static str,
    cmd: &'static str,
    menu: &'static [MenuEntry],
    snap: &RibbonSnapshot,
) {
    let (face_icon, face_label, face_cmd) = current_face(dd, icon, label, cmd, menu, snap);
    let active = snap.is_active(dd) || menu.iter().any(|(c, _, _)| snap.is_active(c));

    ui.push_id(key, |ui| {
        ui.vertical(|ui| {
            ui.set_width(LARGE_W);
            let tex = icon_tex(ui.ctx(), icons, face_icon);
            let face = ui
                .add(
                    egui::Button::image(img(&tex, 30.0))
                        .min_size(Vec2::new(LARGE_W, 36.0))
                        .fill(tool_fill(active)),
                )
                .on_hover_text(tip_text(face_label, face_cmd));
            if face.clicked() {
                model.run_tool(face_cmd);
            }
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(face_label).size(10.5).color(c(0xe8e8e8)));
            });
            // 居中箭头 → 菜单（锚定按钮下方）
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.push_id(format!("{key}-caret"), |ui| {
                    ui.menu_button(egui::RichText::new("▾").size(9.0), |ui| {
                        menu_items_ui(ui, model, icons, menu, dd, snap);
                    });
                });
            });
        });
    });
}

fn small_tool(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    icon: &'static str,
    tip: &str,
    cmd: &'static str,
    snap: &RibbonSnapshot,
) {
    let tex = icon_tex(ui.ctx(), icons, icon);
    let active = snap.is_active(cmd);
    ui.push_id(key, |ui| {
        let r = ui
            .add(
                egui::Button::image(img(&tex, 16.0))
                    .min_size(Vec2::splat(23.0))
                    .fill(tool_fill(active)),
            )
            .on_hover_text(tip_text(tip, cmd));
        if r.clicked() {
            model.run_tool(cmd);
        }
    });
}

fn split_small(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    t: &SmallSplit,
    snap: &RibbonSnapshot,
) {
    let dd = t.cmd;
    let (face_icon, face_cmd) = match snap.current_of(dd) {
        Some(cur) => t
            .menu
            .iter()
            .find(|(c, _, _)| *c == cur)
            .map(|(i, _, c)| (*i, *c))
            .unwrap_or((t.icon, t.cmd)),
        None => (t.icon, t.cmd),
    };
    let active = snap.is_active(dd) || t.menu.iter().any(|(c, _, _)| snap.is_active(c));

    ui.push_id(key, |ui| {
        ui.vertical(|ui| {
            let tex = icon_tex(ui.ctx(), icons, face_icon);
            let face = ui
                .add(
                    egui::Button::image(img(&tex, 14.0))
                        .min_size(Vec2::new(24.0, 15.0))
                        .fill(tool_fill(active)),
                )
                .on_hover_text(tip_text(t.tip, face_cmd));
            if face.clicked() {
                model.run_tool(face_cmd);
            }
            if !t.menu.is_empty() {
                ui.push_id(format!("{key}-m"), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.menu_button(egui::RichText::new("▾").size(7.0), |ui| {
                            menu_items_ui(ui, model, icons, t.menu, dd, snap);
                        });
                    });
                });
            }
        });
    });
}

fn labeled_split(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    icon: &'static str,
    label: &'static str,
    cmd: &'static str,
    menu: &'static [MenuEntry],
    snap: &RibbonSnapshot,
) {
    let dd = cmd;
    let (face_icon, face_label, face_cmd) = current_face(dd, icon, label, cmd, menu, snap);
    let active = snap.is_active(dd) || menu.iter().any(|(c, _, _)| snap.is_active(c));

    ui.push_id(key, |ui| {
        egui::Frame::new()
            .fill(if active {
                Color32::from_rgba_unmultiplied(6, 150, 215, 46)
            } else {
                c(0x212121)
            })
            .stroke(egui::Stroke::new(1.0, if active { ACCENT } else { c(0x2e2e2e) }))
            .corner_radius(3.0)
            .inner_margin(egui::Margin::symmetric(4, 2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tex = icon_tex(ui.ctx(), icons, face_icon);
                    let icon_btn = ui
                        .add(egui::Button::image(img(&tex, 14.0)).fill(Color32::TRANSPARENT))
                        .on_hover_text(tip_text(face_label, face_cmd));
                    if icon_btn.clicked() {
                        model.run_tool(face_cmd);
                    }
                    let label_btn = ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(face_label).size(11.0).color(c(0xe8e8e8)),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text(tip_text(face_label, face_cmd));
                    if label_btn.clicked() {
                        model.run_tool(face_cmd);
                    }
                    if !menu.is_empty() {
                        ui.push_id(format!("{key}-m"), |ui| {
                            ui.menu_button(egui::RichText::new("▾").size(8.0), |ui| {
                                menu_items_ui(ui, model, icons, menu, dd, snap);
                            });
                        });
                    }
                });
            });
    });
}

fn layer_combo(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    snap: &RibbonSnapshot,
) {
    ui.push_id(key, |ui| {
        egui::Frame::new()
            .fill(c(0x212121))
            .stroke(egui::Stroke::new(1.0, c(0x2e2e2e)))
            .corner_radius(3.0)
            .inner_margin(egui::Margin::symmetric(5, 2))
            .show(ui, |ui| {
                ui.menu_button(
                    egui::RichText::new(format!("■ {} ▾", snap.active_layer))
                        .size(11.0)
                        .color(c(0xe8e8e8)),
                    |ui| {
                        layer_panel_ui(ui, model, icons);
                    },
                );
            });
    });
}

/// 图层面板：搜索 + 行内开关 + 活动层勾选（对应 OpenCADStudio layer_combo_overlay）。
fn layer_panel_ui(ui: &mut egui::Ui, model: &mut CadModel, icons: &mut HashMap<String, TextureHandle>) {
    let mut filter = std::mem::take(&mut model.layer_filter);
    ui.add_sized(
        [230.0, 22.0],
        egui::TextEdit::singleline(&mut filter)
            .hint_text("搜索图层…")
            .font(egui::FontId::proportional(11.0)),
    );
    model.layer_filter = filter;
    let query = model.layer_filter.to_lowercase();
    let rows: Vec<(usize, cad_demo::model::LayerInfo)> = model
        .layers
        .iter()
        .enumerate()
        .filter(|(_, l)| query.is_empty() || l.name.to_lowercase().contains(&query))
        .map(|(i, l)| (i, l.clone()))
        .collect();
    egui::ScrollArea::vertical()
        .max_height(150.0)
        .show(ui, |ui| {
            for (ix, l) in &rows {
                let is_active = l.name == model.active_layer;
                let swatch = egui::Color32::from_rgb(
                    (l.color >> 16) as u8,
                    (l.color >> 8) as u8,
                    l.color as u8,
                );
                ui.horizontal(|ui| {
                    if is_active {
                        ui.label(egui::RichText::new("✓").size(10.0).color(ACCENT));
                    } else {
                        ui.label("  ");
                    }
                    let visible = l.visible;
                    let frozen = l.frozen;
                    let locked = l.locked;
                    for (icon, on, kind) in [
                        (
                            if visible { "layers/layon.svg" } else { "layers/layoff.svg" },
                            visible,
                            "visible",
                        ),
                        (
                            if frozen { "layers/layfrz.svg" } else { "layers/laythw.svg" },
                            frozen,
                            "frozen",
                        ),
                        (
                            if locked { "layers/laylck.svg" } else { "layers/layulk.svg" },
                            locked,
                            "locked",
                        ),
                    ] {
                        let tex = icon_tex(ui.ctx(), icons, icon);
                        let mut image = img(&tex, 12.0);
                        if !on {
                            image = image.tint(Color32::from_rgb(90, 90, 90));
                        }
                        if ui.add(egui::Button::image(image).fill(Color32::TRANSPARENT)).clicked() {
                            model.layer_toggle(*ix, kind);
                        }
                    }
                    let (rect, _resp) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, swatch);
                    let name = l.name.clone();
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(name.clone()).size(11.0).color(if is_active {
                                    c(0xf2f2f2)
                                } else {
                                    c(0x9c9c9c)
                                }),
                            )
                            .fill(if is_active {
                                Color32::from_rgba_unmultiplied(6, 150, 215, 46)
                            } else {
                                Color32::TRANSPARENT
                            }),
                        )
                        .clicked()
                    {
                        model.layer_select(name);
                        ui.close();
                    }
                });
            }
        });
    ui.separator();
    if ui
        .add(egui::Button::new(egui::RichText::new("› 图层状态管理器…").size(11.0)))
        .clicked()
    {
        model.run_command("layerstate");
        ui.close();
    }
}

/// 组标题扩展面板触发器。
fn ext_title(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    group: &'static RibbonGroup,
    ext: &'static [ExtTool],
    snap: &RibbonSnapshot,
) {
    let open = snap.ext_open.as_deref() == Some(key);
    ui.push_id(format!("{key}-ext"), |ui| {
        let label = egui::RichText::new(format!("{} {}", group.name, if open { "▴" } else { "▾" }))
            .size(10.0)
            .color(if open { ACCENT } else { c(0x9c9c9c) });
        ui.menu_button(label, |ui| {
            model.set_ext_open(Some(key.to_string()));
            ext_panel_ui(ui, model, icons, ext, snap);
        });
    });
}

fn ext_panel_ui(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    ext: &'static [ExtTool],
    snap: &RibbonSnapshot,
) {
    if let Some(sub) = &snap.ext_sub {
        if let Some(tool) = ext.iter().find(|t| t.cmd == *sub) {
            if !tool.options.is_empty() {
                if ui.button(format!("‹ {}", tool.label)).clicked() {
                    model.set_ext_sub(None);
                }
                for (label, cmd) in tool.options {
                    if ui.button(*label).clicked() {
                        model.select_dropdown(tool.cmd, cmd);
                        model.set_ext_sub(None);
                        model.set_ext_open(None);
                        ui.close();
                    }
                }
                return;
            }
        }
    }
    // 工具网格：每行 4 格
    let mut rows: Vec<Vec<&ExtTool>> = vec![Vec::new()];
    for t in ext {
        if rows.last().is_some_and(|r| r.len() >= 4) {
            rows.push(Vec::new());
        }
        rows.last_mut().unwrap().push(t);
    }
    for row in &rows {
        ui.horizontal(|ui| {
            for t in row {
                ui.vertical(|ui| {
                    ui.set_width(52.0);
                    let tex = icon_tex(ui.ctx(), icons, t.icon);
                    let r = ui
                        .add_sized(
                            [26.0, 26.0],
                            egui::Button::image(img(&tex, 16.0))
                                .fill(tool_fill(snap.is_active(t.cmd))),
                        )
                        .on_hover_text(tip_text(t.label, t.cmd));
                    if r.clicked() {
                        model.run_tool(t.cmd);
                    }
                    if !t.options.is_empty()
                        && ui
                            .add_sized([26.0, 12.0], egui::Button::new(egui::RichText::new("▾").size(7.0)))
                            .clicked()
                    {
                        model.set_ext_sub(Some(t.cmd.to_string()));
                    }
                    ui.vertical_centered_justified(|ui| {
                        ui.label(egui::RichText::new(t.label).size(9.0).color(c(0x9c9c9c)));
                    });
                });
            }
        });
    }
}

/// Flyout（折叠档）按钮。
fn flyout_button(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    group: &'static RibbonGroup,
    _snap: &RibbonSnapshot,
    st: &mut UiState,
) {
    let face_icon = representative(group, &model.snapshot())
        .or_else(|| first_icon_cmd(group))
        .map(|(i, _)| i)
        .unwrap_or("line.svg");
    ui.push_id(format!("{key}-fly"), |ui| {
        let tex = icon_tex(ui.ctx(), icons, face_icon);
        let label = group.name;
        ui.menu_image_text_button(
            img(&tex, 16.0),
            egui::RichText::new(label).size(10.5).color(c(0xe8e8e8)),
            |ui| {
                let snap = model.snapshot();
                let mut rows: Vec<Vec<&RibbonItem>> = vec![Vec::new()];
                for item in &group.items {
                    if rows.last().is_some_and(|r| r.len() >= 3) {
                        rows.push(Vec::new());
                    }
                    rows.last_mut().unwrap().push(item);
                }
                for row in &rows {
                    ui.horizontal(|ui| {
                        for (k, item) in row.iter().enumerate() {
                            let fkey = format!("{key}-fly{}-{k}", snap.ext_sub.is_some() as u8);
                            render_item(ui, model, icons, &fkey, item, &snap, st);
                        }
                    });
                    ui.add_space(2.0);
                }
            },
        );
    });
}

fn representative(
    group: &RibbonGroup,
    snap: &RibbonSnapshot,
) -> Option<(&'static str, &'static str)> {
    for item in &group.items {
        match item {
            RibbonItem::Large { icon, cmd, .. } | RibbonItem::SplitLarge { icon, cmd, .. } => {
                if snap.is_active(cmd) {
                    return Some((icon, cmd));
                }
            }
            RibbonItem::Column { tools } => {
                if let Some((icon, _, cmd)) = tools.iter().find(|(_, _, c)| snap.is_active(c)) {
                    return Some((icon, cmd));
                }
            }
            RibbonItem::SplitColumn { tools } => {
                if let Some(t) = tools.iter().find(|t| snap.is_active(t.cmd)) {
                    return Some((t.icon, t.cmd));
                }
            }
            RibbonItem::LabeledSplit { icon, cmd, .. } => {
                if snap.is_active(cmd) {
                    return Some((icon, cmd));
                }
            }
            RibbonItem::LayerCombo => {}
        }
    }
    None
}

fn first_icon_cmd(group: &RibbonGroup) -> Option<(&'static str, &'static str)> {
    for item in &group.items {
        match item {
            RibbonItem::Large { icon, cmd, .. } | RibbonItem::SplitLarge { icon, cmd, .. } => {
                return Some((icon, cmd))
            }
            RibbonItem::Column { tools } => {
                return tools.first().map(|(i, _, c)| (*i, *c));
            }
            RibbonItem::SplitColumn { tools } => {
                return tools.first().map(|t| (t.icon, t.cmd));
            }
            RibbonItem::LabeledSplit { icon, cmd, .. } => return Some((icon, cmd)),
            RibbonItem::LayerCombo => return Some(("layers/panel.svg", "layer")),
        }
    }
    None
}

// ── 渲染分派 ────────────────────────────────────────────────────────────────

fn render_item(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    item: &RibbonItem,
    snap: &RibbonSnapshot,
    st: &mut UiState,
) {
    match item {
        RibbonItem::Large { icon, label, cmd } => {
            large_tool(ui, model, icons, key, icon, label, cmd, snap);
        }
        RibbonItem::SplitLarge { dd, icon, label, cmd, menu } => {
            split_large(ui, model, icons, key, dd, icon, label, cmd, menu, snap);
        }
        RibbonItem::Column { tools } => {
            for (k, (icon, tip, cmd)) in tools.iter().enumerate() {
                small_tool(ui, model, icons, &format!("{key}s{k}"), icon, tip, cmd, snap);
            }
        }
        RibbonItem::SplitColumn { tools } => {
            for (k, t) in tools.iter().enumerate() {
                split_small(ui, model, icons, &format!("{key}s{k}"), t, snap);
            }
        }
        RibbonItem::LabeledSplit { icon, label, cmd, menu } => {
            labeled_split(ui, model, icons, key, icon, label, cmd, menu, snap);
        }
        RibbonItem::LayerCombo => {
            layer_combo(ui, model, icons, key, snap);
        }
    }
    let _ = st;
}

fn render_compact(
    ui: &mut egui::Ui,
    model: &mut CadModel,
    icons: &mut HashMap<String, TextureHandle>,
    key: &str,
    item: &RibbonItem,
) {
    let icons_of = |item: &RibbonItem| -> Vec<(&'static str, &'static str)> {
        match item {
            RibbonItem::Large { icon, cmd, .. } | RibbonItem::SplitLarge { icon, cmd, .. } => {
                vec![(icon, cmd)]
            }
            RibbonItem::Column { tools } => tools.iter().map(|(i, _, c)| (*i, *c)).collect(),
            RibbonItem::SplitColumn { tools } => tools.iter().map(|t| (t.icon, t.cmd)).collect(),
            RibbonItem::LabeledSplit { icon, cmd, .. } => vec![(icon, cmd)],
            RibbonItem::LayerCombo => vec![("layers/panel.svg", "layer")],
        }
    };
    ui.vertical(|ui| {
        for (k, (icon, cmd)) in icons_of(item).into_iter().enumerate() {
            let tex = icon_tex(ui.ctx(), icons, icon);
            let r = ui
                .push_id(format!("{key}s{k}"), |ui| {
                    ui.add(
                        egui::Button::image(img(&tex, 16.0))
                            .min_size(Vec2::splat(24.0))
                            .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text(cmd)
                })
                .inner;
            if r.clicked() {
                model.run_tool(cmd);
            }
        }
    });
}

// ── 停靠面板内容 ────────────────────────────────────────────────────────────

fn viewport_ui(ui: &mut egui::Ui, model: &mut CadModel) {
    let (rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let p = ui.painter_at(rect);

    p.rect_filled(rect, 0.0, PAPER);
    // 栅格：小格 10、大格 50
    let mut x = 0.0;
    while x <= rect.width() {
        let color = if ((x / 10.0) as i32) % 5 == 0 { GRID_MAJOR } else { GRID_MINOR };
        p.line_segment(
            [egui::Pos2::new(rect.left() + x, rect.top()), egui::Pos2::new(rect.left() + x, rect.bottom())],
            egui::Stroke::new(1.0, color),
        );
        x += 10.0;
    }
    let mut y = 0.0;
    while y <= rect.height() {
        let color = if ((y / 10.0) as i32) % 5 == 0 { GRID_MAJOR } else { GRID_MINOR };
        p.line_segment(
            [egui::Pos2::new(rect.left(), rect.top() + y), egui::Pos2::new(rect.right(), rect.top() + y)],
            egui::Stroke::new(1.0, color),
        );
        y += 10.0;
    }

    let cx = rect.center().x;
    let cy = rect.center().y;
    // 坐标轴
    p.line_segment(
        [egui::Pos2::new(rect.left(), cy), egui::Pos2::new(rect.right(), cy)],
        egui::Stroke::new(1.5, Color32::from_rgba_unmultiplied(0xc0, 0x39, 0x2b, 0x60)),
    );
    p.line_segment(
        [egui::Pos2::new(cx, rect.top()), egui::Pos2::new(cx, rect.bottom())],
        egui::Stroke::new(1.5, Color32::from_rgba_unmultiplied(0x27, 0xae, 0x60, 0x60)),
    );

    // 示例几何：L 形板轮廓 + 圆 + 中心线
    let m2p = |mx: f32, my: f32| egui::Pos2::new(cx + mx * 10.0, cy - my * 10.0);
    let outline = vec![
        m2p(0.0, 0.0),
        m2p(12.0, 0.0),
        m2p(12.0, 4.0),
        m2p(4.0, 4.0),
        m2p(4.0, 10.0),
        m2p(0.0, 10.0),
        m2p(0.0, 0.0),
    ];
    p.add(egui::Shape::line(outline, egui::Stroke::new(2.0, ACCENT)));
    let (ccx, ccy, r) = (17.0_f32, 2.0_f32, 2.5_f32);
    let circle: Vec<egui::Pos2> = (0..=48)
        .map(|i| {
            let t = i as f32 / 48.0 * std::f32::consts::TAU;
            m2p(ccx + r * t.cos(), ccy + r * t.sin())
        })
        .collect();
    p.add(egui::Shape::line(circle, egui::Stroke::new(1.5, c(0x718c9e))));
    let cl = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x9a, 0xa0, 0xa6, 0x90));
    p.add(egui::Shape::line(
        vec![m2p(ccx - r - 1.5, ccy), m2p(ccx + r + 1.5, ccy)],
        cl,
    ));
    p.add(egui::Shape::line(
        vec![m2p(ccx, ccy - r - 1.5), m2p(ccx, ccy + r + 1.5)],
        cl,
    ));

    // 十字光标 + 实时坐标
    if let Some(pos) = resp.hover_pos() {
        let (x, y) = (pos.x, pos.y);
        let stroke = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x1a, 0x1a, 0x1a, 0xb8));
        p.line_segment([egui::Pos2::new(rect.left(), y), egui::Pos2::new(rect.right(), y)], stroke);
        p.line_segment([egui::Pos2::new(x, rect.top()), egui::Pos2::new(x, rect.bottom())], stroke);
        let s = 5.0;
        let pick = egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x1a, 0x1a, 0x1a, 0xe0));
        p.line_segment([egui::Pos2::new(x - s, y - s), egui::Pos2::new(x + s, y - s)], pick);
        p.line_segment([egui::Pos2::new(x - s, y + s), egui::Pos2::new(x + s, y + s)], pick);
        p.line_segment([egui::Pos2::new(x - s, y - s), egui::Pos2::new(x - s, y + s)], pick);
        p.line_segment([egui::Pos2::new(x + s, y - s), egui::Pos2::new(x + s, y + s)], pick);
        model.set_cursor(Some(((x - cx) as f64 / 10.0, (cy - y) as f64 / 10.0)));
    } else {
        model.set_cursor(None);
    }
}

fn command_line_ui(ui: &mut egui::Ui, model: &mut CadModel, input: &mut String) {
    ui.add_space(2.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        for line in model.log.iter().rev() {
            ui.label(
                egui::RichText::new(line)
                    .font(egui::FontId::monospace(11.0))
                    .color(c(0x9c9c9c)),
            );
        }
    });
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("命令:").size(11.5).color(ACCENT));
        let resp = ui.add(
            egui::TextEdit::singleline(input)
                .hint_text("输入命令（LINE / CIRCLE / MOVE …），回车执行")
                .desired_width(ui.available_width() - 8.0)
                .font(egui::FontId::proportional(12.0)),
        );
        if resp.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && !input.trim().is_empty()
        {
            let cmd = std::mem::take(input);
            model.run_command(&cmd);
        }
    });
    ui.add_space(2.0);
}

fn section_header(ui: &mut egui::Ui, name: &str) {
    ui.label(egui::RichText::new(name).size(11.0).strong().color(c(0xe8e8e8)));
    ui.separator();
}

fn properties_ui(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        section_header(ui, "常规");
        for (k, v) in [
            ("颜色", "ByLayer"),
            ("图层", "Walls"),
            ("线型", "ByLayer"),
            ("线宽", "0.30 毫米"),
            ("透明度", "ByLayer"),
            ("打印样式", "ByLayer"),
        ] {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(k).size(11.0).color(c(0x9c9c9c)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(v).size(11.0).color(c(0xe8e8e8)));
                });
            });
        }
        section_header(ui, "几何图形");
        for (k, v) in [
            ("起点 X", "0.0000"),
            ("起点 Y", "0.0000"),
            ("终点 X", "12.0000"),
            ("终点 Y", "4.0000"),
            ("长度", "12.0000"),
            ("角度", "18.435"),
            ("面积", "96.0000"),
        ] {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(k).size(11.0).color(c(0x9c9c9c)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(v).size(11.0).color(c(0xe8e8e8)));
                });
            });
        }
    });
}

fn layers_ui(ui: &mut egui::Ui, model: &mut CadModel) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (ix, l) in model.layers.clone().into_iter().enumerate() {
            let is_active = l.name == model.active_layer;
            let swatch = egui::Color32::from_rgb(
                (l.color >> 16) as u8,
                (l.color >> 8) as u8,
                l.color as u8,
            );
            ui.horizontal(|ui| {
                let (rect, resp) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::click());
                ui.painter().rect_filled(rect, 2.0, swatch);
                if resp.clicked() {
                    model.layer_select(l.name.clone());
                }
                let text = egui::RichText::new(&l.name).size(11.5).color(if l.visible {
                    c(0xe8e8e8)
                } else {
                    c(0x9c9c9c)
                });
                if ui
                    .add(
                        egui::Button::new(text)
                            .fill(if is_active {
                                Color32::from_rgba_unmultiplied(6, 150, 215, 46)
                            } else {
                                Color32::TRANSPARENT
                            })
                            .min_size(Vec2::new(ui.available_width() - 30.0, 20.0)),
                    )
                    .clicked()
                {
                    model.layer_select(l.name.clone());
                }
                if ui.small_button(if l.visible { "●" } else { "○" }).clicked() {
                    model.layer_toggle(ix, "visible");
                }
            });
        }
    });
}

fn palette_ui(ui: &mut egui::Ui, model: &mut CadModel, icons: &mut HashMap<String, TextureHandle>) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        section_header(ui, "块");
        for name in ["Chair", "Door 900", "Window 1200", "Tree Plan", "Sink", "Toilet"] {
            let tex = icon_tex(ui.ctx(), icons, "blocks/block.svg");
            let cmd = format!("insert {name}");
            ui.horizontal(|ui| {
                ui.image((tex.id(), Vec2::splat(14.0)));
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(name).size(11.5).color(c(0xe8e8e8)))
                            .fill(Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    model.run_command(&cmd);
                }
            });
        }
        section_header(ui, "图案填充");
        for name in ["ANSI31", "ANSI37", "SOLID", "GRAVEL"] {
            let tex = icon_tex(ui.ctx(), icons, "hatch/hatch_solid.svg");
            let cmd = format!("hatch {name}");
            ui.horizontal(|ui| {
                ui.image((tex.id(), Vec2::splat(14.0)));
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(name).size(11.5).color(c(0xe8e8e8)))
                            .fill(Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    model.run_command(&cmd);
                }
            });
        }
    });
}

// ── 主题 ────────────────────────────────────────────────────────────────────

/// egui 默认字体不含 CJK 字形，从系统字体目录补一个中文字体。
fn load_cjk_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for path in [
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ] {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("cjk".into(), egui::FontData::from_owned(data).into());
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.get_mut(&family).unwrap().push("cjk".into());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
}

fn apply_theme(ctx: &egui::Context) {
    load_cjk_font(ctx);
    let mut style = egui::Style::default();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = c(0x1a1a1a);
    style.visuals.window_fill = c(0x242424);
    style.visuals.extreme_bg_color = c(0x212121);
    style.visuals.faint_bg_color = c(0x232323);
    style.visuals.override_text_color = Some(c(0xe8e8e8));
    style.visuals.widgets.noninteractive.bg_fill = c(0x1f1f1f);
    style.visuals.widgets.inactive.bg_fill = c(0x232323);
    style.visuals.widgets.hovered.bg_fill = c(0x2c2c2c);
    style.visuals.widgets.active.bg_fill = c(0x333333);
    style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(6, 150, 215, 90);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, c(0xe0e0e0));
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, c(0xffffff));
    style.spacing.button_padding = Vec2::new(6.0, 3.0);
    style.spacing.item_spacing = Vec2::new(5.0, 3.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1520.0, 940.0])
            .with_title("OpenCAD Demo — egui / egui_tiles"),
        ..Default::default()
    };
    eframe::run_native(
        "OpenCAD Demo — egui",
        options,
        Box::new(|cc| Ok(Box::new(CadEgui::new(cc)))),
    )
}
