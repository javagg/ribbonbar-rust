//! RibbonBar：仿 Office/AutoCAD 风格的功能区。
//!
//! 行为对照 OpenCADStudio：
//! - 分裂下拉记录 `last_cmd`：按钮面（图标+文字）切换为最后使用的变体，
//!   菜单勾选联动当前项（`current_of`：镜像状态优先于最后执行）。
//! - 工具点击后"贴住"（active_tool），对应按钮持续高亮；开关类命令
//!   （OSNAP/ORTHO/ViewCube/UCS 图标…）映射到应用持久开关。
//! - 组标题扩展面板：注册了扩展的组，标题变成"标题 ▾"，点开工具网格，
//!   工具可再带子选项列表（返回行导航）。
//! - 快速访问 Undo/Redo 带历史下拉，可一次回退/重做 N 步。
//! - 图层组合框：搜索过滤 + 行内 可见/冻结/锁定 开关 + 活动层勾选。
//! - 富 tooltip："工具名\n命令: XXX"。

use gpui_kit::base::ElementExt;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_kit::component::popover::Popover;
use gpui_kit::component::theme::Theme;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme, Icon, IconName, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    div, img, px, rgba, Anchor, AnyElement, AppContext as _, ClickEvent, Context, ElementId, Entity,
    FontWeight, IntoElement, InteractiveElement as _, ParentElement, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled, Window,
};

use crate::icons;
use crate::office_widgets::office_theme;
use crate::state::{self, LayerInfo, RibbonSnapshot};
use crate::theme::OfficeTheme;

pub use cad_demo::ribbon_data::*;

// ---------------------------------------------------------------------------
// RibbonBar 视图
// ---------------------------------------------------------------------------

const TAB_STRIP_H: f32 = 30.0;
const CONTENT_H: f32 = 82.0;

pub struct RibbonBar {
    pub active: usize,
    pub collapsed: bool,
    row_width: f32,
    /// 图层下拉的搜索框（复用实体，内容闭包里读它的 value 做过滤）。
    layer_filter: Entity<InputState>,
}

impl RibbonBar {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let layer_filter = cx.new(|cx| InputState::new(window, cx).placeholder("搜索图层…"));
        // 输入变化 → 重绘（打开中的下拉随之过滤）
        cx.subscribe(&layer_filter, |_, _, _: &InputEvent, cx| {
            cx.notify();
        })
        .detach();
        // 首帧即用视口宽度决定降级档位（prepaint 阶段的 notify 不可靠）
        let viewport_w = window.viewport_size().width.as_f32();
        Self { active: 0, collapsed: false, row_width: viewport_w, layer_filter }
    }

    pub fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ot = office_theme(cx);
        let theme = cx.theme().clone();
        let this = cx.entity();
        let snap = state::ribbon_snapshot(cx);

        let strip = self.render_tab_strip(window, cx, &snap);
        // Backstage 展开时工具区一并隐藏（Office 行为：只留标签行 + 全屏面板）
        let content = (!self.collapsed && !snap.app_menu_open).then(|| {
            let tabs = tabs();
            let tab = &tabs[self.active.min(tabs.len() - 1)];

            let widths: Vec<GroupWidths> = tab.groups.iter().map(group_widths).collect();
            let levels = if self.row_width > f32::MAX / 2.0 {
                vec![Level::Full; widths.len()]
            } else {
                decide_levels(&widths, self.row_width - 24.0)
            };

            let mut groups = Vec::with_capacity(tab.groups.len());
            for ((gi, group), level) in tab.groups.iter().enumerate().zip(levels) {
                groups.push(self.render_group(self.active, gi, group, level, &snap, &self.layer_filter, &ot, &theme));
            }

            div()
                .id("ribbon-content")
                .w_full()
                .h(px(CONTENT_H))
                .bg(rgba(0xffffff08))
                .border_b_1()
                .border_color(theme.border)
                .flex()
                .items_stretch()
                .overflow_hidden()
                .px_2()
                .py_1()
                .on_prepaint(move |bounds, _, cx| {
                    this.update(cx, |r, cx| {
                        if (r.row_width - bounds.size.width.as_f32()).abs() > 0.5 {
                            r.row_width = bounds.size.width.as_f32();
                            cx.notify();
                        }
                    });
                })
                .children(groups)
        });

        div()
            .flex().flex_col().w_full().flex_shrink_0()
            .bg(ot.tool_bg)
            .border_b_1()
            .border_color(ot.group_line)
            .child(strip)
            .children(content)
    }

    fn render_tab_strip(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
        snap: &RibbonSnapshot,
    ) -> Stateful<gpui_kit::Div> {
        let ot = office_theme(cx);
        let tabs = tabs();

        let mut strip = div()
            .id("ribbon-tabs")
            .h_flex()
            .w_full()
            .h(px(TAB_STRIP_H))
            .bg(ot.tab_bar_bg)
            .px_2();

        // 快速访问：新建/打开/保存/打印 + Undo/Redo 历史下拉（Office 位置在标签行内）
        strip = strip.children([
            quick_button("qa-new", "ui/doc_new.svg", "新建", "new"),
            quick_button("qa-open", "ui/folder_open.svg", "打开", "open"),
            quick_button("qa-save", "ui/save.svg", "保存", "save"),
            quick_button("qa-print", "ui/print.svg", "打印", "plot"),
        ]);
        strip = strip.child(history_button("qa-undo", "ui/undo.svg", "放弃", &snap.undo_labels, true));
        strip = strip.child(history_button("qa-redo", "ui/redo.svg", "重做", &snap.redo_labels, false));
        strip = strip.child(
            div()
                .h(px(18.0))
                .w(px(1.0))
                .bg(ot.group_line)
                .mx_1()
                .flex_shrink_0(),
        );

        // 文件：应用按钮（点击展开 Backstage 全屏菜单，展开时保持高亮）
        let menu_open = snap.app_menu_open;
        strip = strip.child(
            div()
                .id("ribbon-file")
                .h(px(TAB_STRIP_H - 4.0))
                .px_3()
                .flex()
                .items_center()
                .rounded_t_sm()
                .text_size(px(12.5))
                .cursor_pointer()
                .when(menu_open, |el| {
                    el.bg(ot.tab_active_bg).text_color(ot.tab_active_text)
                })
                .when(!menu_open, |el| {
                    el.bg(ot.file_bg)
                        .text_color(ot.tab_text)
                        .hover(|s| s.bg(ot.title_bg))
                })
                .child("文件")
                .on_click(move |_, _, cx| {
                    state::set_app_menu_open(!menu_open, cx);
                }),
        );

        // 标签页：激活 = 白底黑字（点亮工具区）
        strip = strip.children(tabs.iter().enumerate().map(|(ix, tab)| {
            let active = ix == self.active;
            let name = tab.name;
            div()
                .id(ElementId::Name(format!("tab-{ix}").into()))
                .h(px(TAB_STRIP_H - 4.0))
                .px_3()
                .flex()
                .items_center()
                .rounded_t_sm()
                .text_size(px(12.5))
                .cursor_pointer()
                .when(active, |el| {
                    el.bg(ot.tab_active_bg).text_color(ot.tab_active_text)
                })
                .when(!active, |el| {
                    el.text_color(ot.tab_text)
                        .hover(move |s| s.bg(ot.title_bg))
                })
                .child(name)
                .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                    this.active = ix;
                    // Office 经典行为：双击标签折叠/展开功能区
                    if event.click_count() >= 2 {
                        this.collapsed = !this.collapsed;
                    }
                    cx.notify();
                }))
        }));

        // 上下文标签：第二主题色、位置更高；仅当有贴住的绘图工具时出现
        // （对应 Office 选中图片时出现 Picture Tools）
        let tool_stuck = snap.active_tool.is_some();
        strip = strip.when(tool_stuck, |strip| {
            strip.child(
                div()
                    .id("tab-contextual")
                    .mt(px(-6.0))
                    .h(px(TAB_STRIP_H))
                    .px_3()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .rounded_t_sm()
                    .bg(ot.ctx_bg)
                    .cursor_pointer()
                    .child(
                        div().text_size(px(7.5)).text_color(ot.ctx_text).child("打印工具"),
                    )
                    .child(
                        div().text_size(px(12.5)).text_color(ot.ctx_text).child("打印格式"),
                    ),
            )
        });

        strip = strip.child(div().flex_1());

        strip
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn render_group(
        &self,
        tab_ix: usize,
        group_ix: usize,
        group: &'static RibbonGroup,
        level: Level,
        snap: &RibbonSnapshot,
        filter: &Entity<InputState>,
        ot: &OfficeTheme,
        theme: &Theme,
    ) -> AnyElement {
        let key = format!("t{tab_ix}g{group_ix}");
        let body: AnyElement = match level {
            Level::Full => {
                let mut items = Vec::with_capacity(group.items.len());
                for (ii, item) in group.items.iter().enumerate() {
                    items.push(self.render_item(&format!("{key}i{ii}"), item, snap, filter, ot, theme));
                }
                div().flex().flex_row().items_start().h_full().children(items).into_any_element()
            }
            Level::Middle => {
                // FR Middle 档：所有工具变 22 高横排按钮，流式换行
                let mut cells: Vec<AnyElement> = Vec::new();
                let mut k = 0usize;
                for item in group.items.iter() {
                    match item {
                        RibbonItem::Large { icon, label, cmd } => {
                            cells.push(middle_tool(
                                &format!("{key}m{k}"), icon, label, None, cmd, snap, theme,
                            ).into_any_element());
                            k += 1;
                        }
                        RibbonItem::SplitLarge { dd, icon, label, cmd, menu } => {
                            cells.push(middle_tool(
                                &format!("{key}m{k}"), icon, label, Some((dd, menu)), cmd, snap, theme,
                            ).into_any_element());
                            k += 1;
                        }
                        RibbonItem::Column { tools } => {
                            for t in tools.iter() {
                                cells.push(middle_tool(
                                    &format!("{key}m{k}"), t.icon, t.label, None, t.cmd, snap, theme,
                                ).into_any_element());
                                k += 1;
                            }
                        }
                        RibbonItem::SplitColumn { tools } => {
                            for t in tools.iter() {
                                cells.push(middle_tool(
                                    &format!("{key}m{k}"), t.icon, t.label, None, t.cmd, snap, theme,
                                ).into_any_element());
                                k += 1;
                            }
                        }
                        other => {
                            cells.push(self.render_item(&format!("{key}m{k}"), other, snap, filter, ot, theme));
                            k += 1;
                        }
                    }
                }
                div()
                    .flex().flex_row().flex_wrap().gap_x_1().gap_y_0p5()
                    .items_center()
                    .children(cells)
                    .into_any_element()
            }
            Level::Compact => {
                let mut cols = Vec::new();
                for (ii, item) in group.items.iter().enumerate() {
                    cols.push(self.render_compact_item(&format!("{key}c{ii}"), item, theme));
                }
                div().flex().flex_row().items_start().h_full().children(cols).into_any_element()
            }
            Level::Flyout => flyout_button(&key, group, snap, filter, theme).into_any_element(),
        };

        // 组标题：组名（居中）+ 右下角对话框启动器；扩展组为"标题 ▾"。
        // Flyout 档不渲染（标题按钮已含组名，避免重复）
        let title: AnyElement = if level == Level::Flyout {
            div().h(px(15.0)).into_any_element()
        } else {
            match (group.ext, level) {
                (Some(ext), Level::Full) => {
                    ext_title_button(&key, group, ext, snap, theme).into_any_element()
                }
                _ => div()
                    .h(px(15.0))
                    .flex().items_center().justify_center()
                    .text_size(px(10.0))
                    .text_color(ot.group_name)
                    .child(SharedString::from(group.name))
                    .into_any_element(),
            }
        };

        div()
            .flex().flex_col()
            .border_l_1()
            .border_color(ot.group_line)
            .px_2()
            .mx_1()
            .child(div().flex().flex_row().items_start().h_full().child(body))
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .child(
                        div().flex_1().child(title),
                    )
                    .when(group.launcher && level == Level::Full, |el| {
                        // 对话框启动器（FR IsLauncherVisible 默认 false，按组启用）
                        el.child(
                            div()
                                .id(ElementId::Name(format!("{key}-launcher").into()))
                                .size(px(9.0))
                                .rounded_xs()
                                .border_1()
                                .border_color(ot.group_line)
                                .flex().items_center().justify_center()
                                .text_size(px(5.0))
                                .text_color(ot.group_name)
                                .hover(move |s| s.bg(ot.hover).border_color(ot.hover_border))
                                .cursor_pointer()
                                .child("[]")
                                .on_click(move |_, _, cx| {
                                    state::run_command(&format!("{}_options", group.name), cx)
                                }),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_item(
        &self,
        key: &str,
        item: &RibbonItem,
        snap: &RibbonSnapshot,
        filter: &Entity<InputState>,
        ot: &OfficeTheme,
        theme: &Theme,
    ) -> AnyElement {
        match item {
            RibbonItem::Large { icon, label, cmd } => {
                large_tool(key, icon, label, cmd, snap, theme).into_any_element()
            }
            RibbonItem::SplitLarge { dd, icon, label, cmd, menu } => {
                split_large(key, dd, icon, label, cmd, menu, snap, theme).into_any_element()
            }
            RibbonItem::Column { tools } => {
                let mut row = div().flex().flex_row().gap_px();
                for (k, t) in tools.iter().enumerate() {
                    row = row.child(small_tool(&format!("{key}s{k}"), t.icon, t.label, t.cmd, snap, theme));
                }
                row.into_any_element()
            }
            RibbonItem::SplitColumn { tools } => {
                let mut row = div().flex().flex_row().gap_px();
                for (k, t) in tools.iter().enumerate() {
                    row = row.child(split_small(&format!("{key}s{k}"), t, snap, theme));
                }
                row.into_any_element()
            }
            RibbonItem::LabeledSplit { icon, label, cmd, menu } => {
                labeled_split(key, icon, label, cmd, menu, snap, theme).into_any_element()
            }
            RibbonItem::LayerCombo => layer_combo(key, snap, filter, theme).into_any_element(),
            RibbonItem::PasteGroup { icon, label, cmd, sides } => {
                crate::office_widgets::render_paste_group(key, icon, label, cmd, sides, ot)
                    .into_any_element()
            }
            RibbonItem::ComboRow { items } => {
                crate::office_widgets::render_combo_row(key, items, ot).into_any_element()
            }
            RibbonItem::CharFlow { buttons } => {
                crate::office_widgets::render_char_flow(key, buttons, ot).into_any_element()
            }
            RibbonItem::SpinRow { items } => {
                crate::office_widgets::render_spin_row(key, items, ot).into_any_element()
            }
            RibbonItem::ActionRow { items } => {
                crate::office_widgets::render_action_row(key, items, ot).into_any_element()
            }
        }
    }

    /// Compact 档：每一项收成一个 26px 图标竖列。
    fn render_compact_item(&self, key: &str, item: &RibbonItem, theme: &Theme) -> AnyElement {
        let mut col = div().flex().flex_col().gap_0p5().items_center();
        match item {
            RibbonItem::Large { icon, cmd, .. } | RibbonItem::SplitLarge { icon, cmd, .. } => {
                col = col.child(mini_tool(&format!("{key}m"), icon, cmd, theme));
            }
            RibbonItem::Column { tools } => {
                for (k, t) in tools.iter().enumerate() {
                    col = col.child(mini_tool(&format!("{key}s{k}"), t.icon, t.cmd, theme));
                }
            }
            RibbonItem::SplitColumn { tools } => {
                for (k, t) in tools.iter().enumerate() {
                    col = col.child(mini_tool(&format!("{key}s{k}"), t.icon, t.cmd, theme));
                }
            }
            RibbonItem::LabeledSplit { icon, cmd, .. } => {
                col = col.child(mini_tool(&format!("{key}m"), icon, cmd, theme));
            }
            RibbonItem::LayerCombo => {
                col = col.child(mini_tool(&format!("{key}m"), "layers/panel.svg", "layer", theme));
            }
            RibbonItem::PasteGroup { icon, cmd, .. } => {
                col = col.child(mini_tool(&format!("{key}m"), icon, cmd, theme));
            }
            RibbonItem::ComboRow { .. } => {
                col = col.child(mini_tool(&format!("{key}m"), "ui/gear.svg", "combo", theme));
            }
            RibbonItem::CharFlow { buttons, .. } => {
                if let Some(b) = buttons.first() {
                    col = col.child(mini_tool(&format!("{key}m"), "text.svg", b.cmd, theme));
                }
            }
            RibbonItem::SpinRow { items, .. } => {
                if let Some(s) = items.first() {
                    col = col.child(mini_tool(&format!("{key}m"), "status/lwt.svg", s.cmd, theme));
                }
            }
            RibbonItem::ActionRow { items, .. } => {
                if let Some(a) = items.first() {
                    col = col.child(mini_tool(&format!("{key}m"), "ui/gear.svg", a.cmd, theme));
                }
            }
        }
        col.into_any_element()
    }
}

// ---------------------------------------------------------------------------
// 通用小部件
// ---------------------------------------------------------------------------

/// 高亮背景：贴住的工具比 hover 更醒目（对应 tool_btn_style 的 active）。
fn tool_button_base(key: &str, w: f32, h: f32, active: bool) -> Stateful<gpui_kit::Div> {
    div()
        .id(ElementId::Name(key.into()))
        .w(px(w))
        .h(px(h))
        .rounded_sm()
        .flex()
        .cursor_pointer()
        .when(active, |el| el.bg(rgba(0x0696d74d)))
        .hover(move |s| s.bg(if active { rgba(0x0696d760) } else { rgba(0x0696d726) }))
        .active(|s| s.bg(rgba(0x0696d780)))
}

/// 渲染深处取 Office 主题（init 时缓存）。
fn ot() -> OfficeTheme {
    crate::office_widgets::ot_cached()
}

fn tip_text(label: &str, cmd: &str) -> SharedString {
    // 两行标签（\n）在 tooltip 里折叠为空格
    let label = label.replace('\n', " ");
    match shortcut_of(cmd) {
        Some(key) => SharedString::from(format!("{label}\n命令: {cmd}\n快捷键: {key}")),
        None => SharedString::from(format!("{label}\n命令: {cmd}")),
    }
}

/// TwoLineLabel（FR 规范）：Large 态标签最多两行，按 \n 拆行上下叠放。
fn two_line_label(label: &str, theme: &Theme) -> AnyElement {
    let (l1, l2) = match label.split_once('\n') {
        Some((a, b)) => (a, Some(b)),
        None => (label, None),
    };
    div()
        .flex()
        .flex_col()
        .items_center()
        .text_size(px(10.5))
        .text_color(theme.foreground)
        .overflow_hidden()
        .child(SharedString::from(l1.to_string()))
        .children(l2.map(|l| SharedString::from(l.to_string())))
        .into_any_element()
}

fn attach_tooltip(el: Stateful<gpui_kit::Div>, text: SharedString) -> Stateful<gpui_kit::Div> {
    el.tooltip(move |window, cx| Tooltip::new(text.clone()).build(window, cx))
}

fn quick_button(id: &str, icon: &'static str, tip: &'static str, cmd: &'static str) -> Stateful<gpui_kit::Div> {
    attach_tooltip(
        div()
            .id(ElementId::Name(id.into()))
            .size(px(26.0))
            .rounded_sm()
            .flex().items_center().justify_center()
            .hover(|s| s.bg(rgba(0xffffff14)))
            .active(|s| s.bg(rgba(0xffffff20)))
            .cursor_pointer()
            .on_click(move |_, _, cx| state::run_command(cmd, cx))
            .child(img(icons::icon(icon)).size(px(17.0)).flex_shrink_0()),
        tip_text(tip, cmd),
    )
}

/// Undo/Redo 历史下拉：面执行一步，小箭头列出历史标签，点第 k 项一次回退/重做 k 步。
fn history_button(
    id: &str,
    icon: &'static str,
    tip: &'static str,
    labels: &[String],
    is_undo: bool,
) -> Stateful<gpui_kit::Div> {
    let caret_id = ElementId::Name(format!("{id}-caret").into());
    let owned: Vec<String> = labels.to_vec();
    let has_history = !owned.is_empty();

    let face = attach_tooltip(
        div()
            .id(ElementId::Name(id.into()))
            .size(px(26.0))
            .rounded_sm()
            .flex().items_center().justify_center()
            .hover(|s| s.bg(rgba(0xffffff14)))
            .active(|s| s.bg(rgba(0xffffff20)))
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                if is_undo {
                    state::undo(cx)
                } else {
                    state::redo(cx)
                }
            })
            .child(img(icons::icon(icon)).size(px(17.0)).flex_shrink_0()),
        tip_text(tip, if is_undo { "U" } else { "REDO" }),
    );

    div()
        .id(ElementId::Name(format!("{id}-wrap").into()))
        .h_flex()
        .child(face)
        .when(has_history, |el| {
            el.child(
                div().w(px(11.0)).h(px(26.0)).child(
                    Button::new(caret_id)
                        .ghost()
                        .w_full()
                        .h(px(20.0))
                        .child(
                            div().w_full().flex().justify_center()
                                .child(Icon::new(IconName::ChevronDown).size(px(8.0))),
                        )
                        .dropdown_menu(move |popup, _, _| {
                            let mut m = popup;
                            for (i, label) in owned.iter().enumerate() {
                                let n = i + 1;
                                m = m.item(PopupMenuItem::new(label.clone()).on_click(
                                    move |_, _, cx| {
                                        if is_undo {
                                            state::undo_many(n, cx)
                                        } else {
                                            state::redo_many(n, cx)
                                        }
                                    },
                                ));
                            }
                            m
                        }),
                ),
            )
        })
}

fn large_tool(
    key: &str,
    icon: &'static str,
    label: &str,
    cmd: &'static str,
    snap: &RibbonSnapshot,
    theme: &Theme,
) -> Stateful<gpui_kit::Div> {
    let active = snap.is_active(cmd);
    attach_tooltip(
        tool_button_base(key, LARGE_W, LARGE_H, active)
            .flex().flex_col()
            .pt_1()
            .on_click(move |_, _, cx| state::run_tool(cmd, cx))
            // FR Large 结构：图标 32 在上，TwoLineLabel 在下（最多两行）
            .child(
                div().h(px(32.0)).flex().items_center().justify_center()
                    .child(img(icons::icon(icon)).size(px(32.0))),
            )
            .child(
                div()
                    .flex_1()
                    .flex().items_start().justify_center()
                    .child(two_line_label(label, theme)),
            ),
        tip_text(label, cmd),
    )
}

fn split_large(
    key: &str,
    dd: &'static str,
    icon: &'static str,
    label: &'static str,
    cmd: &'static str,
    menu: &'static [MenuEntry],
    snap: &RibbonSnapshot,
    theme: &Theme,
) -> Stateful<gpui_kit::Div> {
    let caret_id = ElementId::Name(format!("{key}-caret").into());
    // 面 = dd 的当前变体（last_cmd 记忆），默认回落到 (icon, label, cmd)
    let (face_icon, face_label, face_cmd) = current_face(dd, icon, label, cmd, menu, snap);
    // 整个分裂按钮高亮：dd 或任一菜单命令贴住
    let active = snap.is_active(dd) || menu.iter().any(|(c, _, _)| snap.is_active(c));

    div()
        .id(ElementId::Name(key.into()))
        .flex().flex_col()
        .w(px(LARGE_W))
        .h(px(LARGE_H))
        .rounded_sm()
        .cursor_pointer()
        .when(active, |el| el.bg(rgba(0x0696d74d)))
        .hover(move |s| s.bg(if active { rgba(0x0696d760) } else { rgba(0x0696d726) }))
        .child(
            // face 与 large_tool 同构（FR：SplitButton 与 Button 外观一致）
            attach_tooltip(
                div()
                    .id(ElementId::Name(format!("{key}-main").into()))
                    .flex_1()
                    .flex().flex_col()
                    .pt_1()
                    .on_click(move |_, _, cx| state::run_tool(face_cmd, cx))
                    .child(
                        div().h(px(32.0)).flex().items_center().justify_center()
                            .child(img(icons::icon(face_icon)).size(px(32.0))),
                    )
                    .child(
                        // 标签行内右下角下拉箭头（FR：TwoLineLabel HasGlyph），独立点击区
                        div()
                            .flex_1()
                            .w_full()
                            .flex().items_stretch()
                            .child(
                                div().flex_1().flex().items_start().justify_center()
                                    .child(two_line_label(face_label, theme)),
                            )
                            .child(
                                div()
                                    .w(px(12.0))
                                    .flex().items_end().justify_center()
                                    .child(
                                        Button::new(caret_id)
                                            .ghost()
                                            .w_full()
                                            .h(px(14.0))
                                            .child(
                                                div().w_full().flex().justify_center()
                                                    .child(Icon::new(IconName::ChevronDown).size(px(8.0))),
                                            )
                                            .dropdown_menu(move |popup, _, _| build_menu(popup, menu, dd)),
                                    ),
                            ),
                    ),
                tip_text(face_label, face_cmd),
            ),
        )
}

/// 分裂按钮的当前面：current_cmd / last_cmd 优先，其次默认。
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

/// 分裂下拉菜单：图标项；选中项经 select_dropdown 联动 last_cmd/勾选。
fn build_menu(popup: PopupMenu, menu: &'static [MenuEntry], dd: &'static str) -> PopupMenu {
    let mut m = popup;
    for (icon, label, cmd) in menu.iter() {
        let path = format!("cad/{icon}");
        m = m.item(
            PopupMenuItem::new(*label)
                .icon(Icon::empty().path(path))
                .on_click(move |_, _, cx| state::select_dropdown(dd, cmd, cx)),
        );
    }
    m
}

/// FR Middle 档按钮：22 高横排（icon 16 + 文字），dd 给定时带行内 ▾。
fn middle_tool(
    key: &str,
    icon: &'static str,
    label: &str,
    dd: Option<(&'static str, &'static [MenuEntry])>,
    cmd: &'static str,
    snap: &RibbonSnapshot,
    theme: &Theme,
) -> AnyElement {
    let active = snap.is_active(cmd)
        || dd.is_some_and(|(d, m)| snap.is_active(d) || m.iter().any(|(c, _, _)| snap.is_active(c)));
    let label = label.replace('\n', "");
    let caret_id = ElementId::Name(format!("{key}-caret").into());

    let face = attach_tooltip(
        div()
            .id(ElementId::Name(key.into()))
            .h(px(22.0))
            .flex().items_center()
            .gap_1()
            .px_1p5()
            .rounded_sm()
            .cursor_pointer()
            .when(active, |el| el.bg(rgba(0x0696d74d)))
            .hover(move |s| s.bg(if active { rgba(0x0696d760) } else { rgba(0x0696d726) }))
            .on_click(move |_, _, cx| state::run_tool(cmd, cx))
            .child(img(icons::icon(icon)).size(px(16.0)).flex_shrink_0())
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(theme.foreground)
                    .child(SharedString::from(label.clone())),
            ),
        tip_text(&label, cmd),
    );

    match dd {
        Some((dd, menu)) => div()
            .h_flex()
            .child(face)
            .child(
                div().w(px(12.0)).h(px(22.0)).child(
                    Button::new(caret_id)
                        .ghost()
                        .w_full()
                        .h(px(20.0))
                        .child(
                            div().w_full().flex().justify_center()
                                .child(Icon::new(IconName::ChevronDown).size(px(7.0))),
                        )
                        .dropdown_menu(move |popup, _, _| build_menu(popup, menu, dd)),
                ),
            )
            .into_any_element(),
        None => face.into_any_element(),
    }
}

/// 带文字标签的小按钮：图标上/文字下（用户准则：所有命令必须有文字标签）。
fn small_tool(
    key: &str,
    icon: &'static str,
    label: &str,
    cmd: &'static str,
    snap: &RibbonSnapshot,
    theme: &Theme,
) -> Stateful<gpui_kit::Div> {
    let active = snap.is_active(cmd);
    attach_tooltip(
        tool_button_base(key, STACK_W, STACK_H, active)
            .flex().flex_col()
            .items_center().justify_center()
            .gap_0p5()
            .on_click(move |_, _, cx| state::run_tool(cmd, cx))
            .child(img(icons::icon(icon)).size(px(16.0)).flex_shrink_0())
            .child(
                div()
                    .max_w_full()
                    .text_size(px(9.0))
                    .text_color(theme.foreground)
                    .overflow_hidden()
                    .child(SharedString::from(label)),
            ),
        tip_text(label, cmd),
    )
}

/// 带菜单的堆叠小按钮：图标 / ▾ / 文字标签 三层，整块高亮，面随 last_cmd 记忆。
fn split_small(key: &str, t: &SmallSplit, snap: &RibbonSnapshot, theme: &Theme) -> Stateful<gpui_kit::Div> {
    let caret_id = ElementId::Name(format!("{key}-caret").into());
    let dd: &'static str = t.cmd;
    let icon_field: &'static str = t.icon;
    let menu: &'static [MenuEntry] = t.menu;
    let (face_icon, face_label, face_cmd): (&'static str, &'static str, &'static str) = match snap.current_of(dd) {
        Some(cur) => menu
            .iter()
            .find(|(c, _, _)| *c == cur)
            .map(|(i, l, c)| (*i, *l, *c))
            .unwrap_or((icon_field, t.label, dd)),
        None => (icon_field, t.label, dd),
    };
    let active = snap.is_active(dd) || t.menu.iter().any(|(c, _, _)| snap.is_active(c));
    let tip = tip_text(face_label, face_cmd);

    let el = div()
        .id(ElementId::Name(key.into()))
        .w(px(STACK_W))
        .h(px(STACK_H))
        .rounded_sm()
        .flex().flex_col()
        .items_center().justify_center()
        .gap_0p5()
        .cursor_pointer()
        .when(active, |el| el.bg(rgba(0x0696d74d)))
        .hover(move |s| s.bg(if active { rgba(0x0696d760) } else { rgba(0x0696d726) }))
        .child(
            div()
                .id(ElementId::Name(format!("{key}-main").into()))
                .flex()
                .items_center().justify_center()
                .on_click(move |_, _, cx| state::run_tool(face_cmd, cx))
                .child(img(icons::icon(face_icon)).size(px(16.0)).flex_shrink_0()),
        );
    let el = if t.menu.is_empty() {
        el
    } else {
        el.child(
            div().w_full().h(px(7.0)).child(
                Button::new(caret_id)
                    .ghost()
                    .w_full()
                    .h(px(7.0))
                    .child(
                        div().w_full().flex().justify_center()
                            .child(Icon::new(IconName::ChevronDown).size(px(6.0))),
                    )
                    .dropdown_menu(move |popup, _, _| build_menu(popup, menu, dd)),
            ),
        )
    };
    let el = el.child(
        div()
            .max_w_full()
            .text_size(px(9.0))
            .text_color(theme.foreground)
            .overflow_hidden()
            .child(SharedString::from(face_label)),
    );
    attach_tooltip(el, tip)
}

fn labeled_split(
    key: &str,
    icon: &'static str,
    label: &'static str,
    cmd: &'static str,
    menu: &'static [MenuEntry],
    snap: &RibbonSnapshot,
    theme: &Theme,
) -> Stateful<gpui_kit::Div> {
    let caret_id = ElementId::Name(format!("{key}-caret").into());
    let dd = cmd;
    let (face_icon, face_label, face_cmd) = current_face(dd, icon, label, cmd, menu, snap);
    let active = snap.is_active(dd) || menu.iter().any(|(c, _, _)| snap.is_active(c));

    div()
        .id(ElementId::Name(key.into()))
        .h(px(22.0))
        .rounded_sm()
        .border_1()
        .when(active, |el| el.border_color(theme.accent).bg(rgba(0x0696d72e)))
        .when(!active, |el| {
            el.border_color(theme.border)
                .bg(theme.input)
                .hover(|s| s.border_color(theme.accent))
        })
        .flex().items_center()
        .pl_1p5()
        .cursor_pointer()
        .child(
            div()
                .id(ElementId::Name(format!("{key}-icon").into()))
                .on_click(move |_, _, cx| state::run_tool(face_cmd, cx))
                .child(img(icons::icon(face_icon)).size(px(14.0)).flex_shrink_0()),
        )
        .child(
            attach_tooltip(
                div()
                    .id(ElementId::Name(format!("{key}-label").into()))
                    .flex_1()
                    .px_1()
                    .text_size(px(11.0))
                    .text_color(theme.foreground)
                    .on_click(move |_, _, cx| state::run_tool(face_cmd, cx))
                    .child(SharedString::from(face_label)),
                tip_text(face_label, face_cmd),
            ),
        )
        .child(
            div().h_full().w(px(16.0)).flex().items_center().justify_center().child(
                Button::new(caret_id)
                    .ghost()
                    .w_full()
                    .h_full()
                    .child(
                        div().w_full().flex().justify_center()
                            .child(Icon::new(IconName::ChevronDown).size(px(8.0))),
                    )
                    .dropdown_menu(move |popup, _, _| build_menu(popup, menu, dd)),
            ),
        )
}

fn mini_tool(key: &str, icon: &'static str, cmd: &'static str, _theme: &Theme) -> Stateful<gpui_kit::Div> {
    div()
        .id(ElementId::Name(key.into()))
        .size(px(MINI_W - 2.0))
        .rounded_sm()
        .flex().items_center().justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(rgba(0x0696d726)))
        .active(|s| s.bg(rgba(0x0696d740)))
        .on_click(move |_, _, cx| state::run_tool(cmd, cx))
        .child(img(icons::icon(icon)).size(px(16.0)).flex_shrink_0())
}

// ---------------------------------------------------------------------------
// 图层组合框（搜索 + 行内开关 + 活动层勾选）
// ---------------------------------------------------------------------------

fn layer_combo(
    key: &str,
    snap: &RibbonSnapshot,
    filter: &Entity<InputState>,
    theme: &Theme,
) -> AnyElement {
    let pop_id = ElementId::Name(format!("{key}-pop").into());
    let trig_id = ElementId::Name(format!("{key}-trig").into());
    let filter = filter.clone();

    let pop = Popover::new(pop_id)
        .anchor(Anchor::TopLeft)
        .trigger(
            Button::new(trig_id)
                .ghost()
                .h(px(22.0))
                .w(px(110.0))
                .child(
                    div().h_flex().w_full().h_full().items_center().gap_1p5().px_1p5()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.input)
                        .rounded_sm()
                        .hover(|s| s.border_color(theme.accent))
                        .child(
                            div().size(px(8.0)).rounded_xs().flex_shrink_0()
                                .bg(gpui_kit::rgb(active_layer_color(snap))),
                        )
                        .child(
                            div().flex_1().text_size(px(11.0)).text_color(theme.foreground)
                                .child(SharedString::from(snap.active_layer.clone())),
                        )
                        .child(
                            div().text_size(px(8.0)).text_color(theme.muted_foreground)
                                .child("▾"),
                        ),
                ),
        )
        .content(move |_, _, cx| {
            let theme = cx.theme().clone();
            let snap = state::ribbon_snapshot(cx);
            let query = filter.read(cx).value().to_string();
            layer_panel(&filter, query, &snap, &theme)
        });

    div().id(ElementId::Name(key.into())).w(px(110.0)).child(pop).into_any_element()
}

fn active_layer_color(snap: &RibbonSnapshot) -> u32 {
    snap.layers
        .iter()
        .find(|l| l.name == snap.active_layer)
        .map(|l| l.color)
        .unwrap_or(0xf2f2f2)
}

fn layer_panel(
    input: &Entity<InputState>,
    query: String,
    snap: &RibbonSnapshot,
    theme: &Theme,
) -> AnyElement {
    let query = query.to_lowercase();
    let rows: Vec<(usize, &LayerInfo)> = snap
        .layers
        .iter()
        .enumerate()
        .filter(|(_, l)| query.is_empty() || l.name.to_lowercase().contains(&query))
        .collect();

    let list_h = (rows.len().max(1) as f32 * 24.0).min(144.0);
    let input = input.clone();

    div()
        .v_flex()
        .w(px(230.0))
        .gap_1()
        .p_1()
        .child(
            div().px_1().pt_1().child(
                Input::new(&input).h(px(22.0)).text_size(px(11.0)),
            ),
        )
        .child(
            div()
                .id("layer-list")
                .v_flex()
                .overflow_y_scroll()
                .h(px(list_h))
                .children(rows.into_iter().map(|(ix, l)| layer_row(ix, l, snap, theme))),
        )
        .child(
            div()
                .id("layer-state-mgr")
                .h_flex()
                .h(px(24.0))
                .px_2()
                .items_center()
                .gap_1p5()
                .text_size(px(11.0))
                .text_color(theme.muted_foreground)
                .hover(|s| s.bg(rgba(0xffffff0d)))
                .cursor_pointer()
                .border_t_1()
                .border_color(theme.border)
                .on_click(|_, _, cx| state::run_command("layerstate", cx))
                .child("›")
                .child("图层状态管理器…"),
        )
        .into_any_element()
}

fn layer_row(ix: usize, l: &LayerInfo, snap: &RibbonSnapshot, theme: &Theme) -> Stateful<gpui_kit::Div> {
    let is_active = l.name == snap.active_layer;
    let name = l.name.clone();
    let icon_btn = |key: String, icon: &'static str, on: bool, kind: &'static str| {
        div()
            .id(ElementId::Name(key.into()))
            .size(px(18.0))
            .rounded_xs()
            .flex().items_center().justify_center()
            .opacity(if on { 1.0 } else { 0.35 })
            .hover(|s| s.bg(rgba(0xffffff14)))
            .cursor_pointer()
            .on_click(move |_, _, cx| state::layer_toggle(ix, kind, cx))
            .child(img(icons::icon(icon)).size(px(12.0)).flex_shrink_0())
    };

    div()
        .id(ElementId::Name(format!("layer-row-{ix}").into()))
        .h_flex()
        .h(px(24.0))
        .px_1()
        .gap_1()
        .items_center()
        .rounded_xs()
        .when(is_active, |el| el.bg(rgba(0x0696d72e)))
        .when(!is_active, |el| el.hover(|s| s.bg(rgba(0xffffff0d))))
        .child(
            div().w(px(12.0)).text_size(px(10.0)).text_color(theme.accent)
                .child(if is_active { "✓" } else { " " }),
        )
        .child(icon_btn(
            format!("ly-vis-{ix}"),
            if l.visible { "layers/layon.svg" } else { "layers/layoff.svg" },
            l.visible,
            "visible",
        ))
        .child(icon_btn(
            format!("ly-frz-{ix}"),
            if l.frozen { "layers/layfrz.svg" } else { "layers/laythw.svg" },
            l.frozen,
            "frozen",
        ))
        .child(icon_btn(
            format!("ly-lck-{ix}"),
            if l.locked { "layers/laylck.svg" } else { "layers/layulk.svg" },
            l.locked,
            "locked",
        ))
        .child(
            div()
                .id(ElementId::Name(format!("ly-sel-{ix}").into()))
                .h_flex()
                .flex_1()
                .h_full()
                .items_center()
                .gap_1p5()
                .cursor_pointer()
                .on_click(move |_, _, cx| state::layer_select(name.clone(), cx))
                .child(
                    div().size(px(10.0)).rounded_xs().border_1()
                        .border_color(theme.border)
                        .bg(gpui_kit::rgb(l.color))
                        .flex_shrink_0(),
                )
                .child(
                    div().flex_1().text_size(px(11.0))
                        .text_color(if is_active { theme.foreground } else { theme.muted_foreground })
                        .child(SharedString::from(l.name.clone())),
                ),
        )
}

// ---------------------------------------------------------------------------
// 组标题扩展面板（对应 OpenCADStudio draw_panel.rs）
// ---------------------------------------------------------------------------

/// "标题 ▾"触发器；点开工具网格，工具可再进子选项列表。
fn ext_title_button(
    key: &str,
    group: &'static RibbonGroup,
    ext: &'static [ExtTool],
    snap: &RibbonSnapshot,
    theme: &Theme,
) -> impl IntoElement {
    let pop_id = ElementId::Name(format!("{key}-ext-pop").into());
    let trig_id = ElementId::Name(format!("{key}-ext-trig").into());
    let group_key = key.to_string();
    let open = snap.ext_open.as_deref() == Some(key);

    let pop = Popover::new(pop_id)
        .anchor(Anchor::TopLeft)
        .open(open)
        .trigger(
            Button::new(trig_id)
                .ghost()
                .h(px(15.0))
                .child(
                    div().h_flex().h_full().items_center().gap_1().px_1()
                        .rounded_xs()
                        .when(open, |el| el.bg(rgba(0x0696d73d)))
                        .child(
                            div().text_size(px(10.0))
                                .text_color(theme.muted_foreground)
                                .child(SharedString::from(group.name)),
                        )
                        .child(
                            div().text_size(px(7.0)).text_color(theme.muted_foreground)
                                .child(if open { "▴" } else { "▾" }),
                        ),
                ),
        )
        .on_open_change(move |is_open, _, cx| {
            state::set_ext_open(
                if *is_open { Some(group_key.clone()) } else { None },
                cx,
            )
        })
        .content(move |_, _, cx| {
            let theme = cx.theme().clone();
            let snap = state::ribbon_snapshot(cx);
            ext_panel(ext, &snap, &theme)
        });

    div().id(ElementId::Name(format!("{key}-ext").into())).child(pop)
}

/// 扩展面板内容：ext_sub 指向某工具时显示其子选项（带返回行），否则 4 列工具网格。
fn ext_panel(ext: &'static [ExtTool], snap: &RibbonSnapshot, theme: &Theme) -> AnyElement {
    if let Some(sub_cmd) = &snap.ext_sub {
        if let Some(tool) = ext.iter().find(|t| t.cmd == *sub_cmd) {
            if !tool.options.is_empty() {
                let mut rows: Vec<AnyElement> = Vec::new();
                rows.push(
                    div()
                        .id("ext-back")
                        .h_flex()
                        .h(px(24.0))
                        .px_2()
                        .items_center()
                        .gap_1p5()
                        .text_size(px(11.0))
                        .text_color(theme.muted_foreground)
                        .hover(|s| s.bg(rgba(0xffffff0d)))
                        .cursor_pointer()
                        .on_click(|_, _, cx| state::set_ext_sub(None, cx))
                        .child("‹")
                        .child(SharedString::from(tool.label))
                        .into_any_element(),
                );
                for (label, cmd) in tool.options {
                    rows.push(
                        div()
                            .id(ElementId::Name(format!("ext-opt-{cmd}").into()))
                            .h_flex()
                            .h(px(24.0))
                            .px_2()
                            .items_center()
                            .text_size(px(11.0))
                            .text_color(theme.foreground)
                            .hover(|s| s.bg(rgba(0x0696d726)))
                            .cursor_pointer()
                            .on_click(move |_, _, cx| {
                                state::select_dropdown(tool.cmd, cmd, cx);
                                state::set_ext_sub(None, cx);
                                state::set_ext_open(None, cx);
                            })
                            .child(SharedString::from(*label))
                            .into_any_element(),
                    );
                }
                return div().v_flex().w(px(150.0)).p_1().children(rows).into_any_element();
            }
        }
    }

    // 工具网格：每行 4 格，face 可直接执行；带子选项的工具下方多一条细箭头
    let mut rows: Vec<Vec<AnyElement>> = vec![Vec::new()];
    for (k, t) in ext.iter().enumerate() {
        if rows.last().is_some_and(|r| r.len() >= 4) {
            rows.push(Vec::new());
        }
        let row = rows.last_mut().unwrap();
        let cell = div()
            .flex().flex_col()
            .w(px(46.0))
            .items_center()
            .gap_0p5()
            .child(
                attach_tooltip(
                    div()
                        .id(ElementId::Name(format!("ext-{k}").into()))
                        .size(px(26.0))
                        .rounded_sm()
                        .flex().items_center().justify_center()
                        .cursor_pointer()
                        .when(snap.is_active(t.cmd), |el| el.bg(rgba(0x0696d74d)))
                        .hover(|s| s.bg(rgba(0x0696d726)))
                        .on_click(move |_, _, cx| state::run_tool(t.cmd, cx))
                        .child(img(icons::icon(t.icon)).size(px(16.0)).flex_shrink_0()),
                    tip_text(t.label, t.cmd),
                ),
            )
            .when(!t.options.is_empty(), |el| {
                el.child(
                    div()
                        .id(ElementId::Name(format!("ext-{k}-opt").into()))
                        .w_full()
                        .h(px(14.0))
                        .flex().justify_center().items_center()
                        .text_size(px(9.0))
                        .text_color(theme.muted_foreground)
                        .hover(|s| s.text_color(theme.accent))
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            state::set_ext_sub(Some(t.cmd.to_string()), cx)
                        })
                        .child("▾"),
                )
            })
            .child(
                div().text_size(px(9.0)).text_color(theme.muted_foreground)
                    .child(SharedString::from(t.label)),
            );
        row.push(cell.into_any_element());
    }

    div()
        .v_flex().w(px(210.0)).p_1().gap_1().children(
        rows.into_iter().map(|row| div().flex().flex_row().gap_1().children(row)),
    )
    .into_any_element()
}

// ---------------------------------------------------------------------------
// Flyout（折叠档）
// ---------------------------------------------------------------------------

fn flyout_button(
    key: &str,
    group: &'static RibbonGroup,
    snap: &RibbonSnapshot,
    filter: &Entity<InputState>,
    theme: &Theme,
) -> Stateful<gpui_kit::Div> {
    // 代表工具：组内最后贴住的工具，默认第一个（对应 last_panel_tool）
    let (face_icon, _face_cmd) = representative(group, snap);
    let pop_id = ElementId::Name(format!("{key}-flyout").into());
    let trig_id = ElementId::Name(format!("{key}-flybtn").into());
    let label = group.name;
    let filter = filter.clone();

    let pop = Popover::new(pop_id)
        .anchor(Anchor::TopLeft)
        .trigger(
            Button::new(trig_id)
                .ghost()
                .w(px(FLYOUT_W))
                .h_full()
                .child(
                    div().flex().flex_col().h_full()
                        .child(
                            div().flex_1().flex().items_center().justify_center()
                                .child(img(icons::icon(face_icon)).size(px(24.0))),
                        )
                        .child(
                            div().h(px(13.0)).flex().justify_center().items_center().gap_0p5()
                                .text_size(px(10.5))
                                .text_color(theme.foreground)
                                .child(SharedString::from(label))
                                .child(
                                    div().text_size(px(7.0)).text_color(theme.muted_foreground)
                                        .child("▾"),
                                ),
                        ),
                ),
        )
        .content(move |_, _, cx| {
            let theme = cx.theme().clone();
            let snap = state::ribbon_snapshot(cx);
            flyout_panel(group, &snap, &filter, &theme)
        });

    div()
        .id(ElementId::Name(format!("{key}-flywrap").into()))
        .w(px(FLYOUT_W))
        .h_full()
        .flex()
        .child(pop)
}

fn representative(group: &RibbonGroup, snap: &RibbonSnapshot) -> (&'static str, &'static str) {
    for item in &group.items {
        match item {
            RibbonItem::Large { icon, cmd, .. } | RibbonItem::SplitLarge { icon, cmd, .. } => {
                if snap.is_active(cmd) {
                    return (icon, cmd);
                }
            }
            RibbonItem::Column { tools } => {
                if let Some(t) = tools.iter().find(|t| snap.is_active(t.cmd)) {
                    return (t.icon, t.cmd);
                }
            }
            RibbonItem::SplitColumn { tools } => {
                if let Some(t) = tools.iter().find(|t| snap.is_active(t.cmd)) {
                    return (t.icon, t.cmd);
                }
            }
            RibbonItem::LabeledSplit { icon, cmd, .. } => {
                if snap.is_active(cmd) {
                    return (icon, cmd);
                }
            }
            RibbonItem::PasteGroup { icon, cmd, .. } => {
                if snap.is_active(cmd) {
                    return (icon, cmd);
                }
            }
            RibbonItem::ComboRow { .. }
            | RibbonItem::CharFlow { .. }
            | RibbonItem::SpinRow { .. }
            | RibbonItem::ActionRow { .. } => {}
            RibbonItem::LayerCombo => {}
        }
    }
    first_icon_cmd(group).unwrap_or(("line.svg", "line"))
}

fn first_icon_cmd(group: &RibbonGroup) -> Option<(&'static str, &'static str)> {
    for item in &group.items {
        match item {
            RibbonItem::Large { icon, cmd, .. } | RibbonItem::SplitLarge { icon, cmd, .. } => {
                return Some((icon, cmd))
            }
            RibbonItem::Column { tools } => {
                if let Some(t) = tools.first() {
                    return Some((t.icon, t.cmd));
                }
            }
            RibbonItem::SplitColumn { tools } => {
                if let Some(t) = tools.first() {
                    return Some((t.icon, t.cmd));
                }
            }
            RibbonItem::LabeledSplit { icon, cmd, .. } => return Some((icon, cmd)),
            RibbonItem::PasteGroup { icon, cmd, .. } => return Some((icon, cmd)),
            RibbonItem::ComboRow { .. } => return Some(("ui/gear.svg", "combo")),
            RibbonItem::CharFlow { buttons, .. } => {
                return buttons.first().map(|b| ("text.svg", b.cmd));
            }
            RibbonItem::SpinRow { items, .. } => {
                return items.first().map(|s| ("status/lwt.svg", s.cmd));
            }
            RibbonItem::ActionRow { items, .. } => {
                return items.first().map(|a| ("ui/gear.svg", a.cmd));
            }
            RibbonItem::LayerCombo => return Some(("layers/panel.svg", "layer")),
        }
    }
    None
}

/// 飞出面板内容：整组工具按每行 3 格显式分行。
fn flyout_panel(
    group: &'static RibbonGroup,
    snap: &RibbonSnapshot,
    filter: &Entity<InputState>,
    theme: &Theme,
) -> AnyElement {
    let mut rows: Vec<Vec<AnyElement>> = vec![Vec::new()];
    let mut k = 0usize;
    for item in &group.items {
        if rows.last().is_some_and(|r| r.len() >= 3) {
            rows.push(Vec::new());
        }
        let row = rows.last_mut().unwrap();
        let cell: AnyElement = match item {
            RibbonItem::Large { icon, label, cmd } => {
                large_tool(&format!("fly{k}"), icon, label, cmd, snap, theme).into_any_element()
            }
            RibbonItem::SplitLarge { dd, icon, label, cmd, menu } => {
                split_large(&format!("fly{k}"), dd, icon, label, cmd, menu, snap, theme)
                    .into_any_element()
            }
            RibbonItem::Column { tools } => {
                let mut row = div().flex().flex_row().gap_px();
                for (j, t) in tools.iter().enumerate() {
                    row = row.child(small_tool(&format!("fly{k}s{j}"), t.icon, t.label, t.cmd, snap, theme));
                }
                row.into_any_element()
            }
            RibbonItem::SplitColumn { tools } => {
                let mut row = div().flex().flex_row().gap_px();
                for (j, t) in tools.iter().enumerate() {
                    row = row.child(split_small(&format!("fly{k}s{j}"), t, snap, theme));
                }
                row.into_any_element()
            }
            RibbonItem::LabeledSplit { icon, label, cmd, menu } => {
                labeled_split(&format!("fly{k}"), icon, label, cmd, menu, snap, theme)
                    .into_any_element()
            }
            RibbonItem::LayerCombo => layer_combo(&format!("fly{k}"), snap, filter, theme),
            RibbonItem::PasteGroup { icon, label, cmd, sides } => {
                crate::office_widgets::render_paste_group(
                    &format!("fly{k}p"),
                    icon,
                    label,
                    cmd,
                    sides,
                    &ot(),
                )
                .into_any_element()
            }
            RibbonItem::ComboRow { items } => {
                crate::office_widgets::render_combo_row(&format!("fly{k}c"), items, &ot())
                    .into_any_element()
            }
            RibbonItem::CharFlow { buttons } => {
                crate::office_widgets::render_char_flow(&format!("fly{k}f"), buttons, &ot())
                    .into_any_element()
            }
            RibbonItem::SpinRow { items } => {
                crate::office_widgets::render_spin_row(&format!("fly{k}s"), items, &ot())
                    .into_any_element()
            }
            RibbonItem::ActionRow { items } => {
                crate::office_widgets::render_action_row(&format!("fly{k}a"), items, &ot())
                    .into_any_element()
            }
        };
        row.push(cell);
        k += 1;
    }

    div()
        .v_flex().w(px(230.0)).p_2().gap_1().children(
        rows.into_iter().map(|row| {
            div().flex().flex_row().gap_1().items_start().children(row)
        }),
    )
    .into_any_element()
}

// ---------------------------------------------------------------------------
// 应用按钮菜单（Office Backstage 风格全屏面板）
// ---------------------------------------------------------------------------

/// 最近文件（demo 数据）：(文件名, 位置与时间)。
const RECENT_FILES: &[(&str, &str)] = &[
    ("厂房平面图.dwg", "D:/projects/plant — 昨天 18:42"),
    ("办公楼给排水.dwg", "D:/projects/mep — 周五 09:15"),
    ("demo_block_test.dwg", "D:/tmp — 上周四"),
    ("site_plan_v2.dwg", "C:/Users/lilu/Documents — 3 月 12 日"),
];

impl RibbonBar {
    /// 渲染 Backstage 全屏面板：左列主题色大按钮（新建/打开/保存/另存为/打印，
    /// 底部"选项"），右列最近文件列表。Esc / 返回 / 再点"文件"关闭。
    pub fn render_app_menu(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let ot = office_theme(cx);

        let item =
            |cx: &mut Context<Self>, key: &'static str, icon: &'static str, label: &'static str, cmd: &'static str| {
                div()
                    .id(ElementId::Name(key.into()))
                    .h(px(34.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .text_size(px(12.5))
                    .text_color(rgba(0xffffff))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba(0xffffff33)))
                    .active(|s| s.bg(rgba(0xffffff4d)))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        state::run_command(cmd, cx);
                        this.close_app_menu(cx);
                    }))                    .child(img(icons::icon(icon)).size(px(15.0)).flex_shrink_0())
                    .child(label)
            };

        let recent_row = |cx: &mut Context<Self>, key: String, name: &'static str, meta: &'static str| {
            div()
                .id(ElementId::Name(key.into()))
                .h(px(44.0))
                .w_full()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .rounded_sm()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(0x0696d71a)))
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    state::run_command("open", cx);
                    this.close_app_menu(cx);
                }))
                .child(img(icons::icon("ui/doc.svg")).size(px(18.0)).flex_shrink_0())
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(ot.text)
                        .child(SharedString::from(name)),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(ot.group_name)
                        .child(SharedString::from(meta)),
                )
        };

        div()
            .id("app-menu")
            .flex_1()
            .min_h_0()
            .flex()
            .bg(ot.tool_bg)
            // 左列：主题色按钮区
            .child(
                div()
                    .v_flex()
                    .w(px(200.0))
                    .h_full()
                    .flex_shrink_0()
                    .bg(ot.accent)
                    .pt_1p5()
                    .child(
                        div()
                            .id("backstage-back")
                            .h(px(38.0))
                            .w_full()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_4()
                            .text_size(px(12.5))
                            .text_color(rgba(0xffffff))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xffffff33)))
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.close_app_menu(cx);
                            }))
                            .child(div().text_size(px(14.0)).child("‹"))
                            .child("返回"),
                    )
                    .child(item(cx, "bs-new", "ui/doc_new.svg", "新建", "new"))
                    .child(item(cx, "bs-open", "ui/folder_open.svg", "打开", "open"))
                    .child(item(cx, "bs-save", "ui/save.svg", "保存", "save"))
                    .child(item(cx, "bs-saveas", "ui/file_export.svg", "另存为", "saveas"))
                    .child(item(cx, "bs-plot", "ui/print.svg", "打印", "plot"))
                    .child(div().flex_1())
                    .child(
                        div()
                            .h(px(1.0))
                            .w_full()
                            .bg(rgba(0xffffff33))
                            .mb_1p5(),
                    )
                    .child(item(cx, "bs-options", "ui/gear.svg", "选项", "options"))
                    .pb_2(),
            )
            // 右列：最近文件
            .child(
                div()
                    .id("backstage-recent")
                    .v_flex()
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .pt_3()
                    .child(
                        div()
                            .px_6()
                            .pb_2()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(ot.text)
                            .child("最近"),
                    )
                    .px_4()
                    .children(
                        RECENT_FILES
                            .iter()
                            .enumerate()
                            .map(|(k, (name, meta))| {
                                recent_row(cx, format!("recent-{k}"), name, meta)
                            }),
                    ),
            )
    }

    /// 关闭应用菜单：状态在 CadModel 上，经 AppEvent 通知 Workspace 切回 dock 区。
    pub fn close_app_menu(&mut self, cx: &mut Context<Self>) {
        state::set_app_menu_open(false, cx);
    }
}
