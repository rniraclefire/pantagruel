use std::collections::HashSet;

use crate::{
    app::dock::{DockContext, DockWidget},
    widgets::{search::Search, toolbar::toolbar},
};

/// The services Studio shows at the top of the Explorer, in its order.
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

struct Row {
    id: ree::InstanceId,
    depth: usize,
    has_children: bool,
}

#[derive(Default)]
pub struct Explorer {
    search: Search,
    expanded: HashSet<ree::InstanceId>,
}

impl Explorer {
    fn matches(&self, dom: &ree::Dom, root: ree::InstanceId) -> HashSet<ree::InstanceId> {
        let mut keep = HashSet::new();
        for id in dom.descendants(root) {
            let Some(node) = dom.get(id) else { continue };
            if !self.search.matches(&node.name.to_str_lossy()) {
                continue;
            }
            let mut current = Some(id);
            while let Some(ancestor) = current {
                if ancestor == root || !keep.insert(ancestor) {
                    break;
                }
                current = dom.parent_of(ancestor);
            }
        }
        keep
    }

    fn collect(
        &self,
        dom: &ree::Dom,
        id: ree::InstanceId,
        depth: usize,
        filter: Option<&HashSet<ree::InstanceId>>,
        rows: &mut Vec<Row>,
    ) {
        if filter.is_some_and(|keep| !keep.contains(&id)) {
            return;
        }
        let children = dom.children_of(id);
        rows.push(Row {
            id,
            depth,
            has_children: !children.is_empty(),
        });
        if filter.is_some() || self.expanded.contains(&id) {
            for &child in children {
                self.collect(dom, child, depth + 1, filter, rows);
            }
        }
    }

    fn toggle(&mut self, id: ree::InstanceId) {
        if !self.expanded.remove(&id) {
            self.expanded.insert(id);
        }
    }
}

fn row_ui(
    ui: &mut egui::Ui,
    row: &Row,
    name: &str,
    open: bool,
    selected: bool,
) -> (egui::Response, bool) {
    let indent = ui.spacing().indent;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
        egui::Sense::click(),
    );
    let left = rect.left() + row.depth as f32 * indent;

    {
        let visuals = ui.visuals();
        let painter = ui.painter();
        if selected {
            painter.rect_filled(
                rect,
                visuals.widgets.active.corner_radius,
                visuals.selection.bg_fill,
            );
        } else if response.hovered() {
            painter.rect_filled(
                rect,
                visuals.widgets.hovered.corner_radius,
                visuals.widgets.hovered.weak_bg_fill,
            );
        }
        for level in 0..row.depth {
            let x = rect.left() + level as f32 * indent + indent / 2.0;
            painter.vline(x, rect.y_range(), visuals.widgets.noninteractive.bg_stroke);
        }
    }

    let mut toggled = false;
    if row.has_children {
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(left + indent / 2.0, rect.center().y),
            egui::Vec2::splat(ui.spacing().icon_width_inner),
        );
        let icon = ui.interact(icon_rect, response.id.with("arrow"), egui::Sense::click());
        egui::collapsing_header::paint_default_icon(ui, if open { 1.0 } else { 0.0 }, &icon);
        toggled = icon.clicked();
    }

    let visuals = ui.visuals();
    let color = if selected {
        visuals.selection.stroke.color
    } else {
        visuals.text_color()
    };
    ui.painter().text(
        egui::pos2(left + indent, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        egui::TextStyle::Body.resolve(ui.style()),
        color,
    );

    (response, toggled)
}

impl DockWidget for Explorer {
    fn title(&self) -> egui::WidgetText {
        "Explorer".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        toolbar(ctx.ui, |ui| {
            self.search.ui(ui, "Search instances");
        });

        let state = ctx.place.emulator.engine.state.borrow();
        let reflection = &ctx.place.emulator.engine.reflection;
        let dom = &state.dom;
        let root = ctx.place.world.game;
        let selection = &mut ctx.place.selection;

        let filter = self.search.is_active().then(|| self.matches(dom, root));
        let mut rows = Vec::new();

        let services = VISIBLE_SERVICES
            .iter()
            .filter_map(|name| reflection.class_id(name))
            .filter_map(|class| dom.find_child_of_class(root, class));

        for service in services {
            self.collect(dom, service, 0, filter.as_ref(), &mut rows);
        }

        let row_height = ctx.ui.spacing().interact_size.y;
        egui::ScrollArea::vertical().auto_shrink(false).show_rows(
            ctx.ui,
            row_height,
            rows.len(),
            |ui, range| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for row in &rows[range] {
                    let Some(node) = dom.get(row.id) else {
                        continue;
                    };

                    let open = filter.is_some() || self.expanded.contains(&row.id);
                    let selected = selection.contains(&row.id);

                    let (response, toggled) =
                        row_ui(ui, row, &node.name.to_str_lossy(), open, selected);

                    if response.clicked() {
                        *selection = vec![row.id];
                    }

                    response.context_menu(|ui| {
                        if ui.button("Cut").clicked() {
                            // TODO
                        }

                        if ui.button("Copy").clicked() {
                            // TODO
                        }

                        ui.menu_button("Copy As", |ui| {
                            if ui.button("Copy Path").clicked() {
                                ui.ctx().copy_text(dom.full_name(reflection, row.id));
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

                        let class_name = &reflection.class(dom.class_of(row.id)).name;
                        if ui.button("Open Documentation").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
                                "https://create.roblox.com/docs/reference/engine/classes/{}",
                                class_name
                            )));
                            // TODO
                        }

                        if ui.button("Open API Reference").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
                                "https://robloxapi.github.io/ref/class/{}.html",
                                class_name
                            )));
                            // TODO
                        }
                    });

                    if toggled || (response.double_clicked() && row.has_children) {
                        self.toggle(row.id);
                    }
                }
            },
        );
    }
}
