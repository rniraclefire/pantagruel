use crate::app::dock::DockWidget;

#[derive(Default)]
pub struct Output {}

impl DockWidget for Output {
    fn title(&self) -> egui::WidgetText {
        "Output".into()
    }

    fn ui(&mut self, ctx: &mut crate::app::dock::DockContext<'_>) -> () {
        ctx.ui.label("WIP");
    }
}
