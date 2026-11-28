use crate::theme::Theme;

pub struct TreeRow {
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    pub selected: bool,
}

pub struct TreeRowOutput {
    pub response: egui::Response,
    pub toggled: bool,
}

impl TreeRow {
    pub fn height(ui: &egui::Ui) -> f32 {
        Theme::of(ui).row_height
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> TreeRowOutput {
        let height = Self::height(ui);
        let indent = ui.spacing().indent;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::click(),
        );

        let caret_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + self.depth as f32 * indent, rect.top()),
            egui::vec2(indent, height),
        );

        {
            let visuals = ui.visuals();
            let painter = ui.painter();

            if self.selected {
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

            let guide = visuals.widgets.noninteractive.bg_stroke;
            for level in 0..self.depth {
                let x = rect.left() + level as f32 * indent + indent / 2.0;
                painter.vline(x, rect.y_range(), guide);
            }
        }

        // Caret
        let mut toggled = false;
        if self.has_children {
            let caret = ui.interact(caret_rect, response.id.with("caret"), egui::Sense::click());
            let visuals = ui.visuals();
            let color = if caret.hovered() {
                visuals.strong_text_color()
            } else {
                visuals.text_color()
            };
            let glyph = if self.expanded { "⏷" } else { "⏵" };
            let font = egui::TextStyle::Body.resolve(ui.style());
            ui.painter().text(
                caret_rect.center(),
                egui::Align2::CENTER_CENTER,
                glyph,
                font,
                color,
            );
            toggled = caret.clicked();
        }

        // Content
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(caret_rect.right() + ui.spacing().item_spacing.x, rect.top()),
            rect.max,
        );
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.set_clip_rect(content_rect.intersect(ui.clip_rect()));
        child.style_mut().interaction.selectable_labels = false;
        if self.selected {
            let color = child.visuals().selection.stroke.color;
            child.visuals_mut().override_text_color = Some(color);
        }
        content(&mut child);

        TreeRowOutput { response, toggled }
    }
}
