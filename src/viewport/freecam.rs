use glam::{Vec3, vec3};
use rbx_dom_weak::types::CFrame;

use crate::place::{
    Place,
    properties::{cframe, to_glam},
};

const LOOK_SPEED: f32 = 0.01;
const MOVE_SPEED: f32 = 80.0;
const SLOW_MOVE_SPEED: f32 = 20.0;
const PITCH_LIMIT: f32 = 1.55;
const DEFAULT_FOV: f32 = 70.0;

pub struct CameraView {
    pub cframe: CFrame,
    pub fov: f32,
}

#[derive(Default)]
pub struct Freecam {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    fov: f32,
}

impl Freecam {
    pub fn new(place: &Place) -> Self {
        let camera = place.world.camera;
        let fov = place
            .get_number(camera, "FieldOfView")
            .map_or(DEFAULT_FOV, |fov| fov as f32);

        let Some(start) = place.get_cframe(camera, "CFrame") else {
            return Self {
                position: vec3(0.0, 10.0, 20.0),
                yaw: 0.0,
                pitch: 0.0,
                fov,
            };
        };

        let (position, rotation) = to_glam(start);
        let look = -rotation.z_axis;

        Self {
            position,
            yaw: (-look.x).atan2(-look.z),
            pitch: look.y.clamp(-1.0, 1.0).asin(),
            fov,
        }
    }

    pub fn update(&mut self, ui: &egui::Ui, response: &egui::Response) -> bool {
        let mut moved = false;

        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            self.yaw -= delta.x * LOOK_SPEED;
            self.pitch = (self.pitch - delta.y * LOOK_SPEED).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            moved = true;
        }

        let can_move =
            (response.hovered() || response.dragged()) && !ui.ctx().wants_keyboard_input();

        if can_move {
            let (look, right) = self.axes();
            ui.input(|input| {
                let speed = if input.modifiers.shift {
                    SLOW_MOVE_SPEED
                } else {
                    MOVE_SPEED
                } * input.stable_dt;

                let bindings = [
                    (egui::Key::W, look),
                    (egui::Key::S, -look),
                    (egui::Key::D, right),
                    (egui::Key::A, -right),
                    (egui::Key::E, Vec3::Y),
                    (egui::Key::Space, Vec3::Y),
                    (egui::Key::Q, -Vec3::Y),
                ];

                for (key, direction) in bindings {
                    if input.key_down(key) {
                        self.position += direction * speed;
                        moved = true;
                    }
                }
            });
        }

        if moved {
            ui.ctx().request_repaint();
        }

        moved
    }

    pub fn view(&self) -> CameraView {
        CameraView {
            cframe: self.cframe(),
            fov: self.fov,
        }
    }

    pub fn axes(&self) -> (Vec3, Vec3) {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        (vec3(-sy * cp, sp, -cy * cp), vec3(cy, 0.0, -sy))
    }

    pub fn cframe(&self) -> CFrame {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        cframe(
            self.position.to_array(),
            [
                [cy, sy * sp, sy * cp],
                [0.0, cp, -sp],
                [-sy, cy * sp, cy * cp],
            ],
        )
    }
}
