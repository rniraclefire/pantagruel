use std::collections::HashSet;

use crate::app::dock::{DockContext, DockWidget};

struct Row {
    id: ree::InstanceId,
    depth: usize,
    has_children: bool,
}

#[derive(Default)]
struct RowEvents {
    select: bool,
    toggle: bool,
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

    fn search_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .inner_margin(ui.spacing().menu_margin)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search")
                        .desired_width(f32::INFINITY),
                );
            });
    }

    fn tree(
        &mut self,
        ui: &mut egui::Ui,
        dom: &ree::Dom,
        root: ree::InstanceId,
        selection: &mut Vec<ree::InstanceId>,
    ) {
        let filter = (!self.query.is_empty()).then(|| self.search(dom, root));
        let mut rows = Vec::new();
        for &child in dom.children_of(root) {
            self.collect(dom, child, 0, filter.as_ref(), &mut rows);
        }

        let row_height = ui.spacing().interact_size.y;
        ui.spacing_mut().item_spacing.y = 0.0;

        egui::ScrollArea::vertical().auto_shrink(false).show_rows(
            ui,
            row_height,
            rows.len(),
            |ui, range| {
                ui.spacing_mut().item_spacing.y = 0.0;

                for row in &rows[range] {
                    let Some(node) = dom.get(row.id) else {
                        continue;
                    };
                    let expanded = filter.is_some() || self.expanded.contains(&row.id);
                    let selected = selection.contains(&row.id);

                    let events = row_ui(ui, row, &node.name.to_str_lossy(), expanded, selected);

                    if events.select {
                        *selection = vec![row.id];
                    }
                    if events.toggle {
                        self.toggle(row.id);
                    }
                }
            },
        );
    }
}

fn row_ui(ui: &mut egui::Ui, row: &Row, name: &str, expanded: bool, selected: bool) -> RowEvents {
    let row_height = ui.spacing().interact_size.y;
    let indent = ui.spacing().indent;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click(),
    );

    let visuals = ui.visuals();
    let painter = ui.painter();
    let font = egui::TextStyle::Body.resolve(ui.style());

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

    let indent_guide_stroke = visuals.widgets.noninteractive.bg_stroke;
    for level in 0..row.depth {
        let x = rect.left() + level as f32 * indent + indent / 2.0;
        painter.vline(x, rect.y_range(), indent_guide_stroke);
    }

    let caret_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + row.depth as f32 * indent, rect.top()),
        egui::vec2(indent, row_height),
    );
    let mut events = RowEvents::default();
    if row.has_children {
        let caret = ui.interact(caret_rect, response.id.with("caret"), egui::Sense::click());
        let color = if caret.hovered() {
            visuals.strong_text_color()
        } else {
            visuals.text_color()
        };
        let glyph = if expanded { "⏷" } else { "⏵" };
        painter.text(
            caret_rect.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            font.clone(),
            color,
        );
        events.toggle = caret.clicked();
    }

    let text_color = if selected {
        visuals.selection.stroke.color
    } else {
        visuals.text_color()
    };
    painter.text(
        egui::pos2(
            caret_rect.right() + ui.spacing().item_spacing.x,
            rect.center().y,
        ),
        egui::Align2::LEFT_CENTER,
        name,
        font,
        text_color,
    );

    events.select = response.clicked();
    events.toggle |= response.double_clicked() && row.has_children;
    events
}

impl DockWidget for Explorer {
    fn title(&self) -> egui::WidgetText {
        "Explorer".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        self.search_bar(ctx.ui);
        let state = ctx.place.emulator.engine.state.borrow();
        self.tree(
            ctx.ui,
            &state.dom,
            ctx.place.world.game,
            &mut ctx.place.selection,
        );
    }
}
