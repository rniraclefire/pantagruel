use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::path::Path;

pub type DomVersion = (u64, u64, u32);

pub struct Place {
    pub emulator: ree::Emulator,
    pub selection: Vec<ree::InstanceId>,
}

impl Place {
    pub fn open(path: &Path, options: ree::Options) -> anyhow::Result<Self> {
        let engine = ree::Engine::new(ree::Options {
            mode: "edit".into(),
            ..options
        })
        .map_err(anyhow::Error::msg)?;

        let emulator = ree::Emulator::start(engine, &ree::mount::ProjectSpec::default())
            .map_err(anyhow::Error::msg)?;

        let place = Self {
            emulator,
            selection: vec![],
        };

        ree::project::load_snapshot(place.engine(), path, place.world().game)
            .map_err(anyhow::Error::msg)?;
        Ok(place)
    }

    pub fn engine(&self) -> &ree::Engine {
        &self.emulator.engine
    }

    pub fn reflection(&self) -> &ree::Reflection {
        &self.engine().reflection
    }

    pub fn world(&self) -> ree::World {
        self.engine().world()
    }

    pub fn state(&self) -> Ref<'_, ree::State> {
        self.engine().state.borrow()
    }

    pub fn state_mut(&self) -> RefMut<'_, ree::State> {
        self.engine().state.borrow_mut()
    }

    pub fn hooks(&self) -> Ref<'_, ree::Hooks> {
        self.engine().hooks.borrow()
    }

    pub fn dom(&self) -> Ref<'_, ree::Dom> {
        Ref::map(self.state(), |state| &state.dom)
    }

    pub fn dom_version(&self) -> DomVersion {
        let dom = self.dom();
        (dom.structure_version, dom.write_version, dom.name_version)
    }

    pub fn name(&self, id: ree::InstanceId) -> String {
        self.dom()
            .get(id)
            .map(|instance| instance.name.to_str_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn class_name(&self, id: ree::InstanceId) -> &str {
        let class = self.dom().class_of(id);
        &self.reflection().class(class).name
    }

    pub fn full_name(&self, id: ree::InstanceId) -> String {
        self.dom().full_name(self.reflection(), id)
    }

    pub fn children(&self, id: ree::InstanceId) -> Vec<ree::InstanceId> {
        self.dom().children_of(id).to_vec()
    }

    pub fn has_children(&self, id: ree::InstanceId) -> bool {
        !self.dom().children_of(id).is_empty()
    }

    pub fn service(&self, class_name: &str) -> Option<ree::InstanceId> {
        let class = self.reflection().class_id(class_name)?;
        self.dom().find_child_of_class(self.world().game, class)
    }

    pub fn insert(&self, parent: ree::InstanceId, class: &str) -> anyhow::Result<ree::InstanceId> {
        ree::instance::new_instance(
            &mut self.state_mut(),
            self.reflection(),
            &self.hooks(),
            class,
            Some(parent),
        )
        .map_err(|error| anyhow::anyhow!("couldn't insert {class}: {error:?}"))
    }

    pub fn set(&self, id: ree::InstanceId, name: ree::NameId, value: ree::Value) -> bool {
        ree::instance::raw_set(
            &mut self.state_mut(),
            self.reflection(),
            &self.hooks(),
            id,
            name,
            value,
        )
    }

    pub fn rename(&self, id: ree::InstanceId, name: &str) -> anyhow::Result<()> {
        let name_id = self
            .reflection()
            .name_id("Name")
            .ok_or_else(|| anyhow::anyhow!("no Name property"))?;
        self.set(id, name_id, ree::Value::String(name.into()));
        Ok(())
    }

    pub fn delete(&self, ids: &[ree::InstanceId]) -> anyhow::Result<()> {
        let mut state = self.state_mut();
        let hooks = self.hooks();
        for &id in ids {
            ree::instance::set_parent(&mut state, self.reflection(), &hooks, id, None)
                .map_err(|error| anyhow::anyhow!("couldn't delete: {error:?}"))?;
        }
        Ok(())
    }

    pub fn to_weak_dom(&self) -> rbx_dom_weak::WeakDom {
        let mut weak = rbx_dom_weak::WeakDom::new(rbx_dom_weak::InstanceBuilder::new("DataModel"));
        let copies = self.copy_tree(&mut weak, self.world().game);

        for (&id, &copy) in &copies {
            if let Some(target) = weak.get_by_ref_mut(copy) {
                self.copy_properties(&copies, id, target);
            }
        }

        weak
    }

    fn copy_tree(
        &self,
        weak: &mut rbx_dom_weak::WeakDom,
        root: ree::InstanceId,
    ) -> HashMap<ree::InstanceId, rbx_dom_weak::types::Ref> {
        let dom = self.dom();
        let mut copies = HashMap::new();
        let mut stack: Vec<_> = dom
            .children_of(root)
            .iter()
            .rev()
            .map(|&id| (id, weak.root_ref()))
            .collect();

        while let Some((id, parent)) = stack.pop() {
            let Some(node) = dom.get(id) else {
                continue;
            };
            let copy = weak.insert(
                parent,
                rbx_dom_weak::InstanceBuilder::new(self.class_name(id))
                    .with_name(node.name.to_str_lossy()),
            );
            copies.insert(id, copy);
            stack.extend(node.children.iter().rev().map(|&child| (child, copy)));
        }

        copies
    }

    fn copy_properties(
        &self,
        copies: &HashMap<ree::InstanceId, rbx_dom_weak::types::Ref>,
        id: ree::InstanceId,
        target: &mut rbx_dom_weak::Instance,
    ) {
        let dom = self.dom();
        let reflection = self.reflection();
        let resolve = |other: ree::InstanceId| copies.get(&other).copied();

        for (property, data_type) in ree::record::encode::serialized_properties(self.class_name(id))
        {
            if property == "Name" {
                continue;
            }
            let Some(value) = reflection
                .name_id(&property)
                .and_then(|name| dom.raw_get(id, name))
            else {
                continue;
            };
            if let Some(variant) =
                ree::record::encode::to_variant(self.engine(), value, data_type, &resolve)
            {
                target.properties.insert(property.as_ref().into(), variant);
            }
        }

        let Some(unmodeled) = &dom.at(id).unmodeled else {
            return;
        };

        for (name, variant) in unmodeled.iter() {
            target.properties.insert(name.into(), variant);
        }
    }
}
