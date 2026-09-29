const fs = require('fs');
let s = fs.readFileSync('src/ribbon.rs', 'utf8');
// render_group 签名加 ot
s = s.replace(`    fn render_group(
        &self,
        tab_ix: usize,
        group_ix: usize,
        group: &'static RibbonGroup,
        level: Level,
        snap: &RibbonSnapshot,
        filter: &Entity<InputState>,
        theme: &Theme,
    ) -> AnyElement {
        let key = format!("t{tab_ix}g{group_ix}");`,
`    #[allow(clippy::too_many_arguments)]
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
        let key = format!("t{tab_ix}g{group_ix}");`);
// 调用点传 ot
s = s.replace('groups.push(self.render_group(self.active, gi, group, level, &snap, &self.layer_filter, &theme));',
              'groups.push(self.render_group(self.active, gi, group, level, &snap, &self.layer_filter, &ot, &theme));');
fs.writeFileSync('src/ribbon.rs', s);
console.log('OK');
