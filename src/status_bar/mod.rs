pub fn ui(ui: &mut egui::Ui) {
    ui.label(format!("{}fps", (1.0 / ui.input(|i| i.stable_dt)).round()));
}
