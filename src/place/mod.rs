use std::{
    cell::{Ref, RefMut},
    path::Path,
};

pub mod dom;
pub mod properties;

pub enum Command {
    Select(Vec<ree::InstanceId>),
    Insert {
        parent: ree::InstanceId,
        class: String,
    },
    Rename {
        id: ree::InstanceId,
        name: String,
    },
    SetProperty {
        id: ree::InstanceId,
        name: ree::NameId,
        value: ree::Value,
    },
    Delete(Vec<ree::InstanceId>),
}

pub type DomVersion = (u64, u64, u32);

pub struct Place {
    pub emulator: ree::Emulator,
    pub world: ree::World,
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

        let world = emulator.engine.world();

        ree::project::load_snapshot(&emulator.engine, path, world.game)
            .map_err(anyhow::Error::msg)?;

        Ok(Self {
            emulator,
            world,
            selection: vec![],
        })
    }

    pub fn engine(&self) -> &ree::Engine {
        &self.emulator.engine
    }

    pub fn reflection(&self) -> &ree::Reflection {
        &self.emulator.engine.reflection
    }

    pub fn state(&self) -> Ref<'_, ree::State> {
        self.emulator.engine.state.borrow()
    }

    pub fn state_mut(&self) -> RefMut<'_, ree::State> {
        self.emulator.engine.state.borrow_mut()
    }

    pub fn dom(&self) -> Ref<'_, ree::Dom> {
        Ref::map(self.state(), |state| &state.dom)
    }

    pub fn hooks(&self) -> Ref<'_, ree::Hooks> {
        self.emulator.engine.hooks.borrow()
    }

    pub fn dom_version(&self) -> DomVersion {
        let state = self.state();
        (
            state.dom.structure_version,
            state.dom.write_version,
            state.dom.name_version,
        )
    }

    pub fn name(&self, id: ree::InstanceId) -> String {
        self.dom()
            .get(id)
            .map(|instance| instance.name.to_str_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn class_name(&self, id: ree::InstanceId) -> Box<str> {
        self.reflection()
            .class(self.dom().class_of(id))
            .name
            .clone()
    }

    pub fn children(&self, id: ree::InstanceId) -> Vec<ree::InstanceId> {
        self.dom().children_of(id).to_vec()
    }

    pub fn has_children(&self, id: ree::InstanceId) -> bool {
        !self.state().dom.children_of(id).is_empty()
    }

    pub fn full_name(&self, id: ree::InstanceId) -> String {
        self.dom().full_name(&self.emulator.engine.reflection, id)
    }

    pub fn services(&self) -> Vec<ree::InstanceId> {
        static VISIBLE_SERVICES: &[&str] = &[
            "Workspace",
            "Players",
            "Lighting",
            "MaterialService",
            "ReplicatedFirst",
            "ReplicatedStorage",
            "ServerScriptService",
            "ServerStorage",
            "StarterGui",
            "StarterPack",
            "StarterPlayer",
            "Teams",
            "SoundService",
            "TextChatService",
        ];

        let dom = self.dom();
        let reflection = self.reflection();

        VISIBLE_SERVICES
            .iter()
            .filter_map(|name| reflection.class_id(name))
            .filter_map(|class| dom.find_child_of_class(self.world.game, class))
            .collect()
    }

    pub fn execute(&mut self, command: Command) -> anyhow::Result<()> {
        let engine = &self.emulator.engine;
        let reflection = &engine.reflection;
        let hooks = engine.hooks.borrow();

        match command {
            Command::Select(ids) => self.selection = ids,

            Command::Insert { parent, class } => {
                let mut state = self.state_mut();
                let id = ree::instance::new_instance(
                    &mut state,
                    reflection,
                    &hooks,
                    &class,
                    Some(parent),
                )
                .map_err(|err| anyhow::Error::msg(format!("{:?}", err)))?;
                drop(state);
                self.selection = vec![id];
            }

            Command::Rename { id, name } => {
                let name_id = reflection
                    .name_id("Name")
                    .ok_or_else(|| anyhow::anyhow!("no Name property"))?;
                let mut state = self.state_mut();
                ree::instance::raw_set(
                    &mut state,
                    reflection,
                    &hooks,
                    id,
                    name_id,
                    ree::Value::String(name.as_str().into()),
                );
            }

            Command::SetProperty { id, name, value } => {
                let mut state = self.state_mut();
                ree::instance::raw_set(&mut state, reflection, &hooks, id, name, value);
            }

            Command::Delete(ids) => {
                let mut state = self.state_mut();
                for &id in &ids {
                    ree::instance::set_parent(&mut state, reflection, &hooks, id, None)
                        .map_err(|error| anyhow::anyhow!("couldn't delete: {error:?}"))?;
                }
                drop(state);
                self.selection.retain(|id| !ids.contains(id));
            }
        }

        Ok(())
    }
}
