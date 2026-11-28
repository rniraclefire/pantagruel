use crate::editor::{
    EditorState, explorer::Explorer, output::Output, properties::Properties, viewport::Viewport,
};

pub struct DockContext<'a> {
    pub tile_id: egui_tiles::TileId,
    pub ui: &'a mut egui::Ui,
    pub state: &'a mut EditorState,
}

pub trait DockWidget {
    fn title(&self) -> egui::WidgetText;
    fn ui(&mut self, ctx: &mut DockContext<'_>) -> ();
}

pub type BoxedWidget = Box<dyn DockWidget>;
pub type Dock = egui_tiles::Tree<BoxedWidget>;

pub struct DockBehavior<'a>(pub &'a mut EditorState);

impl<'a> egui_tiles::Behavior<BoxedWidget> for DockBehavior<'a> {
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
            state: &mut self.0,
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

    fn tab_outline_stroke(
        &self,
        _visuals: &egui::Visuals,
        _tiles: &egui_tiles::Tiles<BoxedWidget>,
        _tile_id: egui_tiles::TileId,
        _state: &egui_tiles::TabState,
    ) -> egui::Stroke {
        egui::Stroke::NONE
    }
}

pub fn default_dock() -> Dock {
    static SIDEBAR_FRACTION: f32 = 0.25;
    static VIEWPORT_FRACTION: f32 = 0.75;

    use egui_tiles::{Linear, LinearDir};

    let mut tiles = egui_tiles::Tiles::<BoxedWidget>::default();

    let viewport = tiles.insert_pane(Box::new(Viewport::default()));
    let explorer = tiles.insert_pane(Box::new(Explorer::default()));
    let output = tiles.insert_pane(Box::new(Output::default()));
    let properties = tiles.insert_pane(Box::new(Properties::default()));

    let sidebar = tiles.insert_vertical_tile(vec![explorer, properties]);
    let main = tiles.insert_container(Linear::new_binary(
        LinearDir::Vertical,
        [viewport, output],
        VIEWPORT_FRACTION,
    ));
    let root = tiles.insert_container(Linear::new_binary(
        LinearDir::Horizontal,
        [sidebar, main],
        SIDEBAR_FRACTION,
    ));

    egui_tiles::Tree::new("dock", root, tiles)
}
