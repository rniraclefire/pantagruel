pub fn ui(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        for label in vec!["Home", "Avatar", "UI", "Script", "Model", "Plugins"] {
            let _ = ui.menu_button(label, |ui| {});
        }
    });
}
