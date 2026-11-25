use std::sync::Arc;

use rbx_dom_weak::{WeakDom, types::Ref};
use ror::scene::{Scene, extract::extract, load::Document};

use crate::{
    place::{DomVersion, Place, dom::place_weak_dom},
    viewport::scene::SceneCache::Available,
};

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

#[derive(Default)]
pub enum SceneCache {
    #[default]
    Pending,
    Available(Arc<Scene>, DomVersion),
}

impl SceneCache {
    pub fn get(&mut self, place: &Place) -> Arc<Scene> {
        let next_version = place.dom_version();
        if let Self::Available(scene, prev_version) = self
            && *prev_version == next_version
        {
            scene.clone()
        } else {
            let document = place_document(place_weak_dom(place));
            let scene = Arc::new(extract(&document));
            *self = Available(scene.clone(), next_version);
            scene
        }
    }
}
