const fs = require('fs');
let s = fs.readFileSync('src/ribbon.rs', 'utf8');
s = s.replace(`    fn render_item(
        &self,
        key: &str,
        item: &RibbonItem,
        snap: &RibbonSnapshot,
        filter: &Entity<InputState>,
        theme: &Theme,
    ) -> AnyElement {`, `    fn render_item(
        &self,
        key: &str,
        item: &RibbonItem,
        snap: &RibbonSnapshot,
        filter: &Entity<InputState>,
        ot: &OfficeTheme,
        theme: &Theme,
    ) -> AnyElement {`);
s = s.replace('items.push(self.render_item(&format!("{key}i{ii}"), item, snap, filter, theme));',
              'items.push(self.render_item(&format!("{key}i{ii}"), item, snap, filter, ot, theme));');
fs.writeFileSync('src/ribbon.rs', s);
console.log('OK');
