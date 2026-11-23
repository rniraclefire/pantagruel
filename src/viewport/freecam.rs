use rbx_dom_weak::types::CFrame;

use crate::place::properties::cframe;

const LOOK_SPEED: f32 = 0.005;
const MOVE_SPEED: f32 = 20.0;
const FAST_MOVE_SPEED: f32 = 80.0;
const PITCH_LIMIT: f32 = 1.55;

#[derive(Default)]
pub struct Freecam {
    yaw: f32,
    pitch: f32,
    oriented: bool,
}

impl Freecam {
    pub fn update(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        current: CFrame,
    ) -> Option<CFrame> {
        if !self.oriented {
            let look = [
                -current.orientation.x.z,
                -current.orientation.y.z,
                -current.orientation.z.z,
            ];
            self.yaw = (-look[0]).atan2(-look[2]);
            self.pitch = look[1].clamp(-1.0, 1.0).asin();
            self.oriented = true;
        }

        let mut changed = false;

        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            self.yaw -= delta.x * LOOK_SPEED;
            self.pitch = (self.pitch - delta.y * LOOK_SPEED).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            changed = true;
        }

        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let look = [-sy * cp, sp, -cy * cp];
        let right = [cy, 0.0, -sy];
        let up = [0.0, 1.0, 0.0];

        let mut position = [current.position.x, current.position.y, current.position.z];

        let can_move =
            (response.hovered() || response.dragged()) && !ui.ctx().wants_keyboard_input();

        if can_move {
            ui.input(|input| {
                let speed = if input.modifiers.shift {
                    FAST_MOVE_SPEED
                } else {
                    MOVE_SPEED
                } * input.stable_dt;
                let bindings = [
                    (egui::Key::W, look, speed),
                    (egui::Key::S, look, -speed),
                    (egui::Key::D, right, speed),
                    (egui::Key::A, right, -speed),
                    (egui::Key::E, up, speed),
                    (egui::Key::Q, up, -speed),
                ];
                for (key, direction, amount) in bindings {
                    if input.key_down(key) {
                        for axis in 0..3 {
                            position[axis] += direction[axis] * amount;
                        }
                        changed = true;
                    }
                }
            });
        }

        changed.then(|| {
            cframe(
                position,
                [
                    [cy, sy * sp, sy * cp],
                    [0.0, cp, -sp],
                    [-sy, cy * sp, cy * cp],
                ],
            )
        })
    }
}
