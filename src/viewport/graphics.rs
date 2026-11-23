use std::sync::Arc;

use eframe::egui_wgpu;

use crate::app::{APP_ID, settings::Settings};

#[derive(Clone)]
pub struct Graphics {
    pub renderer: Arc<ror::render::Renderer>,
    pub render_state: egui_wgpu::RenderState,
}

impl Graphics {
    pub fn new(
        gpu: Arc<ror::render::gpu::Gpu>,
        cc: &eframe::CreationContext<'_>,
        settings: &Settings,
    ) -> anyhow::Result<Self> {
        let render_state = cc
            .wgpu_render_state
            .clone()
            .ok_or_else(|| anyhow::anyhow!("eframe isn't using the wgpu renderer"))?;

        let state_dir = eframe::storage_dir(APP_ID).unwrap_or_else(|| ".pantagruel".into());

        let mut config = ror_assets::AssetConfig::new(state_dir.join("ror"));
        config.studio_content = settings.studio_content();
        config.offline = settings.offline;

        let renderer = ror::render::Renderer::new(gpu, ror_assets::Assets::new(config))?;

        Ok(Self {
            renderer: Arc::new(renderer),
            render_state,
        })
    }
}
