use std::collections::HashSet;

use crate::app::dock::{DockContext, DockWidget};

struct Row {
    id: ree::InstanceId,
    depth: usize,
    has_children: bool,
}

#[derive(Default)]
pub struct Explorer {
    query: String,
    expanded: HashSet<ree::InstanceId>,
}

impl Explorer {
    fn toggle(&mut self, id: ree::InstanceId) {
        if !self.expanded.remove(&id) {
            self.expanded.insert(id);
        }
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

    fn search(&self, dom: &ree::Dom, root: ree::InstanceId) -> HashSet<ree::InstanceId> {
        let query = self.query.to_lowercase();
        let mut keep = HashSet::new();

        for id in dom.descendants(root) {
            let Some(node) = dom.get(id) else { continue };
            if !node.name.to_str_lossy().to_lowercase().contains(&query) {
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
}

impl DockWidget for Explorer {
    fn title(&self) -> egui::WidgetText {
        "Explorer".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        ctx.ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text("Search")
                .desired_width(f32::INFINITY),
        );

        let state = ctx.place.emulator.engine.state.borrow();
        let dom = &state.dom;
        let root = ctx.place.world.game;

        let filter = (!self.query.is_empty()).then(|| self.search(dom, root));

        let mut rows = Vec::new();
        for &child in dom.children_of(root) {
            self.collect(dom, child, 0, filter.as_ref(), &mut rows);
        }

        let row_height = ctx.ui.spacing().interact_size.y;

        egui::ScrollArea::vertical().auto_shrink(false).show_rows(
            ctx.ui,
            row_height,
            rows.len(),
            |ui, range| {
                for row in &rows[range] {
                    let Some(node) = dom.get(row.id) else {
                        continue;
                    };
                    let expanded = filter.is_some() || self.expanded.contains(&row.id);

                    ui.horizontal(|ui| {
                        ui.add_space(row.depth as f32 * ui.spacing().indent);

                        let arrow = if expanded { "⏷" } else { "⏵" };
                        let arrow =
                            ui.add_visible(row.has_children, egui::Button::new(arrow).frame(false));

                        if arrow.clicked() {
                            self.toggle(row.id);
                        }

                        let response = ui.add(
                            egui::Button::selectable(
                                ctx.place.selection.contains(&row.id),
                                (node.name.to_str_lossy().into_owned(), egui::Atom::grow()),
                            )
                            .min_size(egui::vec2(ui.available_width(), 0.0)),
                        );

                        if response.clicked() {
                            ctx.place.selection = vec![row.id];
                        }

                        if response.double_clicked() && row.has_children {
                            self.toggle(row.id);
                        }
                    });
                }
            },
        );
    }
}
