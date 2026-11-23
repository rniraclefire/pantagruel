use std::collections::HashMap;

use rbx_dom_weak::types::Ref;
use rbx_dom_weak::{InstanceBuilder, WeakDom};
use ree::record::encode;

use super::Place;

const SKIPPED_CLASSES: &[&str] = &["SurfaceGui", "BillboardGui", "ScreenGui"];
const KEPT_UNMODELED: &[&str] = &["InitialSize", "MeshSize"];

impl Place {
    pub fn to_weak_dom(&self) -> WeakDom {
        let engine = &self.emulator.engine;
        let state = engine.state.borrow();
        let dom = &state.dom;

        let mut weak = WeakDom::new(InstanceBuilder::new("DataModel"));
        let copies = copy_tree(dom, &engine.reflection, &mut weak, self.world.game);

        for (&id, &copy) in &copies {
            if let Some(target) = weak.get_by_ref_mut(copy) {
                copy_properties(engine, dom, &copies, id, target);
            }
        }

        weak
    }
}

fn class_name<'a>(reflection: &'a ree::Reflection, dom: &ree::Dom, id: ree::InstanceId) -> &'a str {
    &reflection.class(dom.class_of(id)).name
}

fn copy_tree(
    dom: &ree::Dom,
    reflection: &ree::Reflection,
    weak: &mut WeakDom,
    root: ree::InstanceId,
) -> HashMap<ree::InstanceId, Ref> {
    let mut copies = HashMap::new();
    let mut stack: Vec<_> = dom
        .children_of(root)
        .iter()
        .rev()
        .map(|&id| (id, weak.root_ref()))
        .collect();

    while let Some((id, parent)) = stack.pop() {
        let Some(instance) = dom.get(id) else {
            continue;
        };
        let class = class_name(reflection, dom, id);
        if SKIPPED_CLASSES.contains(&class) {
            continue;
        }

        let copy = weak.insert(
            parent,
            InstanceBuilder::new(class).with_name(instance.name.to_str_lossy()),
        );
        copies.insert(id, copy);
        stack.extend(instance.children.iter().rev().map(|&child| (child, copy)));
    }

    copies
}

fn copy_properties(
    engine: &ree::Engine,
    dom: &ree::Dom,
    copies: &HashMap<ree::InstanceId, Ref>,
    id: ree::InstanceId,
    target: &mut rbx_dom_weak::Instance,
) {
    let reflection = &engine.reflection;
    let resolve = |other: ree::InstanceId| copies.get(&other).copied();

    for (property, data_type) in encode::serialized_properties(class_name(reflection, dom, id)) {
        if property == "Name" {
            continue;
        }
        let Some(value) = reflection
            .name_id(&property)
            .and_then(|name| dom.raw_get(id, name))
        else {
            continue;
        };
        if let Some(variant) = encode::to_variant(engine, value, data_type, &resolve) {
            target.properties.insert(property.as_ref().into(), variant);
        }
    }

    let Some(unmodeled) = &dom.at(id).unmodeled else {
        return;
    };
    for (name, variant) in unmodeled
        .iter()
        .filter(|(name, _)| KEPT_UNMODELED.contains(name))
    {
        target.properties.insert(name.into(), variant);
    }
}
