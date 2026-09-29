//! Ribbon 数据模型与自适应降级算法（纯数据 + 纯函数，UI 无关）。
//! gpui 版与 egui 版共用；结构与度量对照 OpenCADStudio（registry.rs / collapse.rs）。

// ---------------------------------------------------------------------------
// 数据模型
// ---------------------------------------------------------------------------

/// 分裂按钮的菜单项：(图标, 标签, 命令)
pub type MenuEntry = (&'static str, &'static str, &'static str);
/// 小工具列中的一项：(图标, 提示文本, 命令)
pub type ToolEntry = (&'static str, &'static str, &'static str);
/// 扩展面板里工具的子选项：(标签, 命令)
pub type ExtOption = (&'static str, &'static str);

/// 带菜单的小工具（下拉 id 即 cmd，命令名天然唯一）。
pub struct SmallSplit {
    pub icon: &'static str,
    pub tip: &'static str,
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
    /// 小工具竖排列。
    Column { tools: &'static [ToolEntry] },
    /// 小工具竖排列，个别项可带下拉箭头。
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
}

pub struct RibbonGroup {
    pub name: &'static str,
    pub items: Vec<RibbonItem>,
    /// 组标题扩展面板（None = 标题不可点）。
    pub ext: Option<&'static [ExtTool]>,
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
                        name: "绘图",
                        ext: Some(DRAW_EXT),
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
                                        tip: "点",
                                        cmd: "point",
                                        menu: &[
                                            ("point.svg", "单点", "point"),
                                            ("multipoint.svg", "多点", "multipoint"),
                                        ],
                                    },
                                    SmallSplit { icon: "ray.svg", tip: "射线", cmd: "ray", menu: &[] },
                                    SmallSplit { icon: "xline.svg", tip: "构造线", cmd: "xline", menu: &[] },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    ("spline.svg", "样条曲线", "spline"),
                                    ("donut.svg", "圆环", "donut"),
                                    ("helix.svg", "螺旋线", "helix"),
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "修改",
                        ext: Some(MODIFY_EXT),
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
                                    ("trim.svg", "修剪", "trim"),
                                    ("extend.svg", "延伸", "extend"),
                                    ("offset.svg", "偏移", "offset"),
                                ],
                            },
                            RibbonItem::SplitColumn {
                                tools: &[
                                    SmallSplit {
                                        icon: "fillet.svg",
                                        tip: "圆角",
                                        cmd: "fillet",
                                        menu: &[
                                            ("fillet.svg", "圆角", "fillet"),
                                            ("model/fillet.svg", "圆角边", "fillet_edge"),
                                        ],
                                    },
                                    SmallSplit {
                                        icon: "chamfer.svg",
                                        tip: "倒角",
                                        cmd: "chamfer",
                                        menu: &[
                                            ("chamfer.svg", "倒角", "chamfer"),
                                            ("model/chamfer.svg", "倒角边", "chamfer_edge"),
                                        ],
                                    },
                                    SmallSplit { icon: "modify_break.svg", tip: "打断", cmd: "break", menu: &[] },
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    ("erase.svg", "删除", "erase"),
                                    ("explode.svg", "分解", "explode"),
                                    ("modify_join.svg", "合并", "join"),
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "注释",
                        ext: None,
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
                        ],
                    },
                    RibbonGroup {
                        name: "图层",
                        ext: None,
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
                        items: vec![
                            RibbonItem::Large { icon: "constrain/auto.svg", label: "自动约束", cmd: "auto_constrain" },
                            RibbonItem::Column {
                                tools: &[
                                    ("constrain/coincident.svg", "重合", "gccoincident"),
                                    ("constrain/parallel.svg", "平行", "gcparallel"),
                                    ("constrain/tangent.svg", "相切", "gctangent"),
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    ("constrain/perpendicular.svg", "垂直", "gcperpendicular"),
                                    ("constrain/colinear.svg", "共线", "gccollinear"),
                                    ("constrain/concentric.svg", "同心", "gcconcentric"),
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    ("constrain/horizontal.svg", "水平", "gchorizontal"),
                                    ("constrain/vertical.svg", "竖直", "gcvertical"),
                                    ("constrain/symmetric.svg", "对称", "gcsymmetric"),
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    ("constrain/fixed.svg", "固定", "gcfixed"),
                                    ("constrain/equal.svg", "相等", "gcequal"),
                                    ("constrain/smooth.svg", "平滑", "gcsmooth"),
                                ],
                            },
                            RibbonItem::Column {
                                tools: &[
                                    ("constrain/show.svg", "显示约束", "constraintbar"),
                                    ("constrain/show_all.svg", "全部显示", "constraint_show_all"),
                                    ("constrain/hide_all.svg", "全部隐藏", "constraint_hide_all"),
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "标注约束",
                        ext: None,
                        items: vec![
                            RibbonItem::Large { icon: "constrain/distance.svg", label: "线性", cmd: "dimconstraint_linear" },
                            RibbonItem::Large { icon: "constrain/distance_x.svg", label: "水平", cmd: "dimconstraint_x" },
                            RibbonItem::Column {
                                tools: &[
                                    ("constrain/angle.svg", "角度", "dimconstraint_ang"),
                                    ("dim_radius.svg", "半径", "dimconstraint_rad"),
                                    ("constrain/convert.svg", "转换", "dimconstraint_convert"),
                                ],
                            },
                        ],
                    },
                    RibbonGroup {
                        name: "管理",
                        ext: None,
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
                        items: vec![
                            RibbonItem::Large { icon: "union.svg", label: "并集", cmd: "union" },
                            RibbonItem::Large { icon: "subtract.svg", label: "差集", cmd: "subtract" },
                            RibbonItem::Large { icon: "intersect.svg", label: "交集", cmd: "intersect" },
                        ],
                    },
                    RibbonGroup {
                        name: "边",
                        ext: None,
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
                        items: vec![
                            RibbonItem::Large { icon: "zoom_ext.svg", label: "范围缩放", cmd: "zoom_extents" },
                            RibbonItem::Column {
                                tools: &[
                                    ("zoom_in.svg", "放大", "zoom_in"),
                                    ("zoom_out.svg", "缩小", "zoom_out"),
                                    ("zoom_window.svg", "窗口缩放", "zoom_window"),
                                ],
                            },
                            RibbonItem::Large { icon: "pan.svg", label: "平移", cmd: "pan" },
                            RibbonItem::Large { icon: "orbit.svg", label: "受约束的动态观察", cmd: "orbit" },
                        ],
                    },
                    RibbonGroup {
                        name: "预设视图",
                        ext: None,
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
                        items: vec![RibbonItem::Column {
                            tools: &[
                                ("status/cleanscreen.svg", "全屏显示", "cleanscreen"),
                                ("viewcube.svg", "ViewCube", "navvcube"),
                                ("ucs_icon.svg", "UCS 图标", "ucsicon"),
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
                        items: vec![
                            RibbonItem::Large { icon: "purge.svg", label: "清理", cmd: "purge" },
                            RibbonItem::Large { icon: "overkill.svg", label: "删除重复对象", cmd: "overkill" },
                            RibbonItem::Large { icon: "audit.svg", label: "核查", cmd: "audit" },
                        ],
                    },
                    RibbonGroup {
                        name: "应用程序",
                        ext: None,
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
    /// 完整面板：大按钮 + 图标列。
    Full,
    /// 紧凑面板：所有项收成 26px 图标竖列。
    Compact,
    /// 折叠为标题按钮，点击弹出整组内容的飞出面板。
    Flyout,
}

pub const LARGE_W: f32 = 60.0;
pub const LARGE_H: f32 = 68.0;
pub const MINI_W: f32 = 26.0;
pub const GROUP_PAD: f32 = 26.0;
pub const FLYOUT_W: f32 = 58.0;

pub fn item_width_full(item: &RibbonItem) -> f32 {
    match item {
        RibbonItem::Large { .. } | RibbonItem::SplitLarge { .. } => LARGE_W,
        RibbonItem::Column { .. } | RibbonItem::SplitColumn { .. } => MINI_W,
        RibbonItem::LayerCombo => 110.0,
        RibbonItem::LabeledSplit { .. } => 104.0,
    }
}

pub fn item_count(item: &RibbonItem) -> usize {
    match item {
        RibbonItem::Column { tools } => tools.len(),
        RibbonItem::SplitColumn { tools } => tools.len(),
        _ => 1,
    }
}

pub struct GroupWidths {
    pub full: f32,
    pub compact: f32,
    pub flyout: f32,
}

pub fn group_widths(group: &RibbonGroup) -> GroupWidths {
    let n: usize = group.items.iter().map(item_count).sum();
    GroupWidths {
        full: GROUP_PAD + group.items.iter().map(item_width_full).sum::<f32>(),
        compact: GROUP_PAD + n as f32 * MINI_W,
        flyout: GROUP_PAD + FLYOUT_W,
    }
}

pub fn decide_levels(widths: &[GroupWidths], max_w: f32) -> Vec<Level> {
    let total = |levels: &[Level]| -> f32 {
        levels
            .iter()
            .zip(widths)
            .map(|(lv, w)| match lv {
                Level::Full => w.full,
                Level::Compact => w.compact,
                Level::Flyout => w.flyout,
            })
            .sum()
    };
    let mut levels = vec![Level::Full; widths.len()];
    for degraded in [Level::Compact, Level::Flyout] {
        for i in (0..widths.len()).rev() {
            if total(&levels) <= max_w {
                break;
            }
            levels[i] = degraded;
        }
    }
    levels
}

