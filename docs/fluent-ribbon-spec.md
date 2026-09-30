# Fluent.Ribbon 规格研读 —— Rust 实现对齐依据

> 来源：D:\srcs\Fluent.Ribbon（C# WPF 库）源码研读，2026-09-30。
> 用途：作为 gpui 版 RibbonBar 向微软规范对齐的设计依据。
> 引用格式 `文件:行`，相对 `Fluent.Ribbon/` 目录。

## 1. 尺寸体系

| 项 | 值 | 出处 |
|---|---|---|
| 控件逻辑尺寸 | `RibbonControlSize { Large, Middle, Small }` | Enumerations\RibbonControlSize.cs |
| 组状态 | `Large / Middle / Small / Collapsed / QuickAccess` | Enumerations\RibbonGroupBoxState.cs |
| 图标尺寸 | Small 16×16、Medium 24×24、Large 32×32 | Controls\IconPresenter.cs:20-28 |
| 大按钮（Large 态） | **H=68**，图标 32，标签最多两行 | Themes\Controls\Button.xaml:240-290 |
| 中/小按钮 | H=22，图标 16；Small 态 ToggleButton 宽 22、纯图标 | 同上；ToggleButton.xaml:20-55 |
| 简化模式按钮 | MinHeight=30，图标 24（Simplified+Small→16） | Button.xaml（IsSimplified 触发器） |
| 小输入控件 | 统一高 22（ComboBox/Spinner/TextBox） | 各 xaml；InputControlProperties.cs:41 |
| 组内容区高 | `RibbonTabControl.DefaultContentHeight=100` | Controls\RibbonTabControl.cs:47 |
| 公共边距 | Margin 1、Padding 2,0,2,0、Border 1px | Themes\Common.xaml:12-13 |
| CheckBox 13×13；ColorGallery 色块 13×13 | | CheckBox.xaml:99；ColorGallery.cs:295 |

**我们现状 vs 规格**：大按钮 56×62→应 **×68**；图标 30→**32**；标签单行→**最多两行（TwoLineLabel）**；小按钮 23×23→**高 22**；内容区 66→FR 为 100（标签行另计）。

## 2. 大按钮与分裂按钮结构（关键规范）

- Button 模板：垂直 [图标(Margin 0,2,0,0)] + [文字 TwoLineLabel(Margin 2,0,0,0)]（Button.xaml:77-110）。
- `TwoLineLabel`：两个 AccessText 上下叠放、`TextWrapping=Wrap`——**文字自动换行成两行**；Large 时 HasTwoLines=True，Small/Middle 单行水平（TwoLineLabel.xaml）。
- **SplitButton 与 Button 外观一致**：共用 PART_ButtonBorder；结构 = 图标区（内嵌 ToggleButton）+ 底部 `downBorder` 内 TwoLineLabel；**箭头 glyph（10×6，线宽 2）加在文字行内**（HasGlyph=True），不是独立底条（SplitButton.xaml:14-61、Common.xaml:74-86）。
- 主按钮与箭头区是**两个点击区、分开 hover**（SplitButton.xaml:171-186）；SplitButton 主 KeyTip 无 Secondary 时自动加后缀 "A"（SplitButton.cs:496-520）。
- Medium/Small 时箭头区替换文字区，固定宽：Split 14px、DropDown 10px。

**对齐动作**：① 大按钮 68 高 + 32 图标 + 两行标签；② 分裂按钮箭头从"底部条"改为**标签行内右侧 glyph**（我们当前是 Word Paste 式底条；FR 是行内箭头）。

## 3. RibbonGroupBox：布局与降级

- 内容容器 `RibbonGroupBoxWrapPanel`：**先竖排成列，放不下横向加列**；同列等宽（SharedSizeGroup）。（RibbonGroupBoxWrapPanel.cs:54-60, 212-279, 409-417）
- 每控件 `SizeDefinition` 默认 `"Large Middle Small"`（RibbonProperties.cs:63-65）——组状态决定各控件取哪档（RibbonGroupBox.UpdateChildSizes:203-249）。
- 降级链默认 `Large→Middle→Small→Collapsed`；Simplified 模式默认 `Large,Middle,Collapsed`。（RibbonGroupBoxStateDefinition.cs；RibbonGroupBox.cs:162）
- 降级算法：父容器 MeasureOverride 累加各组期望宽，超宽则**从最后一组向前**逐组降一档，有富余反向扩（RibbonGroupsContainer.cs:106-157）。ReduceOrder 支持 `"(Name)"` 语法对特定组（如 InRibbonGallery）做缩放而非换态。
- **Middle 态**（我们没有）：组内控件全部变 Medium/Small（H=22）横排多行流式。
- Collapsed 态（=我们的 Flyout）：显示组图标+两行标题（带▾），点开 Popup 弹出原内容（RibbonGroupBox.xaml:277-341）。
- 组标题：内容区下方一行，OneLineHeader 居中 + `TextTrimming=CharacterEllipsis`（RibbonGroupBox.xaml:32-39）。
- **对话框启动器 16×16 在标题行右端，默认 IsLauncherVisible=False（隐藏！）**（RibbonGroupBox.cs:363）。
- 组间分隔符：右侧 1px Rectangle，Margin 0,4。
- `RibbonToolBar`：支持自定义多行布局定义（按 Size 选布局、行高+均匀空白分布、列间自动分隔线）——对应我们手搓的 ComboRow/SpinRow 形态。

## 4. 三态视觉

- 静态：背景/边框全 Transparent，BorderThickness=1，**方角（无圆角）**。
- MouseOver：背景 `AccentLight2`、边框 `Gray2`(#7F7F7F)；Pressed：`AccentLight3` / `Gray3`(#9D9D9D)；Checked：`Accent20` / `Highlight`。
- 禁用：Opacity 0.5 + 灰度效果（IconPresenter）。
- GalleryItem：hover `AccentLight3`、选中 `AccentLight2`。
- 唯一圆角 8px 在 TabControl 内容区。

## 5. 标签行与窗口一体化

- TabControl 顶部行 Grid 4 列：**[Menu(文件/Backstage 按钮)] [标签滚动区*] [空] [QAT(下方模式时)+DisplayOptions 按钮]**（RibbonTabControl.xaml:145-215）。
- **DisplayOptions 按钮 22×22**（右上角，`RibbonDisplayOptions` 图标），下拉菜单：
  Auto-hide ribbon / **Expand ribbon(勾选=非最小化)** / **Minimize ribbon(勾选=最小化)** / 分隔 / Ribbon layout: **Use classic ribbon / Use simplified ribbon**（CanMinimize、CanUseSimplified 控制显隐）。
- 最小化：
  - **双击标签**切换（RibbonTabItem.cs:574-592，CanMinimize 默认 true）；
  - **Ctrl+F1** 切换、**Ctrl+F2** 切 Simplified（Ribbon.cs:1870-1900）；
  - 最小化时点标签 → `IsDropDownOpen` 弹出整层内容（RibbonTabControl IsDropDownOpen/OnSelectionChanged），选中即展开层；点外部收回。
  - 上下文组标签点击时自动 `IsMinimized=false`（RibbonContextualTabGroup.cs:301）。
- **RibbonTitleBar**（RibbonWindow 一体化标题栏）模板三件：`PART_QuickAccessToolbarHolder`（**QAT 默认在标题栏**）+ 标题（居中、省略号）+ `PART_ItemsContainer`（**上下文标签组显示在标题栏中央**，组名在标签上方区域）。（RibbonTitleBar.xaml:22-39）
- QAT：`ShowQuickAccessToolBarAboveRibbon` 切换标题栏/功能区下方；`QuickAccessMenuItem`（IsCheckable）逐项增删；默认右键菜单含自定义/最小化/位置切换。Backstage 打开时隐藏上下文 Tab（HideContextTabsOnOpen）。
- **Simplified 单行功能区**（IsSimplified，默认 false、CanUseSimplified 开关）：Tab 变矮、大按钮变 MinHeight 30 + 图标 24 横排单行、组状态链只有 Large→Middle→Collapsed。

## 6. 上下文标签组

- 颜色通过 `RibbonContextualTabGroup` 的 `TabItemForeground / TabItemSelectedForeground / TabItemMouseOverForeground / TabItemSelectedMouseOverForeground` 四个 Brush 属性下发到组内标签（RibbonContextualTabGroup.cs:22-65）；组容器 `RibbonContextualGroupsContainer` 放在**标题栏**里。
- 我们现状：粉底色块放标签行内 → 应改为**标题栏中央 + 组名在标签上方**的 Office 排布。

## 7. Backstage（全屏文件菜单）

- 机制：**Adorner 覆盖整个窗口内容区**（不盖标题栏，Margin 绑 TitleBar 高度）；打开时隐藏上下文 Tab、临时折叠 HwndHost。
- 左列：MinWidth **125**，背景 #FFF7F7F7（浅色）/ #FF2C2C2C（深色）——**不是主题色**；右列内容区 #FFFAFAFA / #FF1A1A1A。
- 返回按钮：高 48（内含 34×34 椭圆 + 16×12 左箭头，背景 AccentBase），Command 切换 IsOpen。
- 左列项（BackstageTabItem）：**高 38**，横向 [图标 + 单行文字]（Margin 25,0,15,0，省略号）；**选中 = 左侧 4px 竖条**（AccentBase，Margin 4,8,0,8），hover 灰条；禁用 Opacity 0.5；ToolTip 禁用。
- 分隔项 SeparatorTabItem：14pt SemiBold 标题 + 1px 线（Margin 25,10,20,10）；普通 Button 也可作为左列项（同 38 高模板）。
- 动画：整块从左滑入 Margin -125→0，0.5s CubicEase Out；关闭 0.3s。
- 关闭 5 途径：Esc（CloseOnEsc 默认 true）、再点 File 按钮、返回按钮、内容区 `IsDefinitive` 按钮、点普通标签头。
- StartScreen 变体：同 Backstage，左列宽 342，只显示一次。

**我们现状 vs 规格**：左列 200 主题色→**125 浅灰**；无返回按钮→**48 高椭圆返回钮**；TabItem 大按钮式→**38 高图标+单行字+4px 选中竖条**；无动画→滑入；Esc 已有 ✓。

## 8. ApplicationMenu（2010 风格，备选）

- `ApplicationMenu : DropDownButton`：按钮 MinWidth 60、AccentBase 方块、图标 40×16；弹窗 = 左菜单列表(MinWidth 100) + 右窗格(**宽 300**，RightPaneWidth) + 底部 Footer(MinHeight 17)。
- 二级项 MinHeight 53：3 列布局（图标列 44、**32×32 大图标**、粗体标题+灰 Description）。

## 9. KeyTip

- 激活键：**LeftAlt / RightAlt / F10**；Shift+F10 忽略；Alt+小键盘忽略。
- 流程：按键 → **0.7s 延迟显示徽章**（松开键提前显示）→ 再按 Alt 或 Esc 到根或鼠标点击 → 终止并恢复焦点。
- **键串纯手动指定**（`KeyTip.Keys` 附接属性），无自动生成；仅 Backstage/ApplicationMenu 默认 "F"；大小写不敏感；多字符序列按序输入。
- 导航：精确匹配 → 触发该元素并**下钻到子层**（父子 Adorner 链）；前缀匹配 → 渐进过滤其余徽章；无匹配 → 蜂鸣（根层直接终止）。
- Esc：逐级退回（MenuItem 关子菜单、Backstage 关面板），根层终止。
- 徽章：**深底(Grey1)白字、无固定尺寸**（文字 Margin 4,-1,4,1）、禁用 0.5；位置自动计算（大按钮底部居中上移 8px、Tab 底边居中半悬出、MenuItem 偏移 h/3+2 等）。

## 10. ScreenTip（富提示）

- 固定**宽 205**；结构：粗体标题(Margin 7,8,7,10,Wrap) → [图标(MaxHeight 48) + 正文] → 2px 灰线 + **禁用段**（16×16 警告图标 + "This command is currently disabled." + DisableReason，宿主 IsEnabled=false 自动出现）→ 2px 灰线 + **F1 段**（16×16 帮助图标 + "Press F1 for help"，HelpTopic 非空才显示，F1 触发 HelpTopic 事件）。
- 快捷键显示：**无内置格式化**，惯例把 "(Ctrl+C)" 写进标题/正文。
- 延迟：InitialShowDelay **900ms**、ShowDuration 20s、ShowOnDisabled=true。
- 位置：优先贴 Ribbon 底边下方 1px / 顶边上方。

**我们现状**：三行自制（名称/命令/快捷键）→ 应升级为 205px 富提示四段结构；命令串可作为粗体标题下正文段，快捷键并进正文。

## 11. 其他行为细节

- 本地化默认：File 按钮文本 "File"、KeyTip "F"。
- `IsDefaultContextMenuEnabled`：Ribbon 空白处右键默认菜单（显示 QAT/最小化选项）。
- 鼠标滚轮切标签（IsMouseWheelScrollingEnabled 默认 true）。
- 状态持久化接口 IRibbonStateStorage（Unloaded 时 Save）。
- 组自动降级仅按测量宽度，**无滚动条**（组容器溢出才出 RibbonScrollViewer）。

## 12. Rust 实现对齐清单（按优先级）

**P1 核心规格**
1. 大按钮 62→**68 高**、图标 30→**32**、标签支持**两行换行**（数据结构 label 拆两行或自动 wrap）。
2. 分裂按钮箭头改为**标签行内右侧 glyph**（替换现有底条实现）；face 与普通按钮同构保持。
3. 小按钮 **H=22**（我们 23）、Small 态纯图标宽 22。
4. 组降级链插入 **Middle 档**（控件全 22 高横排多行）：Large→Middle→Small(=Compact 图标列)→Collapsed(=Flyout)。
5. ScreenTip 富提示：205px 宽、粗体标题+正文(含"命令: X (快捷键: K)")+禁用原因段+延迟 900ms。
6. Backstage 对齐：左列 125 浅灰(#F7F7F7/暗色 #2C2C2C)、返回钮 48 高椭圆、TabItem 38 高(图标+单行字)、选中左侧 4px 竖条、滑入动画 0.5s、5 种关闭途径。
7. 对话框启动器**默认隐藏**（对齐 IsLauncherVisible=false）。

**P2 行为对齐**
8. QAT 移入标题栏（需 gpui 自绘标题栏与窗口一体化），可定制菜单（逐项勾选 + 最小化开关 + 上/下方位置）；右上 **DisplayOptions 22×22** 下拉（Auto-hide/展开/最小化/布局切换）。
9. 上下文标签组移到**标题栏中央**（组名在标签上方），Backstage 打开时隐藏。
10. KeyTip 体系：Alt/F10 唤出（0.7s 延迟）、深底白字徽章、手动键串（文件=F）、逐级导航+渐进过滤、Esc 逐级退、鼠标点击终止。
11. Ctrl+F1 切最小化、Ctrl+F2 切 Simplified；最小化时点标签弹层（点外部收回）；滚轮切标签。

**P3 扩展**
12. Simplified 单行模式（按钮 MinHeight 30/图标 24、组链 Large→Middle→Collapsed、Tab 变矮）。
13. InRibbonGallery / Gallery 控件（色板、样式库）；RibbonToolBar 自定义多行布局定义化。
14. 三态配色迁移到 Accent 梯度体系（AccentLight2/3、Accent20、Gray2/3），方角化（去 3px 圆角）。
15. 禁用态 0.5 透明 + 灰度；组标题省略号截断。
