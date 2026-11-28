use crate::place::Place;

fn service(dom: &rbx_dom_weak::WeakDom, class_name: &str) -> Option<rbx_dom_weak::types::Ref> {
    dom.root()
        .children()
        .iter()
        .copied()
        .find(|&r| dom.get_by_ref(r).is_some_and(|i| i.class == class_name))
}

pub fn document(place: &Place) -> ror::scene::load::Document {
    let dom = place.to_weak_dom();

    let workspace = service(&dom, "Workspace");
    let lighting = service(&dom, "Lighting");
    let material_service = service(&dom, "MaterialService");

    let roots = workspace
        .and_then(|workspace| dom.get_by_ref(workspace))
        .map(|workspace| {
            workspace
                .children()
                .iter()
                .copied()
                .filter(|&r| {
                    dom.get_by_ref(r)
                        .is_some_and(|i| !matches!(&*i.class, "Terrain" | "Camera"))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut document = ror::scene::load::Document::new(dom);
    document.is_place = true;
    document.roots = roots;
    document.lighting = lighting;
    document.material_service = material_service;
    document
}
