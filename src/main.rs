mod app;
mod explorer;
mod place;
mod viewport;

use eframe::egui_wgpu;
use std::sync::Arc;

fn main() -> eframe::Result {
    env_logger::init();

    let gpu = Arc::new(ror::render::gpu::Gpu::new().expect("no usable GPU"));
    gpu.device.set_device_lost_callback(|reason, message| {
        eprintln!("lost gpu device: {reason:?}: {message}");
    });

    eframe::run_native(
        "Pantagruel",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: egui_wgpu::WgpuConfiguration {
                wgpu_setup: egui_wgpu::WgpuSetup::Existing(egui_wgpu::WgpuSetupExisting {
                    instance: gpu.instance.clone(),
                    adapter: gpu.adapter.clone(),
                    device: gpu.device.clone(),
                    queue: gpu.queue.clone(),
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(app::App::new(cc, gpu)?))),
    )
}
