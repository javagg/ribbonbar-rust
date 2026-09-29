const fs = require('fs');
let s = fs.readFileSync('src/ribbon.rs', 'utf8');

// ── 整段替换 render_tab_strip 为 Office 风格 ──
const oldStripStart = s.indexOf('    fn render_tab_strip(');
const oldStripEnd = s.indexOf('    #[allow(clippy::too_many_arguments)]\n    fn render_group(');
if (oldStripStart < 0 || oldStripEnd < 0) { console.error('strip markers fail'); process.exit(1); }

const newStrip = `    fn render_tab_strip(
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

        // File：左端独立块（应用菜单入口）
        strip = strip.child(
            div()
                .id("ribbon-file")
                .h(px(TAB_STRIP_H - 4.0))
                .px_3()
                .flex()
                .items_center()
                .rounded_t_sm()
                .bg(ot.file_bg)
                .text_color(ot.tab_text)
                .text_size(px(12.5))
                .cursor_pointer()
                .hover(|s| s.bg(ot.title_bg))
                .child("文件")
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.active = 0;
                    cx.notify();
                })),
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
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.active = ix;
                    cx.notify();
                }))
        }));

        // 上下文标签：第二主题色、位置更高（demo 常显；真实应用随对象选择出现）
        strip = strip.child(
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
        );

        strip = strip.child(div().flex_1());

        // Options ▾ + 折叠
        strip = strip.child(
            div()
                .id("ribbon-options")
                .h_flex()
                .items_center()
                .gap_1()
                .px_2()
                .text_color(ot.tab_text)
                .text_size(px(11.5))
                .cursor_pointer()
                .hover(|s| s.bg(ot.title_bg))
                .child("?")
                .child("选项 ▾")
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.collapsed = !this.collapsed;
                    cx.notify();
                })),
        );
        strip
    }

`;
s = s.slice(0, oldStripStart) + newStrip + s.slice(oldStripEnd);

// ── render_group：标题行 + 启动器 Office 化（替换标题分支与组名）──
s = s.replace(
`        // 组标题：注册了扩展的组是"标题 ▾"按钮，点开工具网格（可再进子选项）
        let title: AnyElement = match group.ext {
            Some(ext) if level != Level::Flyout => {
                ext_title_button(&key, group, ext, snap, theme).into_any_element()
            }
            _ => div()
                .h(px(15.0))
                .flex().items_center().justify_center()
                .text_size(px(10.0))
                .when(level == Level::Flyout, |el| el.text_color(theme.accent))
                .text_color(theme.muted_foreground)
                .child(SharedString::from(group.name))
                .into_any_element(),
        };

        div()
            .flex().flex_col()
            .border_l_1()
            .border_color(theme.border)
            .px_2()
            .mx_1()
            .child(div().flex().flex_row().items_start().h_full().child(body))
            .child(title)
            .into_any_element()`,
`        // 组标题：组名（居中）+ 右下角对话框启动器；扩展组为"标题 ▾"
        let has_ext = group.ext.is_some();
        let title: AnyElement = match (group.ext, level) {
            (Some(ext), Level::Full) => {
                ext_title_button(&key, group, ext, snap, theme).into_any_element()
            }
            _ => div()
                .h(px(15.0))
                .flex().items_center().justify_center()
                .text_size(px(10.0))
                .when(level == Level::Flyout, |el| el.text_color(ot.accent))
                .text_color(ot.group_name)
                .child(SharedString::from(group.name))
                .into_any_element(),
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
                    .when(has_ext && level == Level::Full, |el| {
                        // 对话框启动器（ Office 风格组角标）
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
            .into_any_element()`);

fs.writeFileSync('src/ribbon.rs', s);
console.log('OK');
