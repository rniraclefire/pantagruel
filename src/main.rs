mod app;
mod browser;
mod editor;
mod place;
mod widgets;

use app::App;

pub fn main() -> eframe::Result {
    eframe::run_native(
        "Petrova",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
