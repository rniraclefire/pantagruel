pub mod camera;
pub mod freecam;
pub mod graphics;
pub mod scene;
pub mod texture;
pub mod view;
pub mod worker;

use crate::app::dock::{DockContext, DockWidget};
use camera::ViewportCamera;
use scene::SceneCache;
use texture::ViewportTexture;
use worker::RenderWorker;

#[derive(Default)]
pub struct Viewport {
    camera: ViewportCamera,
    scene: SceneCache,
    worker: Option<RenderWorker>,
    texture: ViewportTexture,
}

impl DockWidget for Viewport {
    fn title(&self) -> egui::WidgetText {
        "Viewport".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        let (rect, response) = ctx
            .ui
            .allocate_exact_size(ctx.ui.available_size(), egui::Sense::click_and_drag());

        let before = ctx.place.dom_version();
        let Some(camera) = self.camera.update(ctx.ui, &response, ctx.place) else {
            return;
        };
        if camera.moved {
            self.scene.is_still_valid(before, ctx.place.dom_version());
        }

        let scene = self.scene.get(ctx.place);
        let view = view::request(ctx.ui, rect, &camera);

        let worker = self.worker.get_or_insert_with(|| {
            RenderWorker::new(ctx.graphics.renderer.clone(), ctx.ui.ctx().clone())
        });
        worker.request(scene, view);
        if let Some(texture) = worker.latest() {
            self.texture.present(texture, ctx.graphics);
        }
        self.texture.paint(ctx.ui, rect);
    }
}
