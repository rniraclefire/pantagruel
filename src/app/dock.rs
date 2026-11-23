use crate::{
    explorer::Explorer,
    place::Place,
    viewport::{Viewport, graphics::Graphics},
};

pub struct DockContext<'a> {
    pub ui: &'a mut egui::Ui,
    pub tile_id: egui_tiles::TileId,
    pub graphics: &'a Graphics,
    pub place: &'a mut Place,
}

pub trait DockWidget {
    fn title(&self) -> egui::WidgetText;
    fn ui(&mut self, ctx: &mut DockContext<'_>) -> ();
}

pub type BoxedWidget = Box<dyn DockWidget>;
pub type Dock = egui_tiles::Tree<BoxedWidget>;

pub struct DockBehavior<'a> {
    pub graphics: &'a Graphics,
    pub place: &'a mut Place,
}

impl egui_tiles::Behavior<BoxedWidget> for DockBehavior<'_> {
    fn tab_title_for_pane(&mut self, widget: &BoxedWidget) -> egui::WidgetText {
        widget.title()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        widget: &mut BoxedWidget,
    ) -> egui_tiles::UiResponse {
        let mut ctx = DockContext {
            ui,
            tile_id,
            graphics: &mut self.graphics,
            place: &mut self.place,
        };
        widget.ui(&mut ctx);
        egui_tiles::UiResponse::None
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

pub fn default_dock() -> Dock {
    let mut tiles: egui_tiles::Tiles<BoxedWidget> = egui_tiles::Tiles::default();
    let viewport = tiles.insert_pane(Box::new(Viewport::default()));
    let explorer = tiles.insert_pane(Box::new(Explorer::default()));
    let root = tiles.insert_horizontal_tile(vec![explorer, viewport]);
    egui_tiles::Tree::new("dock", root, tiles)
}
