pub mod dock;
pub mod session;
pub mod settings;

use std::path::Path;
use std::sync::Arc;

use self::dock::DockBehavior;
use self::session::Session;
use self::settings::Settings;
use crate::viewport::graphics::Graphics;
use crate::{ribbon_bar, status_bar};

pub const APP_ID: &str = "pantagruel";

pub struct App {
    pub graphics: Graphics,
    pub settings: Settings,
    pub sessions: Vec<Session>,
    pub active: usize,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        gpu: Arc<ror::render::gpu::Gpu>,
    ) -> anyhow::Result<Self> {
        let settings = Settings::load(cc.storage);
        let graphics = Graphics::new(gpu, cc, &settings)?;

        let mut app = Self {
            graphics,
            settings,
            sessions: Vec::new(),
            active: 0,
        };

        let startup = std::env::args_os()
            .nth(1)
            .map(std::path::PathBuf::from)
            .or_else(|| app.settings.last_place.clone());

        if let Some(path) = startup {
            app.open(&path);
        }

        cc.egui_ctx.style_mut(|s| s.animation_time = 0.0);

        Ok(app)
    }

    pub fn open(&mut self, path: &Path) {
        match Session::open(path, Default::default()) {
            Ok(session) => {
                self.sessions.push(session);
                self.active = self.sessions.len() - 1;
                self.settings.last_place = Some(path.to_owned());
            }
            Err(error) => log::error!("couldn't open {}: {error}", path.display()),
        }
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) {
        let mut close = None;
        ui.horizontal(|ui| {
            for (index, _) in self.sessions.iter().enumerate() {
                if ui.selectable_label(index == self.active, "TBA").clicked() {
                    self.active = index;
                }
                if ui.small_button("x").clicked() {
                    close = Some(index);
                }
            }
        });

        if let Some(index) = close {
            self.sessions.remove(index);
            self.active = self.active.min(self.sessions.len().saturating_sub(1));
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.sessions.len() > 1 {
            egui::TopBottomPanel::top("sessions").show(ctx, |ui| self.tab_strip(ui));
        }

        match self.sessions.get_mut(self.active) {
            None => {
                // tba
            }
            Some(session) => {
                egui::TopBottomPanel::top("ribbon").show(ctx, ribbon_bar::ui);
                egui::TopBottomPanel::bottom("status").show(ctx, status_bar::ui);

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::ZERO),
                    )
                    .show(ctx, |ui| {
                        let mut behavior = DockBehavior {
                            place: &mut session.place,
                            graphics: &self.graphics,
                            tree_id: session.dock.id(),
                        };
                        session.dock.ui(&mut behavior, ui);
                    });
            }
        };
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.settings.save(storage);
    }
}
