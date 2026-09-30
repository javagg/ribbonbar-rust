//! Ribbon 数据模型与自适应降级算法（纯数据 + 纯函数，UI 无关）。
//! gpui 版与 egui 版共用；结构与度量对照 OpenCADStudio（registry.rs / collapse.rs）。

// ---------------------------------------------------------------------------
// 数据模型
// ---------------------------------------------------------------------------

/// 分裂按钮的菜单项：(图标, 标签, 命令)
pub type MenuEntry = (&'static str, &'static str, &'static str);
/// 小工具：图标 + 文字标签（所有命令必须有文字标签，见 HANDOFF §5 用户准则）。
pub struct Tool {
    pub icon: &'static str,
    pub label: &'static str,
    pub cmd: &'static str,
}
/// 扩展面板里工具的子选项：(标签, 命令)
pub type ExtOption = (&'static str, &'static str);

/// 带菜单的小工具（下拉 id 即 cmd，命令名天然唯一）。
pub struct SmallSplit {
    pub icon: &'static str,
    pub label: &'static str,
    pub cmd: &'static str,
    pub menu: &'static [MenuEntry],
}

/// 组标题扩展面板里的工具：可带子选项列表。
pub struct ExtTool {
    pub icon: &'static str,
    pub label: &'static str,
    pub cmd: &'static str,
    pub options: &'static [ExtOption],
}

pub enum RibbonItem {
    /// 大按钮：图标在上、标签在下。
    Large {
        icon: &'static str,
        label: &'static str,
        cmd: &'static str,
    },
    /// 大分裂按钮：面显示 dd 的当前变体，底部整宽居中箭头展开菜单。
    SplitLarge {
        dd: &'static str,
        icon: &'static str,
        label: &'static str,
        cmd: &'static str,
        menu: &'static [MenuEntry],
    },
    /// 小工具行：图标上/文字下的带标签小按钮，横排一行。
    Column { tools: &'static [Tool] },
    /// 小工具行，个别项可带下拉箭头。
    SplitColumn { tools: &'static [SmallSplit] },
    /// 横向按钮：左图标 + 右文字 + 小箭头（下拉 id 即 cmd）。
    LabeledSplit {
        icon: &'static str,
        label: &'static str,
        cmd: &'static str,
        menu: &'static [MenuEntry],
    },
    /// 图层下拉组合框（搜索 + 行内开关 + 活动层）。
    LayerCombo,
    /// Office 剪贴板形态：大按钮 + 右侧竖排横排小按钮（图标+文字，可带▾）。
    PasteGroup {
        icon: &'static str,
        label: &'static str,
        cmd: &'static str,
        sides: &'static [(&'static str, &'static str, &'static str, bool)],
    },
    /// Office 组合框行：标签/值 + ▾ 一体（如 字体/字号）。
    ComboRow { items: &'static [Combo] },
    /// Office 流式小按钮行：纯字符自绘按钮（B/I/U/abc…）。
    CharFlow { buttons: &'static [CharBtn] },
    /// Office 数值微调组：标签 + 值 + ▴▾。
    SpinRow { items: &'static [SpinEdit] },
    /// 横排带文字按钮（右箭头下拉），可禁用。
    ActionRow { items: &'static [ActionBtn] },
}

/// Office 组合框：一体式（值 + ▾）。
pub struct Combo {
    pub label: &'static str,
    pub value: &'static str,
    pub width: f32,
    pub options: &'static [&'static str],
    pub cmd: &'static str,
}

/// Office 流式字符按钮：(字符, 命令, 是否切换型)。
pub struct CharBtn {
    pub ch: &'static str,
    pub cmd: &'static str,
    pub toggle: bool,
}

/// Office 横排带文字按钮：(文字, 命令, 是否禁用)。
pub struct ActionBtn {
    pub label: &'static str,
    pub cmd: &'static str,
    pub disabled: bool,
}

/// Office 数值微调框：(标签, 值, 宽度)。
pub struct SpinEdit {
    pub label: &'static str,
    pub value: &'static str,
    pub width: f32,
    pub cmd: &'static str,
}

pub struct RibbonGroup {
    pub name: &'static str,
    pub items: Vec<RibbonItem>,
    /// 组标题扩展面板（None = 标题不可点）。
    pub ext: Option<&'static [ExtTool]>,
    /// 对话框启动器（FR IsLauncherVisible 默认 false，按组显式启用）。
    pub launcher: bool,
}

pub struct RibbonTab {
    pub name: &'static str,
    pub groups: Vec<RibbonGroup>,
}

/// 绘图组扩展：放不下面板的绘制工具（对应 OpenCADStudio draw_extension）。
const DRAW_EXT: &[ExtTool] = &[
    ExtTool { icon: "spline_cv.svg", label: "样条CV", cmd: "splinecv", options: &[] },
    ExtTool { icon: "divide.svg", label: "定数等分", cmd: "divide", options: &[] },
    ExtTool { icon: "measure.svg", label: "定距等分", cmd: "measure", options: &[] },
    ExtTool { icon: "region.svg", label: "面域", cmd: "region", options: &[] },
    ExtTool {
        icon: "boundary.svg",
        label: "边界",
        cmd: "boundary",
        options: &[("拾取内部点", "boundary_pick"), ("新建边界集", "boundary_new")],
    },
];

/// 修改组扩展。
const MODIFY_EXT: &[ExtTool] = &[
    ExtTool { icon: "modify_align.svg", label: "对齐", cmd: "align", options: &[] },
    ExtTool { icon: "modify_lengthen.svg", label: "加长", cmd: "lengthen", options: &[] },
    ExtTool { icon: "modify_pedit.svg", label: "编辑多段线", cmd: "pedit", options: &[] },
    ExtTool { icon: "modify_breakatpoint.svg", label: "打断于点", cmd: "breakatpoint", options: &[] },
];

pub fn tabs() -> &'static Vec<RibbonTab> {
    static TABS: std::sync::OnceLock<Vec<RibbonTab>> = std::sync::OnceLock::new();
    TABS.get_or_init(|| {
        vec![
            RibbonTab {
                name: "绘图",
                groups: vec![
                    RibbonGroup {
                        name: "剪贴板",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::PasteGroup {
                                icon: "paste.svg",
                                label: "粘贴",
                                cmd: "paste",
                                sides: &[
                                    ("modify_cut.svg", "剪切", "cut", true),
                                    ("copy_clip.svg", "复制", "copy", false),
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "文字",
                        ext: None,
                        launcher: true,
                        items: vec![
                            RibbonItem::ComboRow {
                                items: &[
                                    Combo { label: "字体", value: "SimSun", width: 96.0, options: &["SimSun", "Arial", "Courier New"], cmd: "textfont" },
                                    Combo { label: "字号", value: "2.5", width: 44.0, options: &["2.5", "3.5", "5.0"], cmd: "textheight" },
                                ],
                            },
                            RibbonItem::CharFlow {
                                buttons: &[
                                    CharBtn { ch: "B", cmd: "bold", toggle: true },
                                    CharBtn { ch: "I", cmd: "italic", toggle: true },
                                    CharBtn { ch: "U", cmd: "underline", toggle: true },
                                    CharBtn { ch: "abc", cmd: "strikethru", toggle: false },
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "栅格",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::SpinRow {
                                items: &[
                                    SpinEdit { label: "X 间距", value: "10", width: 46.0, cmd: "gridx" },
                                    SpinEdit { label: "Y 间距", value: "10", width: 46.0, cmd: "gridy" },
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "绘图",
                        ext: Some(DRAW_EXT),
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "line.svg", label: "直线", cmd: "line" },
                            RibbonItem::Large { icon: "polyline.svg", label: "多段线", cmd: "pline" },
                            RibbonItem::SplitLarge {
                                dd: "circle",
                                icon: "circle/circle_cr.svg",
                                label: "圆",
                                cmd: "circle",
                                menu: &[
                                    ("circle/circle_cr.svg", "圆心、半径", "circle"),
                                    ("circle/circle_cd.svg", "圆心、直径", "circle_cd"),
                                    ("circle/circle_2p.svg", "两点", "circle_2p"),
                                    ("circle/circle_3p.svg", "三点", "circle_3p"),
                                    ("circle/circle_ttr.svg", "相切、相切、半径", "circle_ttr"),
                                ],
                            },
                            RibbonItem::SplitLarge {
                                dd: "arc",
                                icon: "arc/arc_3p.svg",
                                label: "圆弧",
                                cmd: "arc",
                                menu: &[
                                    ("arc/arc_3p.svg", "三点", "arc_3p"),
                                    ("arc/arc_sce.svg", "起点、圆心、端点", "arc_sce"),
                                    ("arc/arc_sea.svg", "起点、端点、角度", "arc_sea"),
                                    ("arc/arc_scl.svg", "圆心、起点、长度", "arc_scl"),
                                ],
                            },
                            RibbonItem::SplitLarge {
                                dd: "rectang",
                                icon: "shapes/rect.svg",
                                label: "矩形",
                                cmd: "rectang",
                                menu: &[
                                    ("shapes/rect.svg", "两点", "rectang"),
                                    ("shapes/rect_cen.svg", "中心", "rectang_center"),
                                    ("shapes/rect_rot.svg", "旋转", "rectang_rotated"),
                                    ("shapes/poly_i.svg", "内接多边形", "polygon_i"),
                                    ("shapes/poly_c.svg", "外切多边形", "polygon_c"),
                                ],
                            },
                            RibbonItem::SplitLarge {
                                dd: "ellipse",
                                icon: "ellipse/ellipse_ctr.svg",
                                label: "椭圆",
                                cmd: "ellipse",
                                menu: &[
                                    ("ellipse/ellipse_ctr.svg", "中心点", "ellipse"),
                                    ("ellipse/ellipse_axis.svg", "轴、端点", "ellipse_axis"),
                                    ("ellipse/ellipse_arc.svg", "椭圆弧", "ellipse_arc"),
                                ],
                            },
                            RibbonItem::SplitColumn {
                                tools: &[
                                    SmallSplit {
                                        icon: "point.svg",
                                        label: "点",
                                        cmd: "point",
                                        menu: &[
                                            ("point.svg", "单点", "point"),
                                            ("multipoint.svg", "多点", "multipoint"),
                                        ],
                                    },
                                    SmallSplit { icon: "ray.svg", label: "射线", cmd: "ray", menu: &[] },
                                    SmallSplit { icon: "xline.svg", label: "构造线", cmd: "xline", menu: &[] },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "spline.svg", label: "样条曲线", cmd: "spline" },
                                    Tool { icon: "donut.svg", label: "圆环", cmd: "donut" },
                                    Tool { icon: "helix.svg", label: "螺旋线", cmd: "helix" },
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "修改",
                        ext: Some(MODIFY_EXT),
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "move.svg", label: "移动", cmd: "move" },
                            RibbonItem::Large { icon: "copy_clip.svg", label: "复制", cmd: "copy" },
                            RibbonItem::Large { icon: "rotate.svg", label: "旋转", cmd: "rotate" },
                            RibbonItem::Large { icon: "scale.svg", label: "缩放", cmd: "scale" },
                            RibbonItem::Large { icon: "mirror.svg", label: "镜像", cmd: "mirror" },
                            RibbonItem::SplitLarge {
                                dd: "array",
                                icon: "array_rect.svg",
                                label: "阵列",
                                cmd: "array",
                                menu: &[
                                    ("array_rect.svg", "矩形阵列", "array_rect"),
                                    ("array_path.svg", "路径阵列", "array_path"),
                                    ("array_polar.svg", "环形阵列", "array_polar"),
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "trim.svg", label: "修剪", cmd: "trim" },
                                    Tool { icon: "extend.svg", label: "延伸", cmd: "extend" },
                                    Tool { icon: "offset.svg", label: "偏移", cmd: "offset" },
                                ],
                            },
                            RibbonItem::SplitColumn {
                                tools: &[
                                    SmallSplit {
                                        icon: "fillet.svg",
                                        label: "圆角",
                                        cmd: "fillet",
                                        menu: &[
                                            ("fillet.svg", "圆角", "fillet"),
                                            ("model/fillet.svg", "圆角边", "fillet_edge"),
                                        ],
                                    },
                                    SmallSplit {
                                        icon: "chamfer.svg",
                                        label: "倒角",
                                        cmd: "chamfer",
                                        menu: &[
                                            ("chamfer.svg", "倒角", "chamfer"),
                                            ("model/chamfer.svg", "倒角边", "chamfer_edge"),
                                        ],
                                    },
                                    SmallSplit { icon: "modify_break.svg", label: "打断", cmd: "break", menu: &[] },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "erase.svg", label: "删除", cmd: "erase" },
                                    Tool { icon: "explode.svg", label: "分解", cmd: "explode" },
                                    Tool { icon: "modify_join.svg", label: "合并", cmd: "join" },
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "注释",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "mtext.svg", label: "多行文字", cmd: "mtext" },
                            RibbonItem::Large { icon: "text.svg", label: "单行文字", cmd: "text" },
                            RibbonItem::SplitLarge {
                                dd: "dim",
                                icon: "dim_linear.svg",
                                label: "标注",
                                cmd: "dim_linear",
                                menu: &[
                                    ("dim_linear.svg", "线性", "dim_linear"),
                                    ("dim_aligned.svg", "对齐", "dim_aligned"),
                                    ("dim_angular.svg", "角度", "dim_angular"),
                                    ("dim_radius.svg", "半径", "dim_radius"),
                                    ("dim_diameter.svg", "直径", "dim_diameter"),
                                ],
                            },
                            RibbonItem::Large { icon: "leader.svg", label: "引线", cmd: "leader" },
                            RibbonItem::LabeledSplit {
                                icon: "dim_style.svg",
                                label: "ISO-25",
                                cmd: "dimstyle",
                                menu: &[
                                    ("dim_style.svg", "ISO-25", "dimstyle_iso25"),
                                    ("dim_style.svg", "GB-35", "dimstyle_gb35"),
                                ],
                            },
                            RibbonItem::ActionRow {
                                items: &[
                                    ActionBtn { label: "快速选择", cmd: "qselect", disabled: false },
                                    ActionBtn { label: "查找文字", cmd: "find", disabled: false },
                                    ActionBtn { label: "替换", cmd: "replace", disabled: true },
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "图层",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "layers/panel.svg", label: "图层特性", cmd: "layer" },
                            RibbonItem::LayerCombo,
                            RibbonItem::LabeledSplit {
                                icon: "status/lwt.svg",
                                label: "随层",
                                cmd: "lweight",
                                menu: &[
                                    ("status/lwt.svg", "随层", "lweight_bylayer"),
                                    ("status/lwt.svg", "0.30 毫米", "lweight_030"),
                                    ("status/lwt.svg", "0.50 毫米", "lweight_050"),
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "块",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "blocks/block.svg", label: "创建块", cmd: "block" },
                            RibbonItem::Large { icon: "blocks/insert.svg", label: "插入块", cmd: "insert" },
                        ],
                    },
                ],
            },
            RibbonTab {
                name: "参数化",
                groups: vec![
                    RibbonGroup {
                        name: "几何约束",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "constrain/auto.svg", label: "自动约束", cmd: "auto_constrain" },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "constrain/coincident.svg", label: "重合", cmd: "gccoincident" },
                                    Tool { icon: "constrain/parallel.svg", label: "平行", cmd: "gcparallel" },
                                    Tool { icon: "constrain/tangent.svg", label: "相切", cmd: "gctangent" },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "constrain/perpendicular.svg", label: "垂直", cmd: "gcperpendicular" },
                                    Tool { icon: "constrain/colinear.svg", label: "共线", cmd: "gccollinear" },
                                    Tool { icon: "constrain/concentric.svg", label: "同心", cmd: "gcconcentric" },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "constrain/horizontal.svg", label: "水平", cmd: "gchorizontal" },
                                    Tool { icon: "constrain/vertical.svg", label: "竖直", cmd: "gcvertical" },
                                    Tool { icon: "constrain/symmetric.svg", label: "对称", cmd: "gcsymmetric" },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "constrain/fixed.svg", label: "固定", cmd: "gcfixed" },
                                    Tool { icon: "constrain/equal.svg", label: "相等", cmd: "gcequal" },
                                    Tool { icon: "constrain/smooth.svg", label: "平滑", cmd: "gcsmooth" },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "constrain/show.svg", label: "显示约束", cmd: "constraintbar" },
                                    Tool { icon: "constrain/show_all.svg", label: "全部显示", cmd: "constraint_show_all" },
                                    Tool { icon: "constrain/hide_all.svg", label: "全部隐藏", cmd: "constraint_hide_all" },
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "标注约束",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "constrain/distance.svg", label: "线性", cmd: "dimconstraint_linear" },
                            RibbonItem::Large { icon: "constrain/distance_x.svg", label: "水平", cmd: "dimconstraint_x" },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "constrain/angle.svg", label: "角度", cmd: "dimconstraint_ang" },
                                    Tool { icon: "dim_radius.svg", label: "半径", cmd: "dimconstraint_rad" },
                                    Tool { icon: "constrain/convert.svg", label: "转换", cmd: "dimconstraint_convert" },
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "管理",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "constrain/delete.svg", label: "删除约束", cmd: "delconstraint" },
                            RibbonItem::Large { icon: "constrain/parameters.svg", label: "参数管理器", cmd: "parameters" },
                        ],
                    },
                ],
            },
            RibbonTab {
                name: "模型",
                groups: vec![
                    RibbonGroup {
                        name: "创建",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::SplitLarge {
                                dd: "solid3d",
                                icon: "model/box.svg",
                                label: "实体",
                                cmd: "box",
                                menu: &[
                                    ("model/box.svg", "长方体", "box"),
                                    ("model/cylinder.svg", "圆柱体", "cylinder"),
                                    ("model/cone.svg", "圆锥体", "cone"),
                                    ("model/sphere.svg", "球体", "sphere"),
                                    ("model/wedge.svg", "楔体", "wedge"),
                                    ("model/torus.svg", "圆环体", "torus"),
                                ],
                            },
                            RibbonItem::Large { icon: "extrude.svg", label: "拉伸", cmd: "extrude" },
                            RibbonItem::Large { icon: "revolve.svg", label: "旋转", cmd: "revolve" },
                            RibbonItem::Large { icon: "loft.svg", label: "放样", cmd: "loft" },
                            RibbonItem::Large { icon: "sweep.svg", label: "扫掠", cmd: "sweep" },
                            RibbonItem::Large { icon: "presspull.svg", label: "按住并拖动", cmd: "presspull" },
                        ],
                    },
                    RibbonGroup {
                        name: "布尔运算",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "union.svg", label: "并集", cmd: "union" },
                            RibbonItem::Large { icon: "subtract.svg", label: "差集", cmd: "subtract" },
                            RibbonItem::Large { icon: "intersect.svg", label: "交集", cmd: "intersect" },
                        ],
                    },
                    RibbonGroup {
                        name: "边",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "model/fillet.svg", label: "圆角边", cmd: "fillet_edge" },
                            RibbonItem::Large { icon: "model/chamfer.svg", label: "倒角边", cmd: "chamfer_edge" },
                            RibbonItem::Large { icon: "solid.svg", label: "抽壳", cmd: "shell" },
                        ],
                    },
                ],
            },
            RibbonTab {
                name: "视图",
                groups: vec![
                    RibbonGroup {
                        name: "导航",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "zoom_ext.svg", label: "范围缩放", cmd: "zoom_extents" },
                            RibbonItem::Column {
                                tools: &[
                                    Tool { icon: "zoom_in.svg", label: "放大", cmd: "zoom_in" },
                                    Tool { icon: "zoom_out.svg", label: "缩小", cmd: "zoom_out" },
                                    Tool { icon: "zoom_window.svg", label: "窗口缩放", cmd: "zoom_window" },
                                ],
                            },
                            RibbonItem::Large { icon: "pan.svg", label: "平移", cmd: "pan" },
                            RibbonItem::Large { icon: "orbit.svg", label: "受约束的\n动态观察", cmd: "orbit" },
                        ],
                    },
                    RibbonGroup {
                        name: "预设视图",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "view_top.svg", label: "俯视", cmd: "view_top" },
                            RibbonItem::Large { icon: "view_front.svg", label: "前视", cmd: "view_front" },
                            RibbonItem::Large { icon: "view_right.svg", label: "右视", cmd: "view_right" },
                            RibbonItem::Large { icon: "view_iso.svg", label: "等轴测", cmd: "view_iso" },
                        ],
                    },
                    RibbonGroup {
                        name: "视觉样式",
                        ext: None,
                        launcher: false,
                        items: vec![RibbonItem::SplitLarge {
                            dd: "visualstyle",
                            icon: "wireframe.svg",
                            label: "样式",
                            cmd: "vscurrent_wireframe",
                            menu: &[
                                ("wireframe.svg", "二维线框", "vscurrent_wireframe"),
                                ("hidden.svg", "隐藏", "vscurrent_hidden"),
                                ("solid.svg", "真实", "vscurrent_realistic"),
                                ("xray.svg", "线框", "vscurrent_xray"),
                            ],
                        }],
                    },
                    RibbonGroup {
                        name: "界面",
                        ext: None,
                        launcher: false,
                        items: vec![RibbonItem::Column {
                            tools: &[
                                Tool { icon: "status/cleanscreen.svg", label: "全屏显示", cmd: "cleanscreen" },
                                Tool { icon: "viewcube.svg", label: "ViewCube", cmd: "navvcube" },
                                Tool { icon: "ucs_icon.svg", label: "UCS 图标", cmd: "ucsicon" },
                            ],
                        }],
                    },
                ],
            },
            RibbonTab {
                name: "管理",
                groups: vec![
                    RibbonGroup {
                        name: "清理",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "purge.svg", label: "清理", cmd: "purge" },
                            RibbonItem::Large { icon: "overkill.svg", label: "删除重复\n对象", cmd: "overkill" },
                            RibbonItem::Large { icon: "audit.svg", label: "核查", cmd: "audit" },
                        ],
                    },
                    RibbonGroup {
                        name: "应用程序",
                        ext: None,
                        launcher: false,
                        items: vec![
                            RibbonItem::Large { icon: "ui/gear.svg", label: "选项", cmd: "options" },
                            RibbonItem::Large { icon: "plot.svg", label: "打印", cmd: "plot" },
                            RibbonItem::Large { icon: "properties.svg", label: "特性", cmd: "properties" },
                        ],
                    },
                ],
            },
        ]
    })
}

// ---------------------------------------------------------------------------
// 自适应降级（对应 OpenCADStudio collapse.rs 的 decide_levels）
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    /// 完整面板：大按钮 + 带标签小按钮行。
    Full,
    /// 中间档：所有控件变 22 高横排按钮（icon 16 + 文字），流式换行。
    Middle,
    /// 紧凑面板：所有项收成 24px 图标竖列。
    Compact,
    /// 折叠为标题按钮，点击弹出整组内容的飞出面板。
    Flyout,
}

pub const LARGE_W: f32 = 60.0;
pub const LARGE_H: f32 = 68.0;
pub const MINI_W: f32 = 24.0;
pub const GROUP_PAD: f32 = 26.0;
pub const FLYOUT_W: f32 = 58.0;

/// 带文字标签的小按钮（图标上/文字下）：宽、高、按钮间距。
pub const STACK_W: f32 = 46.0;
pub const STACK_H: f32 = 40.0;
pub const STACK_GAP: f32 = 2.0;

/// 小工具行（Column/SplitColumn）整行的宽度。
pub fn stack_row_width(n: usize) -> f32 {
    if n == 0 {
        0.0
    } else {
        n as f32 * STACK_W + (n - 1) as f32 * STACK_GAP
    }
}

// ---------------------------------------------------------------------------
// 单键快捷键表（demo 用合理子集；tooltip 第三行与此联动）
// ---------------------------------------------------------------------------

/// (键, 命令)。gpui 版经 ToolAction 绑定到窗口；两前端 tooltip 共用。
pub const TOOL_KEYS: &[(&str, &str)] = &[
    ("L", "line"),
    ("P", "pline"),
    ("C", "circle"),
    ("A", "arc"),
    ("R", "rectang"),
    ("E", "erase"),
    ("M", "move"),
    ("O", "offset"),
    ("F", "fillet"),
    ("T", "mtext"),
    ("B", "block"),
    ("Z", "zoom_extents"),
];

/// 命令的单键快捷键（无则 None）。
pub fn shortcut_of(cmd: &str) -> Option<&'static str> {
    TOOL_KEYS
        .iter()
        .find(|(_, c)| *c == cmd)
        .map(|(k, _)| *k)
}

pub fn item_width_full(item: &RibbonItem) -> f32 {
    match item {
        RibbonItem::Large { .. } | RibbonItem::SplitLarge { .. } => LARGE_W,
        RibbonItem::Column { tools } => stack_row_width(tools.len()),
        RibbonItem::SplitColumn { tools } => stack_row_width(tools.len()),
        RibbonItem::LayerCombo => 110.0,
        RibbonItem::LabeledSplit { .. } => 104.0,
        // Office 形态
        RibbonItem::PasteGroup { .. } => 96.0,
        RibbonItem::ComboRow { .. } => 170.0,
        RibbonItem::CharFlow { .. } => 92.0,
        RibbonItem::SpinRow { .. } => 150.0,
        RibbonItem::ActionRow { .. } => 130.0,
    }
}

pub fn item_count(item: &RibbonItem) -> usize {
    match item {
        RibbonItem::Column { tools } => tools.len(),
        RibbonItem::SplitColumn { tools } => tools.len(),
        _ => 1,
    }
}

/// Middle 档单按钮宽估算：icon 16 + 间隙 + 文字（CJK 10.5/字符，ASCII 6）。
fn middle_btn_width(label: &str) -> f32 {
    let text_w: f32 = label
        .chars()
        .map(|c| if c.is_ascii() { 6.0 } else { 10.5 })
        .sum();
    (16.0 + 4.0 + text_w + 10.0).max(44.0)
}

/// Middle 档整行宽：每个工具变独立 22 高按钮，横排。
fn item_width_middle(item: &RibbonItem) -> f32 {
    const GAP: f32 = 2.0;
    match item {
        RibbonItem::Large { label, .. } | RibbonItem::SplitLarge { label, .. } => {
            middle_btn_width(&label.replace('\n', ""))
        }
        RibbonItem::Column { tools } => {
            tools.iter().map(|t| middle_btn_width(t.label)).sum::<f32>()
                + (tools.len() as f32 - 1.0) * GAP
        }
        RibbonItem::SplitColumn { tools } => {
            tools.iter().map(|t| middle_btn_width(t.label)).sum::<f32>()
                + (tools.len() as f32 - 1.0) * GAP
        }
        RibbonItem::LayerCombo => 110.0,
        RibbonItem::LabeledSplit { .. } => 104.0,
        RibbonItem::PasteGroup { .. } => 96.0,
        RibbonItem::ComboRow { .. } => 170.0,
        RibbonItem::CharFlow { .. } => 92.0,
        RibbonItem::SpinRow { .. } => 150.0,
        RibbonItem::ActionRow { .. } => 130.0,
    }
}

pub struct GroupWidths {
    pub full: f32,
    pub middle: f32,
    pub compact: f32,
    pub flyout: f32,
}

pub fn group_widths(group: &RibbonGroup) -> GroupWidths {
    let n: usize = group.items.iter().map(item_count).sum();
    GroupWidths {
        full: GROUP_PAD + group.items.iter().map(item_width_full).sum::<f32>(),
        middle: GROUP_PAD + group.items.iter().map(item_width_middle).sum::<f32>(),
        compact: GROUP_PAD + n as f32 * MINI_W,
        flyout: GROUP_PAD + FLYOUT_W,
    }
}

fn width_of(level: Level, w: &GroupWidths) -> f32 {
    match level {
        Level::Full => w.full,
        Level::Middle => w.middle,
        Level::Compact => w.compact,
        Level::Flyout => w.flyout,
    }
}

pub fn decide_levels(widths: &[GroupWidths], max_w: f32) -> Vec<Level> {
    let total = |levels: &[Level]| -> f32 {
        levels
            .iter()
            .zip(widths)
            .map(|(lv, w)| width_of(*lv, w))
            .sum()
    };
    let mut levels = vec![Level::Full; widths.len()];
    for degraded in [Level::Middle, Level::Compact, Level::Flyout] {
        for i in (0..widths.len()).rev() {
            if total(&levels) <= max_w {
                break;
            }
            // 下一档不更窄的组跳过（如堆叠标签小按钮在 Middle 横排反而变宽）
            if width_of(degraded, &widths[i]) >= width_of(levels[i], &widths[i]) {
                continue;
            }
            levels[i] = degraded;
        }
    }
    levels
}


/// Office Fluent 外观尺寸规格（px @96dpi，见 docs/office-ribbon-style-notes.md §9）。
pub mod metrics {
    /// 标签行高。
    pub const TAB_STRIP_H: f32 = 30.0;
    /// 工具内容区高（组名除外）。
    pub const TOOL_AREA_H: f32 = 66.0;
    /// 组名行高。
    pub const GROUP_LABEL_H: f32 = 16.0;
    /// 大按钮宽。
    pub const LARGE_W: f32 = 60.0;
    /// 大按钮高。
    pub const LARGE_H: f32 = 68.0;
    /// 大按钮图标。
    pub const LARGE_ICON: f32 = 32.0;
    /// 横排小按钮高。
    pub const SMALL_H: f32 = 22.0;
    /// 小图标尺寸。
    pub const SMALL_ICON: f32 = 16.0;
    /// 图标列宽。
    pub const MINI_W: f32 = 24.0;
    /// 组合框高。
    pub const COMBO_H: f32 = 20.0;
    /// 组水平内边距。
    pub const GROUP_PAD_X: f32 = 6.0;
    /// 分隔线上下缩进。
    pub const SEP_INSET: f32 = 4.0;
}
