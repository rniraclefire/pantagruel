pub struct TreeRow<'a> {
    pub label: &'a str,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    pub selected: bool,
}

pub struct TreeRowOutput {
    pub response: egui::Response,
    pub toggled: bool,
}

impl TreeRow<'_> {
    pub fn show(self, ui: &mut egui::Ui) -> TreeRowOutput {
        let row_height = ui.spacing().interact_size.y;
        let indent = ui.spacing().indent;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_height),
            egui::Sense::click(),
        );

        let visuals = ui.visuals();
        let painter = ui.painter();
        let font = egui::TextStyle::Body.resolve(ui.style());

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

        let caret_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + self.depth as f32 * indent, rect.top()),
            egui::vec2(indent, row_height),
        );

        let mut toggled = false;
        if self.has_children {
            let caret = ui.interact(caret_rect, response.id.with("caret"), egui::Sense::click());
            let color = if caret.hovered() {
                visuals.strong_text_color()
            } else {
                visuals.text_color()
            };
            let glyph = if self.expanded { "⏷" } else { "⏵" };
            painter.text(
                caret_rect.center(),
                egui::Align2::CENTER_CENTER,
                glyph,
                font.clone(),
                color,
            );
            toggled = caret.clicked() || response.double_clicked();
        }

        let text_color = if self.selected {
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
            self.label,
            font,
            text_color,
        );

        TreeRowOutput { response, toggled }
    }
}
