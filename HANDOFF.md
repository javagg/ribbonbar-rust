# HANDOFF — OpenCAD Demo（gpui + egui 双前端）交接文档

> 更新：2026-09-29。新会话从这里恢复全部上下文。
> **下一步工作**见 §6；**踩坑清单**见 §7（必读，能省大量调试时间）。

---

## 1. 项目是什么

CAD 桌面应用 UI 框架 demo：RibbonBar（对照 OpenCADStudio 与 Office Fluent 风格，
已两轮迭代）+ 停靠面板 + 命令行 + 状态栏。**两个前端共享一套核心**：

- **gpui 版** `cargo run --bin cad_demo`（主力，功能最全，Office Fluent 风格）
- **egui 版** `cargo run --bin cad_egui`（eframe 0.36 + egui_tiles 0.17，同套核心）

仓库 `D:\srcs\gpapp`，分支 master，最新提交见 `git log`。工具链 **stable 1.98.1**
（不要装 rust-toolchain.toml 固定 nightly——stable 已满足，用户明确要求）。

## 2. 命令

```bash
cargo run --bin cad_demo    # gpui 版
cargo run --bin cad_egui    # egui 版
cargo build                 # 两者都建；当前零警告
```

依赖：gpui-kit 0.7.0（捆绑 gpui-pre 0.3.7 + gpui-component 0.7.0）、
eframe/egui 0.36.2、egui_tiles 0.17.1、resvg 0.48、tiny-skia。
Windows 需要 MSVC + CMake（已具备）。

## 3. 代码结构（哪些文件干什么）

```
src/lib.rs            共享核心入口（UI 无关，两前端共用）
  icon_bytes.rs       SVG 编译期嵌入(build.rs 生成清单) + 语义色重映射 + 尺寸统一到64
  model.rs            CadModel 应用状态（last_cmd/active_tool/undo-redo/图层/开关/命令流）
  ribbon_data.rs      Ribbon 数据（5 标签页全部工具数据）+ decide_levels 自适应降级
                      + metrics 模块（Office 尺寸规格常量）
src/main.rs           gpui 版入口（主题初始化 + Office 主题初始化）
  state.rs            CadModel 的 gpui Entity/Global 包装 + 命令入口（run_tool 等）
  theme.rs            OfficeTheme（蓝/绿预设）——配色不是重点，行为/尺寸才是
  office_widgets.rs   Office 主题全局 + 三态按钮原语 + 新形态渲染
                      （PasteGroup/ComboRow/CharFlow/SpinRow/ActionRow）
  icons.rs            gpui Image 缓存 + CadAssetSource（cad/ 前缀 → 内置图标）
  ribbon.rs           RibbonBar 渲染（Office 风格标签行/工具区/自适应/扩展面板）
  panels.rs           停靠面板（视口画布/命令行/特性/图层/工具选项板）
  workspace.rs        装配 + Office 状态栏（坐标/开关/视图/缩放滑块/主题切换）
src/bin/cad_egui.rs   egui 版全部（Ribbon 自绘三态原语 + egui_tiles 停靠）
assets/icons/         OpenCADStudio 的 353 个 SVG（GPL-3.0！demo 用，用户会整体替换）
build.rs              扫描 assets/icons 生成 OUT_DIR/cad_icons.rs 清单
docs/                 截图归档 + office-ribbon-style-notes.md（风格研究+尺寸/行为规格）
```

## 4. 已实现的功能（全部实测过）

- 自适应降级：`decide_levels` 从右往左 Full→Compact(26px 图标列)→Flyout(标题按钮+面板)
- 分裂按钮三种形态（大/小列内/横排），**last_cmd 面记忆**（面=最后用过的变体）、
  菜单勾选联动、菜单锚定按钮下方左对齐、贴住工具整块高亮
- 组标题扩展面板（绘图/修改组：标题▾ → 工具网格 → 子选项二级导航）
- Undo/Redo 历史下拉（一次多步）
- 图层组合框（搜索过滤 + 行内可见/冻结/锁定开关 + 活动层勾选）
- 富 tooltip（名称\n命令: X）
- Office Fluent 改造：File 块、激活标签白底点亮、上下文标签（粉、高位、
  随贴住工具出现/消失）、Options▾、组名+对话框启动器、
  剪贴板式组/组合框行/字符按钮行/数值微调组/横排文字按钮组（含禁用态）、
  状态栏（命令状态+坐标+捕捉开关+视图切换+缩放滑块+蓝绿主题切换）
- 双击标签折叠/展开功能区；尺寸对齐 Office 规格（`ribbon_data::metrics`）
- **文字标签 + 单键快捷键体系（§6.1 已完成）**：所有小工具带"图标上/文字下"
  标签按钮（46×40 横排行，宽度由 decide_levels 自适应吸收）；`ribbon_data::TOOL_KEYS`
  12 键（L/C/A/R/E/M/O/F/P/T/B/Z）经 `keys::ToolAction`（带参 action, no_json）
  绑定；tooltip 三行（名称/命令/快捷键）
- **应用按钮菜单（§6.2 已完成）**：点"文件"展开 Backstage 全屏面板
  （左列主题色大按钮 新建/打开/保存/另存为/打印+底部"选项"，右列最近文件），
  展开"文件"高亮、工具区隐藏；返回/Esc/再点"文件"关闭；状态在
  `CadModel::app_menu_open`（经 AppEvent 驱动 Workspace 主体切换）；
  右上"选项 ▾"已移除（折叠只剩双击标签）

## 5. 设计依据文档

- `docs/office-ribbon-style-notes.md`：Qitan Ribbon Office 风格研究 +
  **§9 外观尺寸规格表 + 行为规格表**（改造依据，改尺寸前必读）
- 用户准则：**关注行为与外观尺寸，主题配色不重要**；
  图标规范=单线扁平/16·20·32 多尺寸/修饰元素右下角；
  所有命令应有文字标签（非肌肉记忆图标）；
  Alt KeyTips、富 tooltip（快捷键+描述）、应用按钮菜单是规范要求

## 6. 下一步工作（按优先级，用户已认可的方向）

**先保证 gpui 版正常**，egui 版同步放后面。

1. **图标按尺寸栅格化**：规范 16/20/32 多尺寸；当前 64px 纹理缩放到 16px 发虚。
   `icon_bytes::prepared_at(rel, size)`；小按钮用 16px 专用笔画
2. **KeyTips**：Alt 唤出字母徽章（文件=F、绘图=D、直线=L…），键盘逐级导航
3. QAT 定制菜单（增删命令+最小化开关）、可编辑组合框（Input+Popup）、
   对话框启动器弹真对话框（gpui-component dialog）
4. egui 版同步以上能力（metrics/数据已在 lib；Office 形态项渲染 + 快捷键，
   render_item 里 Office 形态分支目前是空臂占位）

## 7. 踩坑清单（血泪教训，改代码前必读）

**gpui / gpui-kit 0.7.0**
- `AppModel` 全局必须持**强 Entity**（WeakEntity 会被回收→命令行/状态栏全空白）
- `Context::subscribe` 闭包 **4 参数**（无 window）；带 window 用 `subscribe_in`
- `ClickEvent::click_count()` 做双击；`Pixels` 私有字段，用 `.as_f32()`/`.to_f64()`
- Rust 2024 `impl Trait` 捕获所有生命周期 → 闭包里造的元素**返回 AnyElement**
  （`.into_any_element()`），别返回 `impl IntoElement`
- `PopupMenu::new` 是 pub(crate) → 在 `DropdownMenu` 闭包里用传入的 popup 链式构建
- 菜单图标：`Icon::empty().path("cad/x.svg")`，靠 `CadAssetSource`（svg() 单色渲染）
- `Icon::empty().path()` 能渲染的前提是 AssetSource 已安装
  （`application().with_assets`）
- Slider：`gpui_base::slider::{SliderState, SliderValue::Single(f32)}`，
  元素 `component::slider::Slider::new(&state)`
- gpui-component 的 `StyledExt`（h_flex 等）在 `gpui_kit::component::StyledExt`
- 主题切换要 `cx.refresh_windows()`（`cx.notify()` 需要 EntityId，App 级没有）
- **带参 action**：`#[derive(Clone, PartialEq, gpui_kit::Action)]` +
  `#[action(namespace = cad, no_json)]`，payload 走字段；`actions!` 只做 unit action。
  绑定 `KeyBinding::new("l", ToolAction("line"), None)` + App 级 `cx.bind_keys`，
  处理器挂窗口根 div `.on_action`
- **输入框焦点守卫（单键绑定防误触）**：gpui-component Input 点击后聚焦的是
  Input 元素内部 frame 句柄，**≠** `InputState::focus_handle(cx)`——句柄相等比较
  永远 false。正确做法：面板根 div `.track_focus(&panel.focus_handle)`，判断用
  `panel_focus.contains_focused(window, cx)`（沿焦点树找后代）
- **跨实体 UI 状态（如 Backstage 开关）必须放 CadModel 走 AppEvent::Updated**：
  放 RibbonBar 字段 + `cx.notify()` 只重绘 RibbonBar，Workspace 的主体切换
  （dock ↔ 全屏菜单）不会发生
- **Windows 键盘**：WM_CHAR 文本插入独立于 KeyDown 消费——KeyDown 被绑定吃掉后
  字符仍会进输入框，所以守卫只需跳过工具、不用手动回放字符；单键绑定注册为
  小写（`"l"` 而非 `"L"`）

**图标管线**
- `set_attr` 必须用**前导空格** needle（`" width=\""`），否则吞掉
  `stroke-width` 的空格 → SVG 根标签损坏 → 图标空白
- 替换区间含 needle 前导空格，**补回空格**（此 bug 曾导致部分图标全白）
- svg() 元素是单色（alpha mask + text_color）；多色图标用 `img()` 全彩渲染
- 白底（Office 工具区）必须 `icon_bytes::set_light_mode(true)` 做亮色映射

**egui 0.36**
- `eframe::App::ui(&mut self, ui, frame)`（不是 update）；`egui::Panel::top/bottom`
  （不是 TopBottomPanel）；`ScrollArea::auto_shrink([false, true])`——
  `false` 会把 ribbon 撑满窗口挤掉 dock
- Response::on_hover_text 消费所有权 → `if r.hovered() { r.clone().on_hover_text(...) }`
- CJK 字体：启动时加载 `C:\Windows\Fonts\msyh.ttc` push 进 families
- egui 反应式渲染：无输入不重绘，截图前先点一下/移动鼠标

**工具链**
- stable 1.98.1 编译通过；**不要**装 rust-toolchain.toml 固定 nightly
- cargo build 报 "Access is denied" = 应用还在运行锁住 exe，先 taskkill

## 8. 验证流程

运行 → computer-use 截图 → 点击目标 → 再截图。
注意 CUA 点击坐标有轻微缩放偏移（图像 1280 vs 窗口 1344），
小控件（<14px）点不中时先把命中区做大，或从放大截图精确读坐标。
egui 无输入不重绘，截图前先触发一次交互。

**截图工具**：仓库根 `tmp_shots/capture.ps1`（PrintWindow，不受窗口遮挡影响，
需 `SetProcessDPIAware`）+ `crop.ps1`（裁剪放大）；归档截图放 `docs/`。
**键盘注入不可靠**：CUA `pressKey("Escape")` 时好时坏、PowerShell
`SetForegroundWindow` 常被拒（返回 False 且按键发给别的窗口）——可靠手法：
先 CUA 点击目标窗口（建立前台），再 PowerShell `SendInput`（tmp_shots/sendesc2.ps1，
检查输出里 `foreground_is_cad=True`）。判定按键是否进入 gpui 用
`cx.intercept_keystrokes`（在一切 action 机制之前触发）。

## 9. 未实现的备忘（规范要求，见 §6 之外的远期项）

- KeyTips 细节（Alt 后逐级键盘导航）
- 对话框启动器开真对话框；可编辑组合框
- 富 tooltip 的补充描述段；缩放滑块双击复位 100%
- 真实的对象选择驱动上下文标签（现为"贴住工具"近似）
