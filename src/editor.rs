pub mod commands;
pub mod dock;
pub mod explorer;
pub mod output;
pub mod properties;
pub mod viewport;

use crate::{editor::commands::Command, graphics::Graphics, place::Place};
use dock::{Dock, DockBehavior, default_dock};
use log::warn;

pub struct Editor {
    pub dock: Dock,
    pub state: EditorState,
}

pub struct EditorState {
    pub place: Place,
    pub selection: Vec<ree::InstanceId>,
    pub graphics: Graphics,
    pub commands: Vec<Command>,
}

impl Editor {
    pub fn new(place: Place, graphics: Graphics) -> Self {
        Self {
            dock: default_dock(),
            state: EditorState {
                place,
                selection: vec![],
                graphics,
                commands: vec![],
            },
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                self.dock.ui(&mut DockBehavior(&mut self.state), ui)
            });

        for command in std::mem::take(&mut self.state.commands) {
            if let Err(err) = command.run(&mut self.state) {
                warn!("error while running command: {err}");
            };
        }
    }
}
