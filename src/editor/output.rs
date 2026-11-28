use crate::editor::dock::{DockContext, DockWidget};

#[derive(Default)]
pub struct Output {}

impl DockWidget for Output {
    fn title(&self) -> egui::WidgetText {
        "Output".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) -> () {}
}
