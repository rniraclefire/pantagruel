pub fn toolbar(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        egui::Frame::new()
            .inner_margin(ui.spacing().menu_margin)
            .show(ui, |ui| ui.horizontal(add_contents));
        ui.add(egui::Separator::default().spacing(0.0));
    });
}
