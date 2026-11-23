use std::time::{Duration, Instant};

use rbx_dom_weak::types::CFrame;

use super::freecam::Freecam;
use crate::place::Place;

const SETTLE_TIME: Duration = Duration::from_millis(150);

#[derive(Clone, Copy, PartialEq)]
pub enum Quality {
    Moving,
    Still,
}

pub struct CameraView {
    pub cframe: CFrame,
    pub fov: f32,
    pub quality: Quality,
    pub moved: bool,
}

#[derive(Default)]
pub struct ViewportCamera {
    freecam: Freecam,
    last_moved: Option<Instant>,
}

impl ViewportCamera {
    pub fn update(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        place: &mut Place,
    ) -> Option<CameraView> {
        let camera = place.world.camera;
        let mut cframe = place.get_cframe(camera, "CFrame")?;
        let fov = place.get_number(camera, "FieldOfView").unwrap_or(70.0) as f32;

        let moved = match self.freecam.update(ui, response, cframe) {
            Some(next) => {
                place.set_cframe(camera, "CFrame", next);
                cframe = next;
                self.last_moved = Some(Instant::now());
                ui.ctx().request_repaint();
                true
            }
            None => false,
        };

        let settling = self.last_moved.is_some_and(|at| at.elapsed() < SETTLE_TIME);
        if settling {
            ui.ctx().request_repaint_after(SETTLE_TIME);
        }

        let quality = if settling {
            Quality::Moving
        } else {
            Quality::Still
        };
        Some(CameraView {
            cframe,
            fov,
            quality,
            moved,
        })
    }
}
