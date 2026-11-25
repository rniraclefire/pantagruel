use ror::render::ViewRequest;
use ror::render::camera::CameraSpec;

use super::freecam::CameraView;

pub fn request(ui: &egui::Ui, rect: egui::Rect, camera: &CameraView) -> ViewRequest {
    let scale = ui.ctx().pixels_per_point();

    let position = camera.cframe.position;
    let rotation = camera.cframe.orientation;
    let spec = CameraSpec::CFrame {
        components: [
            position.x,
            position.y,
            position.z,
            rotation.x.x,
            rotation.x.y,
            rotation.x.z,
            rotation.y.x,
            rotation.y.y,
            rotation.y.z,
            rotation.z.x,
            rotation.z.y,
            rotation.z.z,
        ],
        fov: camera.fov,
        near: None,
    };

    ViewRequest {
        width: (rect.width() * scale).round().max(1.0) as u32,
        height: (rect.height() * scale).round().max(1.0) as u32,
        camera: spec,
        msaa: 4,
        supersample: 1,
        texture: true,
        ..Default::default()
    }
}
