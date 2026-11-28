use crate::theme::Theme;

#[derive(Default)]
pub struct Search {
    pub query: String,
}

impl Search {
    pub fn ui(&mut self, ui: &mut egui::Ui, hint: impl Into<egui::WidgetText>) -> egui::Response {
        ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text(hint)
                .min_size(egui::vec2(0.0, Theme::of(ui).control_height))
                .desired_width(f32::INFINITY),
        )
    }

    pub fn is_active(&self) -> bool {
        !self.query.trim().is_empty()
    }

    pub fn matches(&self, s: &str) -> bool {
        s.to_lowercase().contains(&self.query.trim().to_lowercase())
    }
}
