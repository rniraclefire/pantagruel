pub mod dock;
pub mod explorer;

use crate::place::Place;
use dock::{Dock, DockBehavior, default_dock};

pub struct Editor {
    pub place: Place,
    pub selection: Vec<ree::InstanceId>,
    dock: Dock,
}

impl Editor {
    pub fn new(place: Place) -> Self {
        Self {
            place,
            selection: Vec::new(),
            dock: default_dock(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let Self {
            place,
            selection,
            dock,
        } = self;

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                dock.ui(&mut DockBehavior { place, selection }, ui)
            });
    }
}
