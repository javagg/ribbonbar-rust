# HANDOFF — OpenCAD Demo（gpui + egui 双前端）交接文档

> 更新：2026-09-29。新会话从这里恢复全部上下文。
> **下一步工作**见 §6；**踩坑清单**见 §7（必读，能省大量调试时间）。

---

## 1. 项目是什么

CAD 桌面应用 UI 框架 demo：RibbonBar（对照 OpenCADStudio 与 Office Fluent 风格，
已两轮迭代）+ 停靠面板 + 命令行 + 状态栏。**两个前端共享一套核心**：

- **gpui 版** `cargo run --bin cad_demo`（主力，功能最全，Office Fluent 风格）
- **egui 版** `cargo run --bin cad_egui`（eframe 0.36 + egui_tiles 0.17，同套核心）

仓库 `D:\srcs\gpapp`，分支 master，最新提交 `1b8c5d2`。工具链 **stable 1.98.1**
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

## 5. 设计依据文档

- `docs/office-ribbon-style-notes.md`：Qitan Ribbon Office 风格研究 +
  **§9 外观尺寸规格表 + 行为规格表**（改造依据，改尺寸前必读）
- 用户准则：**关注行为与外观尺寸，主题配色不重要**；
  图标规范=单线扁平/16·20·32 多尺寸/修饰元素右下角；
  所有命令应有文字标签（非肌肉记忆图标）；
  Alt KeyTips、富 tooltip（快捷键+描述）、应用按钮菜单是规范要求

## 6. 下一步工作（按优先级，用户已认可的方向）

**先保证 gpui 版正常**，egui 版同步放后面。

1. **图标+文字标签补全 + 快捷键体系**（一次数据结构改动）
   - 现状违规：16px 纯图标按钮（约束列/绘图列等）无文字，违反
     "非肌肉记忆图标必须带文字"
   - 做法：`ribbon_data` 工具定义加 `label`（竖列小按钮用）与 `shortcut`
     （LINE→L、CIRCLE→C…）字段；渲染为 图标上/文字下 小按钮（高约 40）；
     tooltip 变三行（名称 / 命令: X / 快捷键: L）；
     用 gpui KeyBinding 把单键绑上（焦点不在输入框时）
2. **应用按钮菜单**：现"文件"只是标签块。改为点击展开全屏菜单面板
   （左列大按钮 新建/打开/保存/另存为/打印，右列最近文件，底部"选项"）；
   移除右上"选项 ▾"
3. **图标按尺寸栅格化**：规范 16/20/32 多尺寸；当前 64px 纹理缩放到 16px 发虚。
   `icon_bytes::prepared_at(rel, size)`；小按钮用 16px 专用笔画
4. **KeyTips**：Alt 唤出字母徽章（文件=F、绘图=D、直线=L…），键盘逐级导航
5. QAT 定制菜单（增删命令+最小化开关）、可编辑组合框（Input+Popup）、
   对话框启动器弹真对话框（gpui-component dialog）
6. egui 版同步以上能力（metrics/数据已在 lib，主要补渲染）

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

## 9. 未实现的备忘（规范要求，见 §6 之外的远期项）

- KeyTips 细节（Alt 后逐级键盘导航）
- 对话框启动器开真对话框；可编辑组合框
- 富 tooltip 的补充描述段；缩放滑块双击复位 100%
- 真实的对象选择驱动上下文标签（现为"贴住工具"近似）
