pub fn ui(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.horizontal(|ui| {
            for label in vec!["Home", "Avatar", "UI", "Script", "Model", "Plugins"] {
                let _ = ui.button(label);
            }
        });
    });
}
