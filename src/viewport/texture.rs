use eframe::egui_wgpu::wgpu;

use super::graphics::Graphics;

#[derive(Default)]
pub struct ViewportTexture {
    shown: Option<(egui::TextureId, wgpu::Texture)>,
}

impl ViewportTexture {
    pub fn present(&mut self, texture: wgpu::Texture, graphics: &Graphics) {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let device = &graphics.render_state.device;
        let mut egui_renderer = graphics.render_state.renderer.write();

        let id = match self.shown.take() {
            Some((id, old)) => {
                egui_renderer.update_egui_texture_from_wgpu_texture(
                    device,
                    &view,
                    wgpu::FilterMode::Linear,
                    id,
                );
                graphics.renderer.give_back(old);
                id
            }
            None => egui_renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear),
        };
        self.shown = Some((id, texture));
    }

    pub fn paint(&self, ui: &egui::Ui, rect: egui::Rect) {
        if let Some((id, _)) = &self.shown {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(*id, rect, uv, egui::Color32::WHITE);
        }
    }
}
