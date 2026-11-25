use crate::app::dock::DockWidget;

#[derive(Default)]
pub struct Properties {}

impl DockWidget for Properties {
    fn title(&self) -> egui::WidgetText {
        "Properties".into()
    }

    fn ui(&mut self, ctx: &mut crate::app::dock::DockContext<'_>) -> () {
        ctx.ui.label("WIP");
    }
}
