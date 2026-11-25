pub fn row(ui: &mut egui::Ui, name: &str, value: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    let h = ui.spacing().interact_size.y;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), h), egui::Sense::click());
    let (name_rect, value_rect) = rect.split_left_right_at_x(ui.clip_rect().center().x);
    let cell = |r| {
        egui::UiBuilder::new()
            .max_rect(r)
            .layout(egui::Layout::left_to_right(egui::Align::Center))
    };
    ui.new_child(cell(name_rect))
        .add(egui::Label::new(name).truncate());
    value(&mut ui.new_child(cell(value_rect)));
    response
}
