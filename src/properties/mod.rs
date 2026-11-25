pub mod metadata;

use crate::{
    app::dock::DockWidget,
    widgets::{search::Search, toolbar::toolbar, value_editor::property},
};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Properties {
    search: Search,
}

impl DockWidget for Properties {
    fn title(&self) -> egui::WidgetText {
        "Properties".into()
    }

    fn ui(&mut self, ctx: &mut crate::app::dock::DockContext<'_>) -> () {
        let Some(&selected) = ctx.place.selection.first() else {
            return;
        };

        let state = ctx.place.emulator.engine.state.borrow();
        let reflection = &ctx.place.emulator.engine.reflection;
        let class_info = reflection.class(state.dom.class_of(selected));

        let mut categorized: BTreeMap<&str, Vec<&ree::reflection::PropertyInfo>> = BTreeMap::new();
        for (_, index) in &class_info.properties {
            let p = reflection.property(*index);

            let class_name = reflection.class(p.owner).name.to_string();
            let property_name = reflection.name(p.name);

            if metadata::is_hidden(&class_name, property_name)
                || reflection.property_alias(class_info.id, p.name).is_some()
            {
                continue;
            }

            let category = metadata::category_of(&class_name, property_name);
            categorized.entry(category).or_default().push(p);
        }

        toolbar(ctx.ui, |ui| {
            self.search.ui(ui, "Search properties");
        });

        let mut pending = None;

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ctx.ui, |ui| {
                for (category, properties) in &categorized {
                    let category_matches = self.search.matches(category);

                    let mut visible: Vec<_> = properties
                        .iter()
                        .filter(|p| {
                            category_matches || self.search.matches(reflection.name(p.name))
                        })
                        .collect();

                    if visible.is_empty() {
                        continue;
                    }

                    visible.sort_by_key(|k| reflection.name(k.name));
                    egui::CollapsingHeader::new(*category)
                        .default_open(true)
                        .show(ui, |ui| {
                            for (i, p) in visible.iter().enumerate() {
                                let Some(value) = state.dom.raw_get(selected, p.name) else {
                                    continue;
                                };
                                let id = ui.make_persistent_id(p.name);
                                let name = reflection.name(p.name);
                                if let Some(new) = property(ui, id, reflection, name, value) {
                                    pending = Some((p.name, new));
                                }
                            }
                        });
                }
            });

        drop(state);
        if let Some((name, value)) = pending {
            ctx.place.set(selected, name, value);
        }
    }
}
