use std::sync::Arc;

use crate::place::{DomVersion, Place};
use rbx_dom_weak::WeakDom;
use ror::scene::load::Document;

#[derive(Default)]
pub struct SceneCache {
    built: Option<(Arc<ror::scene::Scene>, DomVersion)>,
}

impl SceneCache {
    pub fn get(&mut self, place: &Place) -> Arc<ror::scene::Scene> {
        let version = place.dom_version();
        if !self
            .built
            .as_ref()
            .is_some_and(|(_, built)| *built == version)
        {
            let document = place_document(place.to_weak_dom());
            self.built = Some((Arc::new(ror::scene::extract::extract(&document)), version));
        }
        Arc::clone(&self.built.as_ref().expect("just built").0)
    }

    pub fn is_still_valid(&mut self, before: DomVersion, after: DomVersion) {
        if let Some((_, built)) = &mut self.built {
            if *built == before {
                *built = after;
            }
        }
    }
}

fn place_document(dom: WeakDom) -> Document {
    let service = |class: &str| {
        dom.root()
            .children()
            .iter()
            .copied()
            .find(|&r| dom.get_by_ref(r).is_some_and(|i| &*i.class == class))
    };
    let workspace = service("Workspace");
    let lighting = service("Lighting");
    let material_service = service("MaterialService");

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
