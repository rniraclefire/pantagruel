mod rows;

use crate::{
    editor::{
        EditorState,
        commands::Command,
        dock::{DockContext, DockWidget},
    },
    place::Place,
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

fn context_menu(
    ui: &mut egui::Ui,
    place: &Place,
    commands: &mut Vec<Command>,
    id: ree::InstanceId,
) {
    if ui.button("Cut").clicked() {
        // TODO
    }

    if ui.button("Copy").clicked() {
        // TODO
    }

    ui.menu_button("Copy As", |ui| {
        if ui.button("Copy Path").clicked() {
            ui.ctx().copy_text(place.full_name(id));
        }

        if ui.button("Copy as JSON").clicked() {
            // TODO
        }
    });

    if ui.button("Paste Into").clicked() {
        // TODO
    }

    if ui.button("Paste Into Original").clicked() {
        // TODO
    }

    ui.separator();

    if ui.button("Duplicate").clicked() {
        // TODO
    }

    if ui.button("Rename").clicked() {
        // TODO
    }

    if ui.button("Delete").clicked() {
        // TODO
    }

    ui.separator();

    if ui.button("Insert Object").clicked() {
        // TODO
    }

    ui.separator();

    ui.menu_button("Select", |ui| {
        if ui.button("Children").clicked() {
            // TODO
        }

        if ui.button("Descendants").clicked() {
            // TODO
        }
    });

    if ui.button("Expand All").clicked() {
        // TODO
    }

    if ui.button("Collapse All").clicked() {
        // TODO
    }

    ui.separator();

    let class_name = &place.class_name(id);
    if ui.button("Open Documentation").clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
            "https://create.roblox.com/docs/reference/engine/classes/{class_name}"
        )));
    }

    if ui.button("Open API Reference").clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
            "https://robloxapi.github.io/ref/class/{class_name}.html"
        )));
    }
}

impl Explorer {
    fn list(&mut self, ui: &mut egui::Ui, state: &mut EditorState, rows: Vec<Row>) {
        let place = &state.place;
        let selection = &state.selection;
        let commands = &mut state.commands;
        let dom = place.dom();

        let filtering = self.search.is_active();
        let row_height = TreeRow::height(ui);

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
                        depth: row.depth,
                        has_children: row.has_children,
                        expanded: filtering || self.expanded.contains(&row.id),
                        selected: selection.contains(&row.id),
                    }
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(&*node.name.to_str_lossy()).truncate());
                    });

                    out.response
                        .context_menu(|ui| context_menu(ui, place, commands, row.id));

                    if out.response.clicked() {
                        commands.push(Command::Select(vec![row.id]));
                    }

                    let toggle = out.toggled || (row.has_children && out.response.double_clicked());
                    if toggle && !self.expanded.remove(&row.id) {
                        self.expanded.insert(row.id);
                    }
                }
            },
        );
    }
}

impl DockWidget for Explorer {
    fn title(&self) -> egui::WidgetText {
        "Explorer".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        ctx.ui.spacing_mut().item_spacing.y = 0.0;

        toolbar(ctx.ui, |ui| {
            self.search.ui(ui, "Search instances");
        });

        let rows = {
            let place = &ctx.state.place;
            let dom = place.dom();
            let game = place.world().game;
            let filter = self
                .search
                .is_active()
                .then(|| rows::matching_rows(&dom, game, |name| self.search.matches(name)));
            rows::flatten_rows(
                &dom,
                services(&dom, place.reflection(), game),
                &self.expanded,
                filter.as_ref(),
            )
        };

        ctx.ui.scope(|ui| self.list(ui, ctx.state, rows));
    }
}
