use crate::{browser::Browser, editor::Editor, place::Place};

pub enum Screen {
    Browser(Browser),
    Editor(Editor),
}

pub struct App {
    pub screen: Screen,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            screen: Screen::Browser(Browser::default()),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &mut self.screen {
            Screen::Browser(browser) => {
                if let Some(path) = browser.show(ctx) {
                    match Place::open(&path, Default::default()) {
                        Ok(place) => self.screen = Screen::Editor(Editor::new(place)),
                        Err(err) => log::error!("failed to open place: {err}"),
                    }
                }
            }
            Screen::Editor(editor) => editor.show(ctx),
        };
    }
}
