use super::document::document;
use crate::place::{DomVersion, Place};
use ror::scene::{Playback, Scene, extract::extract, load::Document};
use std::sync::Arc;

const PREWARM: f32 = 10.0;

pub struct Cached {
    pub scene: Arc<Scene>,
    pub document: Document,
}

impl Cached {
    pub fn animated(&self) -> bool {
        !self.scene.particles.is_empty()
    }

    pub fn at(&self, time: f32) -> Arc<Scene> {
        if !self.animated() {
            return self.scene.clone();
        }

        let playback = Playback {
            time,
            prewarm: PREWARM,
            ..Default::default()
        };
        let resolved = match playback.resolve(&self.document) {
            Ok(resolved) => resolved,
            Err(error) => {
                log::error!("particle playback failed: {error:#}");
                return self.scene.clone();
            }
        };

        let mut scene = (*self.scene).clone();
        resolved.apply_history(&self.document, &mut scene);
        Arc::new(scene)
    }
}

#[derive(Default)]
pub enum SceneCache {
    #[default]
    Pending,
    Available(Cached, DomVersion),
}

impl SceneCache {
    pub fn get(&mut self, place: &Place) -> &Cached {
        let version = place.dom_version();
        if !matches!(self, Self::Available(_, prev) if *prev == version) {
            let document = document(place);
            let scene = Arc::new(extract(&document));
            *self = Self::Available(Cached { scene, document }, version);
        }

        match self {
            Self::Available(cached, _) => cached,
            Self::Pending => unreachable!(),
        }
    }
}
