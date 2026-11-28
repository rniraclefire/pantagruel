use eframe::egui_wgpu::wgpu;

use super::thread::Frame;
use crate::graphics::Graphics;

#[derive(Default)]
pub enum Texture {
    #[default]
    Pending,
    Available(egui::TextureId),
}

impl Texture {
    pub fn present(&mut self, frame: Frame, graphics: &Graphics) {
        let view = frame
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default());
        let device = &graphics.render_state.device;
        let mut egui_renderer = graphics.render_state.renderer.write();

        let id = match self {
            Self::Pending => {
                egui_renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear)
            }
            Self::Available(id) => {
                egui_renderer.update_egui_texture_from_wgpu_texture(
                    device,
                    &view,
                    wgpu::FilterMode::Linear,
                    *id,
                );
                *id
            }
        };
        *self = Self::Available(id);
    }

    pub fn paint(&self, ui: &egui::Ui, rect: egui::Rect) {
        if let Self::Available(id) = self {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(*id, rect, uv, egui::Color32::WHITE);
        }
    }
}
