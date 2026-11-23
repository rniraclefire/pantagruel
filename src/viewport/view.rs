use ror::render::ViewRequest;
use ror::render::camera::CameraSpec;

use super::camera::{CameraView, Quality};

pub fn request(ui: &egui::Ui, rect: egui::Rect, camera: &CameraView) -> ViewRequest {
    let moving = camera.quality == Quality::Moving;
    let scale = ui.ctx().pixels_per_point() * if moving { 0.5 } else { 1.0 };
    let p = camera.cframe.position;
    let r = camera.cframe.orientation;

    ViewRequest {
        width: (rect.width() * scale).round().max(1.0) as u32,
        height: (rect.height() * scale).round().max(1.0) as u32,
        camera: CameraSpec::CFrame {
            components: [
                p.x, p.y, p.z, r.x.x, r.x.y, r.x.z, r.y.x, r.y.y, r.y.z, r.z.x, r.z.y, r.z.z,
            ],
            fov: camera.fov,
            near: None,
        },
        msaa: if moving { 1 } else { 4 },
        bloom: !moving,
        supersample: 1,
        texture: true,
        ..Default::default()
    }
}
