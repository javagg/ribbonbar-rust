const fs = require('fs');
let s = fs.readFileSync('src/ribbon.rs', 'utf8');

// render_item 补 Office 变体分支
s = s.replace(`            RibbonItem::LayerCombo => layer_combo(key, snap, filter, theme).into_any_element(),
        }
    }

    /// Compact 档：每一项收成一个 26px 图标竖列。`, `            RibbonItem::LayerCombo => layer_combo(key, snap, filter, theme).into_any_element(),
            RibbonItem::PasteGroup { icon, label, cmd, sides } => {
                crate::office_widgets::render_paste_group(key, icon, label, cmd, sides, ot, cx_of(theme))
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

    /// Compact 档：每一项收成一个 26px 图标竖列。`);

// render_compact_item 补分支
s = s.replace(`            RibbonItem::LayerCombo => {
                col = col.child(mini_tool(&format!("{key}m"), "layers/panel.svg", "layer", theme));
            }
        }
        col.into_any_element()`, `            RibbonItem::LayerCombo => {
                col = col.child(mini_tool(&format!("{key}m"), "layers/panel.svg", "layer", theme));
            }
            RibbonItem::PasteGroup { icon, cmd, .. } | RibbonItem::LabeledSplit { icon, cmd, .. }
                if false => {}
            _ => {
                // Office 形态在 Compact 下收缩为单一图标（取第一个可执行命令）
                let (icon, cmd) = match item {
                    RibbonItem::PasteGroup { icon, cmd, .. } => (icon, cmd),
                    RibbonItem::ComboRow { items } => ("ui/gear.svg", items.first().map(|c| c.cmd).unwrap_or("combo")),
                    RibbonItem::CharFlow { buttons } => (buttons.first().map(|b| "").unwrap_or(""), buttons.first().map(|b| b.cmd).unwrap_or("char")),
                    RibbonItem::SpinRow { items } => ("status/lwt.svg", items.first().map(|s| s.cmd).unwrap_or("spin")),
                    RibbonItem::ActionRow { items } => ("ui/gear.svg", items.first().map(|a| a.cmd).unwrap_or("action")),
                    _ => ("ui/gear.svg", "unknown"),
                };
                if !icon.is_empty() {
                    col = col.child(mini_tool(&format!("{key}m"), icon, cmd, theme));
                }
            }
        }
        col.into_any_element()`);

// render_item 需要 cx —— 检查签名：render_item 无 cx。cx_of(theme) 方案改为直接读全局
s = s.replace('crate::office_widgets::render_paste_group(key, icon, label, cmd, sides, ot, cx_of(theme))',
              'crate::office_widgets::render_paste_group(key, icon, label, cmd, sides, ot)');

fs.writeFileSync('src/ribbon.rs', s);
console.log('OK');
