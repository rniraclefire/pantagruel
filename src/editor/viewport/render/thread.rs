use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use anyhow::{Context, Result};
use eframe::wgpu;
use ror::render::{Renderer, ViewRequest};
use ror::scene::Scene;

pub struct Frame {
    texture: Option<wgpu::Texture>,
    renderer: Arc<Renderer>,
}

impl Frame {
    pub fn texture(&self) -> &wgpu::Texture {
        self.texture
            .as_ref()
            .expect("frame texture taken before drop")
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        if let Some(texture) = self.texture.take() {
            self.renderer.give_back(texture);
        }
    }
}

struct Job {
    scene: Arc<Scene>,
    view: ViewRequest,
}

pub struct Thread {
    jobs: Sender<Job>,
    frames: Receiver<Result<Frame>>,
}

impl Thread {
    pub fn new(renderer: Arc<Renderer>, egui: egui::Context) -> Self {
        let (jobs, job_receiver) = mpsc::channel();
        let (frame_sender, frames) = mpsc::channel();

        thread::Builder::new()
            .name("viewport-render".into())
            .spawn(move || run(&renderer, &job_receiver, &frame_sender, &egui))
            .expect("failed to spawn viewport render thread");

        Self { jobs, frames }
    }

    pub fn submit(&self, scene: Arc<Scene>, view: ViewRequest) {
        let _ = self.jobs.send(Job { scene, view });
    }

    pub fn latest(&self) -> Option<Result<Frame>> {
        self.frames.try_iter().last()
    }
}

fn run(
    renderer: &Arc<Renderer>,
    jobs: &Receiver<Job>,
    frames: &Sender<Result<Frame>>,
    egui: &egui::Context,
) {
    while let Ok(mut job) = jobs.recv() {
        while let Ok(newer) = jobs.try_recv() {
            job = newer;
        }

        if frames.send(render(renderer, &job)).is_err() {
            break;
        }
        egui.request_repaint();
    }
}

fn render(renderer: &Arc<Renderer>, job: &Job) -> Result<Frame> {
    let (images, _) = renderer.render(&job.scene, std::slice::from_ref(&job.view))?;
    let texture = images
        .into_iter()
        .next()
        .and_then(|image| image.texture)
        .context("render returned no texture")?;

    Ok(Frame {
        texture: Some(texture),
        renderer: renderer.clone(),
    })
}
