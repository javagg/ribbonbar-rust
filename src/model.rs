//! CAD 应用状态模型（纯逻辑，UI 无关）：gpui 版与 egui 版共用。
//!
//! 参照 OpenCADStudio 的 Ribbon 状态模型：
//! - `last_cmd`：每个分裂下拉记录最后选择的命令（按钮面随之切换、菜单勾选联动）
//! - `active_tool`：当前"贴着的"工具，对应按钮保持高亮
//! - undo/redo 栈：快速访问的历史下拉可一次回退/重做 N 步
//! - 图层数据：图层下拉的搜索过滤与行内开关
//! - `current_cmd`：需要镜像文档状态的下拉由外部同步"当前项"而不执行命令

use std::collections::HashMap;

/// 图层下拉的行数据（对应 OpenCADStudio 的 LayerInfo）。
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub name: String,
    pub color: u32,
    pub visible: bool,
    pub frozen: bool,
    pub locked: bool,
}

/// Ribbon 渲染期间一次性取出的快照（owned，避免借用冲突）。
pub struct RibbonSnapshot {
    pub last_cmd: HashMap<String, String>,
    pub current_cmd: HashMap<String, String>,
    pub active_tool: Option<String>,
    pub osnap: bool,
    pub ortho: bool,
    pub polar: bool,
    pub lwt: bool,
    pub viewcube: bool,
    pub ucs_icon: bool,
    pub undo_labels: Vec<String>,
    pub redo_labels: Vec<String>,
    pub layers: Vec<LayerInfo>,
    pub active_layer: String,
    pub ext_sub: Option<String>,
    pub ext_open: Option<String>,
    pub app_menu_open: bool,
    /// 功能区最小化（FR IsMinimized，DisplayOptions/双击/Ctrl+F1 切换）。
    pub ribbon_minimized: bool,
    /// 最小化时点标签的临时展开层（点外部收回）。
    pub ribbon_transient_open: bool,
}

impl RibbonSnapshot {
    /// 某下拉的"当前项"命令：镜像状态优先，其次最后执行，最后 None（用默认）。
    pub fn current_of(&self, dd: &str) -> Option<&str> {
        self.current_cmd
            .get(dd)
            .or_else(|| self.last_cmd.get(dd))
            .map(|s| s.as_str())
    }

    /// 按钮/命令是否处于"贴住"状态（映射持久开关）。
    pub fn is_active(&self, cmd: &str) -> bool {
        match cmd {
            "osnap" => self.osnap,
            "ortho" => self.ortho,
            "polar" => self.polar,
            "lwt" => self.lwt,
            "navvcube" => self.viewcube,
            "ucsicon" => self.ucs_icon,
            "persp" => !self.ortho,
            id => self.active_tool.as_deref() == Some(id),
        }
    }
}

#[derive(Default)]
pub struct CadModel {
    pub log: Vec<String>,
    pub cursor: Option<(f64, f64)>,
    pub osnap: bool,
    pub ortho: bool,
    pub polar: bool,
    pub lwt: bool,
    pub viewcube: bool,
    pub ucs_icon: bool,
    pub seq: usize,
    /// 分裂下拉 id → 最后选择的命令。按钮面与菜单勾选联动。
    pub last_cmd: HashMap<String, String>,
    /// 当前贴住的工具 id；对应按钮保持高亮，再次点击取消。
    pub active_tool: Option<String>,
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
    /// 需要镜像文档状态的下拉（如视觉样式）由外部同步"当前项"而不执行命令。
    pub current_cmd: HashMap<String, String>,
    pub layers: Vec<LayerInfo>,
    pub active_layer: String,
    /// 图层下拉的搜索过滤（关闭时由前端清空）。
    pub layer_filter: String,
    /// 组标题扩展面板里打开子选项的工具命令（None = 显示工具网格）。
    pub ext_sub: Option<String>,
    /// 打开中的扩展面板组 key（箭头方向与高亮）。
    pub ext_open: Option<String>,
    /// 应用按钮菜单（Backstage 全屏面板）是否展开。
    pub app_menu_open: bool,
    /// 功能区最小化（FR IsMinimized）。
    pub ribbon_minimized: bool,
    /// 最小化时点标签的临时展开层。
    pub ribbon_transient_open: bool,
}

fn push_undo(s: &mut CadModel, cmd: &str) {
    s.undo_stack.push(cmd.to_string());
    s.redo_stack.clear();
}

impl CadModel {
    pub fn new() -> Self {
        let mut m = Self {
            osnap: true,
            polar: true,
            lwt: true,
            viewcube: true,
            ucs_icon: true,
            ..Default::default()
        };
        m.log.push("OpenCAD Demo — gpui-kit 0.7.0 UI 框架演示".into());
        m.log
            .push("输入 LINE / CIRCLE / MOVE 等命令后回车，或点击功能区按钮。".into());
        m.layers = vec![
            LayerInfo { name: "0".into(), color: 0xf2f2f2, visible: true, frozen: false, locked: false },
            LayerInfo { name: "Defpoints".into(), color: 0x808080, visible: true, frozen: false, locked: true },
            LayerInfo { name: "Walls".into(), color: 0xe53935, visible: true, frozen: false, locked: false },
            LayerInfo { name: "Doors".into(), color: 0x1e88e5, visible: true, frozen: false, locked: false },
            LayerInfo { name: "Dimensions".into(), color: 0x43a047, visible: true, frozen: false, locked: false },
            LayerInfo { name: "Text".into(), color: 0xfdd835, visible: false, frozen: false, locked: false },
            LayerInfo { name: "Furniture".into(), color: 0x8e24aa, visible: true, frozen: true, locked: false },
        ];
        m.active_layer = "Walls".into();
        m
    }

    pub fn snapshot(&self) -> RibbonSnapshot {
        RibbonSnapshot {
            last_cmd: self.last_cmd.clone(),
            current_cmd: self.current_cmd.clone(),
            active_tool: self.active_tool.clone(),
            osnap: self.osnap,
            ortho: self.ortho,
            polar: self.polar,
            lwt: self.lwt,
            viewcube: self.viewcube,
            ucs_icon: self.ucs_icon,
            undo_labels: self.undo_stack.iter().rev().cloned().collect(),
            redo_labels: self.redo_stack.iter().rev().cloned().collect(),
            layers: self.layers.clone(),
            active_layer: self.active_layer.clone(),
            ext_sub: self.ext_sub.clone(),
            ext_open: self.ext_open.clone(),
            app_menu_open: self.app_menu_open,
            ribbon_minimized: self.ribbon_minimized,
            ribbon_transient_open: self.ribbon_transient_open,
        }
    }

    fn log_cmd(&mut self, cmd: &str) {
        self.seq += 1;
        self.log.push(format!("{:>02}  命令: {}", self.seq, cmd));
        if self.log.len() > 300 {
            self.log.remove(0);
        }
    }

    /// 普通命令：记录日志 + 进撤销栈。
    pub fn run_command(&mut self, cmd: &str) {
        self.log_cmd(cmd);
        push_undo(self, cmd);
    }

    /// 工具按钮：除记录命令外，把该工具"贴住"（按钮持续高亮，再点取消）。
    /// 开关类命令走 toggle 而不进栈。
    pub fn run_tool(&mut self, cmd: &'static str) {
        if Self::is_toggle_cmd(cmd) {
            self.toggle(cmd);
            return;
        }
        self.log_cmd(cmd);
        push_undo(self, cmd);
        self.active_tool = match self.active_tool.as_deref() {
            Some(cur) if cur == cmd => None,
            _ => Some(cmd.to_string()),
        };
    }

    /// 分裂下拉选中一项：记录 last_cmd（按钮面切换、勾选联动）并执行。
    pub fn select_dropdown(&mut self, dd: &'static str, cmd: &'static str) {
        self.last_cmd.insert(dd.to_string(), cmd.to_string());
        self.log_cmd(cmd);
        push_undo(self, cmd);
        self.active_tool = Some(cmd.to_string());
    }

    /// 外部状态同步"当前项"：只挪勾选，不执行命令、不进栈。
    pub fn set_dropdown_current(&mut self, dd: &'static str, cmd: &'static str) {
        self.current_cmd
            .entry(dd.to_string())
            .or_insert_with(|| cmd.to_string());
    }

    pub fn is_toggle_cmd(cmd: &str) -> bool {
        matches!(cmd, "osnap" | "ortho" | "polar" | "lwt" | "navvcube" | "ucsicon")
    }

    pub fn toggle(&mut self, kind: &str) {
        match kind {
            "osnap" => self.osnap = !self.osnap,
            "ortho" => self.ortho = !self.ortho,
            "polar" => self.polar = !self.polar,
            "lwt" => self.lwt = !self.lwt,
            "navvcube" => self.viewcube = !self.viewcube,
            "ucsicon" => self.ucs_icon = !self.ucs_icon,
            _ => {}
        }
    }

    // ── Undo / Redo（含一次多步）───────────────────────────────────────────

    pub fn undo(&mut self) {
        self.step(&mut |s| s.undo_stack.pop(), &mut |s, c| s.redo_stack.push(c));
    }

    pub fn redo(&mut self) {
        self.step(&mut |s| s.redo_stack.pop(), &mut |s, c| s.undo_stack.push(c));
    }

    pub fn undo_many(&mut self, n: usize) {
        for _ in 0..n {
            self.step(&mut |s| s.undo_stack.pop(), &mut |s, c| s.redo_stack.push(c));
        }
    }

    pub fn redo_many(&mut self, n: usize) {
        for _ in 0..n {
            self.step(&mut |s| s.redo_stack.pop(), &mut |s, c| s.undo_stack.push(c));
        }
    }

    fn step(
        &mut self,
        pop: &mut dyn FnMut(&mut Self) -> Option<String>,
        push: &mut dyn FnMut(&mut Self, String),
    ) {
        if let Some(cmd) = pop(self) {
            push(self, cmd.clone());
            self.log_cmd(&cmd);
        }
    }

    // ── 图层 ────────────────────────────────────────────────────────────────

    pub fn layer_select(&mut self, name: String) {
        self.active_layer = name;
        self.log_cmd(&format!("layer {}", self.active_layer));
    }

    pub fn layer_toggle(&mut self, index: usize, kind: &str) {
        if let Some(l) = self.layers.get_mut(index) {
            match kind {
                "visible" => l.visible = !l.visible,
                "frozen" => l.frozen = !l.frozen,
                "locked" => l.locked = !l.locked,
                _ => {}
            }
        }
    }

    pub fn set_cursor(&mut self, pos: Option<(f64, f64)>) {
        if self.cursor != pos {
            self.cursor = pos;
        }
    }

    // ── 组标题扩展面板 ──────────────────────────────────────────────────────

    pub fn set_ext_sub(&mut self, sub: Option<String>) {
        self.ext_sub = sub;
    }

    pub fn set_ext_open(&mut self, key: Option<String>) {
        if key.is_none() {
            self.ext_sub = None;
        }
        self.ext_open = key;
    }

    /// 展开/关闭应用按钮菜单（Backstage）。
    pub fn set_app_menu_open(&mut self, open: bool) {
        self.app_menu_open = open;
    }

    /// 切换功能区最小化（展开时清临时层）。
    pub fn set_ribbon_minimized(&mut self, minimized: bool) {
        self.ribbon_minimized = minimized;
        if minimized {
            self.ribbon_transient_open = false;
        }
    }

    /// 最小化时点标签的临时展开层。
    pub fn set_ribbon_transient_open(&mut self, open: bool) {
        self.ribbon_transient_open = open;
    }

    /// 点击功能区内任意工具/标签（收最小化临时层的锚点之一）。
    pub fn ribbon_click(&mut self) {
        self.ribbon_transient_open = false;
    }
}
