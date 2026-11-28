#[derive(Clone)]
pub struct Theme {
    pub control_height: f32,
    pub row_height: f32,
    pub button_padding: egui::Vec2,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            control_height: 20.0,
            row_height: 24.0,
            button_padding: egui::vec2(6.0, 2.0),
        }
    }
}

impl Theme {
    pub fn apply(&self, ctx: &egui::Context) {
        ctx.style_mut(|style| {
            style.spacing.interact_size.y = self.control_height;
            style.spacing.button_padding = self.button_padding;
        });
        ctx.data_mut(|data| data.insert_temp(egui::Id::NULL, self.clone()));
    }

    pub fn of(ui: &egui::Ui) -> Self {
        ui.ctx()
            .data(|data| data.get_temp::<Self>(egui::Id::NULL))
            .unwrap_or_default()
    }
}
