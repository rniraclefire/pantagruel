use std::sync::Arc;

use rbx_dom_weak::{WeakDom, types::Ref};
use ror::scene::{Playback, Scene, extract::extract, load::Document};

use crate::place::{DomVersion, Place, dom::place_weak_dom};

const PREWARM: f32 = 10.0;

fn service(dom: &WeakDom, class_name: &str) -> Option<Ref> {
    dom.root()
        .children()
        .iter()
        .copied()
        .find(|&r| dom.get_by_ref(r).is_some_and(|i| i.class == class_name))
}

pub fn place_document(dom: WeakDom) -> Document {
    let workspace = service(&dom, "Workspace");
    let lighting = service(&dom, "Lighting");
    let material_service = service(&dom, "MaterialService");

    let roots = workspace
        .and_then(|ws| dom.get_by_ref(ws))
        .map(|ws| {
            ws.children()
                .iter()
                .copied()
                .filter(|&r| {
                    dom.get_by_ref(r)
                        .is_some_and(|i| !matches!(&*i.class, "Terrain" | "Camera"))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut document = Document::new(dom);
    document.is_place = true;
    document.roots = roots;
    document.lighting = lighting;
    document.material_service = material_service;

    document
}

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
            let document = place_document(place_weak_dom(place));
            let scene = Arc::new(extract(&document));
            *self = Self::Available(Cached { scene, document }, version);
        }

        match self {
            Self::Available(cached, _) => cached,
            Self::Pending => unreachable!(),
        }
    }
}
