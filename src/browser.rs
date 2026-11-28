use std::path::PathBuf;

use log::error;

use crate::{theme::Theme, widgets::layout::toolbar};

pub static TEMPLATE: &[u8] = include_bytes!("../assets/template.rbxl");

#[derive(Debug, Default, PartialEq)]
pub enum Sort {
    #[default]
    Modified,
    Name,
    Path,
}

#[derive(Debug, Default, PartialEq)]
pub enum View {
    #[default]
    List,
    Grid,
}

#[derive(Default)]
pub struct Browser {
    pub search: String,
    pub sort: Sort,
    pub view: View,
}

impl Browser {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<PathBuf> {
        let mut open: Option<PathBuf> = None;
        let style = ctx.style();
        let frame = egui::Frame::new().fill(style.visuals.panel_fill);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            toolbar(ui, |ui| {
                if ui.button("New Place").clicked() {
                    open = rfd::FileDialog::default()
                        .add_filter("Roblox Place", &["rbxl"])
                        .save_file()
                        .map(|path| path.with_extension("rbxl"))
                        .and_then(|path| match std::fs::write(&path, TEMPLATE) {
                            Ok(_) => Some(path),
                            Err(err) => {
                                error!("failed to save {:?}: {}", path, err);
                                None
                            }
                        });
                }

                if ui.button("Open Place").clicked() {
                    open = rfd::FileDialog::default()
                        .add_filter("Roblox Place", &["rbxl"])
                        .set_directory("/")
                        .pick_file()
                }

                ui.separator();

                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("Search places")
                    .min_size(egui::vec2(0.0, Theme::of(ui).control_height))
                    .show(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("browser_sort")
                        .selected_text(format!("{:?}", self.sort))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort, Sort::Modified, "Modified");
                            ui.selectable_value(&mut self.sort, Sort::Name, "Name");
                            ui.selectable_value(&mut self.sort, Sort::Path, "Path");
                        });

                    ui.label("Sort by");

                    ui.separator();

                    egui::ComboBox::from_id_salt("browser_view")
                        .selected_text(format!("{:?}", self.view))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.view, View::List, "List");
                            ui.selectable_value(&mut self.view, View::Grid, "Grid");
                        });

                    ui.label("View by");
                });
            });
        });

        open
    }
}
