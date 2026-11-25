pub mod freecam;
pub mod graphics;
pub mod scene;
pub mod texture;
pub mod thread;
pub mod view;

use self::{freecam::Freecam, scene::SceneCache, texture::Texture, thread::Thread};
use crate::app::dock::{DockContext, DockWidget};
use ror::scene::Scene;
use std::sync::Arc;

#[derive(Default)]
pub struct Viewport {
    camera: Option<Freecam>,
    scene: SceneCache,
    thread: Option<Thread>,
    submitted: Option<(Arc<Scene>, [u32; 2])>,
    texture: Texture,
}

impl DockWidget for Viewport {
    fn title(&self) -> egui::WidgetText {
        "Viewport".into()
    }

    fn ui(&mut self, ctx: &mut DockContext<'_>) {
        let ui = &mut *ctx.ui;
        let place = &*ctx.place;
        let graphics = ctx.graphics;

        let (rect, response) = egui::Frame::new()
            .inner_margin(ui.spacing().window_margin)
            .show(ui, |ui| {
                ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag())
            })
            .inner;

        if rect.width() < 1.0 || rect.height() < 1.0 {
            return;
        }

        let camera = self.camera.get_or_insert_with(|| Freecam::new(place));
        let moved = camera.update(ui, &response);
        let view = view::request(ui, rect, &camera.view());
        let size = [view.width, view.height];

        let cached = self.scene.get(place);
        let animated = cached.animated();
        let scene = if animated {
            cached.at(ui.input(|i| i.time) as f32)
        } else {
            cached.scene.clone()
        };

        let thread = self
            .thread
            .get_or_insert_with(|| Thread::new(graphics.renderer.clone(), ui.ctx().clone()));

        let changed = moved
            || self
                .submitted
                .as_ref()
                .is_none_or(|(last, last_size)| !Arc::ptr_eq(last, &scene) || *last_size != size);

        if changed {
            thread.submit(scene.clone(), view);
            self.submitted = Some((scene, size));
        }

        match thread.latest() {
            Some(Ok(frame)) => self.texture.present(frame, graphics),
            Some(Err(error)) => log::error!("render failed: {error:#}"),
            None => {}
        }

        self.texture.paint(ui, rect);

        if animated {
            ui.ctx().request_repaint();
        }
    }
}
