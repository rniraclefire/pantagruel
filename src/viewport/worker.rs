use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use eframe::egui_wgpu::wgpu;
use ror::render::camera::CameraSpec;
use ror::render::{Renderer, ViewRequest};
use ror::scene::Scene;

const MIN_FRAME_TIME: Duration = Duration::from_millis(16);

struct Job {
    scene: Arc<Scene>,
    view: ViewRequest,
}

#[derive(PartialEq)]
struct JobKey {
    scene: usize,
    camera: [f32; 12],
    size: [u32; 2],
    msaa: u32,
}

impl JobKey {
    fn of(job: &Job) -> Self {
        let camera = match job.view.camera {
            CameraSpec::CFrame { components, .. } => components,
            _ => [0.0; 12],
        };
        Self {
            scene: Arc::as_ptr(&job.scene) as usize,
            camera,
            size: [job.view.width, job.view.height],
            msaa: job.view.msaa,
        }
    }
}

pub struct RenderWorker {
    renderer: Arc<Renderer>,
    jobs: Sender<Job>,
    frames: Receiver<wgpu::Texture>,
    last: Option<JobKey>,
}

impl RenderWorker {
    pub fn new(renderer: Arc<Renderer>, egui: egui::Context) -> Self {
        let (jobs, job_rx) = channel();
        let (frame_tx, frames) = channel();
        let thread_renderer = Arc::clone(&renderer);

        std::thread::Builder::new()
            .name("viewport-render".into())
            .spawn(move || run(&thread_renderer, &job_rx, &frame_tx, &egui))
            .expect("spawn viewport render thread");

        Self {
            renderer,
            jobs,
            frames,
            last: None,
        }
    }

    pub fn request(&mut self, scene: Arc<Scene>, view: ViewRequest) {
        let job = Job { scene, view };
        let key = JobKey::of(&job);
        if self.last.as_ref() == Some(&key) {
            return;
        }
        self.last = Some(key);
        let _ = self.jobs.send(job);
    }

    pub fn latest(&self) -> Option<wgpu::Texture> {
        let mut newest = None;
        for texture in self.frames.try_iter() {
            if let Some(older) = newest.replace(texture) {
                self.renderer.give_back(older);
            }
        }
        newest
    }
}

fn run(
    renderer: &Renderer,
    jobs: &Receiver<Job>,
    frames: &Sender<wgpu::Texture>,
    egui: &egui::Context,
) {
    let mut last = Instant::now() - MIN_FRAME_TIME;

    while let Ok(mut job) = jobs.recv() {
        if let Some(wait) = MIN_FRAME_TIME.checked_sub(last.elapsed()) {
            std::thread::sleep(wait);
        }
        while let Ok(newer) = jobs.try_recv() {
            job = newer;
        }
        last = Instant::now();

        match renderer.render(&job.scene, std::slice::from_ref(&job.view)) {
            Ok((mut images, _)) => {
                let Some(texture) = images.pop().and_then(|image| image.texture) else {
                    continue;
                };
                if frames.send(texture).is_err() {
                    break;
                }
                egui.request_repaint();
            }
            Err(error) => log::error!("viewport render failed: {error}"),
        }
    }
}
