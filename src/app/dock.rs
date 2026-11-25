use crate::{
    explorer::Explorer,
    output::Output,
    place::Place,
    properties::Properties,
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
    pub tree_id: egui::Id,
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

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<BoxedWidget>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        let size = egui::Vec2::splat(ui.spacing().interact_size.y);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let response = ui.interact(rect, tile_id.egui_id(self.tree_id), egui::Sense::drag());

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "⠿",
            egui::TextStyle::Body.resolve(ui.style()),
            ui.style().interact(&response).fg_stroke.color,
        );

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
    }
}

const SIDEBAR_FRACTION: f32 = 0.25;
const VIEWPORT_FRACTION: f32 = 0.75;

pub fn default_dock() -> Dock {
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
