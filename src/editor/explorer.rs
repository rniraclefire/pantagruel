mod rows;

use crate::{
    editor::dock::{DockContext, DockWidget},
    widgets::{layout::toolbar, search::Search, tree::TreeRow},
};
use rows::Row;
use std::collections::HashSet;

const VISIBLE_SERVICES: &[&str] = &[
    "Workspace",
    "Players",
    "Lighting",
    "MaterialService",
    "ReplicatedFirst",
    "ReplicatedStorage",
    "ServerScriptService",
    "ServerStorage",
    "StarterGui",
    "StarterPack",
    "StarterPlayer",
    "Teams",
    "SoundService",
    "TextChatService",
];

fn services<'a>(
    dom: &'a ree::Dom,
    reflection: &'a ree::reflection::Reflection,
    game: ree::InstanceId,
) -> impl Iterator<Item = ree::InstanceId> + 'a {
    VISIBLE_SERVICES
        .iter()
        .filter_map(|name| reflection.class_id(name))
        .filter_map(move |class| dom.find_child_of_class(game, class))
}

#[derive(Default)]
pub struct Explorer {
    search: Search,
    expanded: HashSet<ree::InstanceId>,
}

impl Explorer {
    fn list(
        &mut self,
        ui: &mut egui::Ui,
        dom: &ree::Dom,
        rows: &[Row],
        selection: &mut Vec<ree::InstanceId>,
    ) {
        let filtering = self.search.is_active();
        let row_height = ui.spacing().interact_size.y;

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            egui::ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                row_height,
                rows.len(),
                |ui, range| {
                    for row in &rows[range] {
                        let Some(node) = dom.get(row.id) else {
                            continue;
                        };

                        let out = TreeRow {
                            label: &node.name.to_str_lossy(),
                            depth: row.depth,
                            has_children: row.has_children,
                            expanded: filtering || self.expanded.contains(&row.id),
                            selected: selection.contains(&row.id),
                        }
                        .show(ui);

                        if out.response.clicked() {
                            *selection = vec![row.id];
                        }
                        if out.toggled && !self.expanded.remove(&row.id) {
                            self.expanded.insert(row.id);
                        }
                    }
                },
            );
        });
    }
}

impl DockWidget for Explorer {
    fn title(&self) -> egui::WidgetText {
        "Explorer".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        toolbar(ctx.ui, |ui| {
            self.search.ui(ui, "Search instances");
        });

        let dom = ctx.place.dom();
        let reflection = ctx.place.reflection();
        let game = ctx.place.world().game;

        let filter = self
            .search
            .is_active()
            .then(|| rows::matching_rows(&dom, game, |name| self.search.matches(name)));

        let rows = rows::flatten_rows(
            &dom,
            services(&dom, reflection, game),
            &self.expanded,
            filter.as_ref(),
        );

        self.list(ctx.ui, &dom, &rows, &mut ctx.selection);
    }
}
