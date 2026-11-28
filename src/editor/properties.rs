pub mod rows;

use self::rows::rows;
use crate::{
    editor::{
        commands::Command,
        dock::{DockContext, DockWidget},
    },
    place::Place,
    widgets::{layout::toolbar, search::Search, tree::TreeRow},
};
use std::collections::{BTreeMap, HashSet};

struct Field {
    label: String,
    name: ree::NameId,
    value: ree::Value,
    read_only: bool,
}

const NAME_SPLIT: f32 = 1.0 / 3.0;

fn fields(place: &Place, id: ree::InstanceId, search: &Search) -> BTreeMap<String, Vec<Field>> {
    let reflection = place.reflection();
    let dom = place.dom();
    let mut groups: BTreeMap<String, Vec<Field>> = BTreeMap::new();

    let class_id = place.class_id(id);
    let class_info = reflection.class(class_id);
    let class_name = &class_info.name;

    for (property_name, _data_type) in ree::record::encode::serialized_properties(class_name) {
        if search.is_active() && !search.matches(&property_name) {
            continue;
        }

        let Some(property_id) = reflection.name_id(&property_name) else {
            continue;
        };

        let Some(value) = dom.raw_get(id, property_id) else {
            continue;
        };

        if matches!(value, ree::Value::Dom(_)) {
            continue;
        }

        let info = reflection.member_by_name(class_name, &property_name);

        groups
            .entry(
                info.and_then(|info| info.category())
                    .unwrap_or("Other")
                    .to_string(),
            )
            .or_default()
            .push(Field {
                label: property_name.to_string(),
                name: property_id,
                value: value.clone(),
                read_only: info.map(|info| info.is_deprecated()).unwrap_or_default(),
            });
    }

    for fields in groups.values_mut() {
        fields.sort_by(|lhs, rhs| lhs.label.cmp(&rhs.label));
    }

    groups
}

#[derive(Default)]
pub struct Properties {
    search: Search,
    collapsed: HashSet<String>,
}

impl DockWidget for Properties {
    fn title(&self) -> egui::WidgetText {
        "Properties".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        ctx.ui.spacing_mut().item_spacing.y = 0.0;

        toolbar(ctx.ui, |ui| {
            self.search.ui(ui, "Search properties");
        });

        let &[id] = ctx.state.selection.as_slice() else {
            ctx.ui.centered_and_justified(|ui| {
                ui.weak(if ctx.state.selection.is_empty() {
                    "Nothing selected"
                } else {
                    "Multiple selected"
                })
            });

            return;
        };

        let mut groups = fields(&ctx.state.place, id, &self.search);
        let filtering = self.search.is_active();
        let name_width =
            (ctx.ui.available_width() * NAME_SPLIT).max(ctx.ui.spacing().interact_size.x);

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ctx.ui, |ui| {
                for (category, fields) in &mut groups {
                    let expanded = filtering || !self.collapsed.contains(category);

                    let header = TreeRow {
                        depth: 0,
                        has_children: true,
                        expanded,
                        selected: false,
                    }
                    .show(ui, |ui| {
                        ui.strong(category.as_str());
                    });

                    if (header.toggled || header.response.clicked())
                        && !self.collapsed.remove(category)
                    {
                        self.collapsed.insert(category.clone());
                    }

                    if !expanded {
                        continue;
                    }

                    for field in fields {
                        let changed = ui
                            .add_enabled_ui(!field.read_only, |ui| {
                                rows(ui, name_width, &field.label, &mut field.value)
                            })
                            .inner;

                        if changed {
                            ctx.state.commands.push(Command::SetProperty {
                                id,
                                name: field.name,
                                value: field.value.clone(),
                            });
                        }
                    }
                }
            });
    }
}
