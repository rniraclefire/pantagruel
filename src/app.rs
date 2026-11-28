use crate::{
    browser::Browser, editor::Editor, graphics::Graphics, place::Place, settings::Settings,
    theme::Theme,
};
use std::sync::Arc;

pub const APP_ID: &str = "pantagruel";

pub enum Screen {
    Browser(Browser),
    Editor(Editor),
}

pub struct App {
    pub graphics: Graphics,
    pub settings: Settings,
    pub screen: Screen,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        gpu: Arc<ror::render::gpu::Gpu>,
    ) -> anyhow::Result<Self> {
        let settings = Settings::load(cc.storage);
        let graphics = Graphics::new(gpu, cc, &settings)?;
        let screen = Screen::Browser(Browser::default());

        Theme::default().apply(&cc.egui_ctx);
        Ok(Self {
            screen,
            settings,
            graphics,
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &mut self.screen {
            Screen::Browser(browser) => {
                if let Some(path) = browser.show(ctx) {
                    match Place::open(&path, Default::default()) {
                        Ok(place) => {
                            self.screen = Screen::Editor(Editor::new(place, self.graphics.clone()))
                        }
                        Err(err) => log::error!("failed to open place: {err}"),
                    }
                }
            }
            Screen::Editor(editor) => editor.show(ctx),
        };
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.settings.save(storage);
    }
}
