use std::path::Path;

pub mod exports;
pub mod properties;

pub struct Place {
    pub emulator: ree::Emulator,
    pub world: ree::World,
    pub selection: Vec<ree::InstanceId>,
}

pub type DomVersion = (u64, u64, u32);

impl Place {
    pub fn open(path: &Path, options: ree::Options) -> anyhow::Result<Self> {
        let engine = ree::Engine::new(options).map_err(anyhow::Error::msg)?;
        let emulator = ree::Emulator::start(engine, &ree::mount::ProjectSpec::default())
            .map_err(anyhow::Error::msg)?;
        let world = emulator.engine.world();
        ree::project::load_snapshot(&emulator.engine, &path, world.game)
            .map_err(anyhow::Error::msg)?;

        return Ok(Self {
            emulator,
            world,
            selection: vec![],
        });
    }

    pub fn dom_version(&self) -> DomVersion {
        let state = self.emulator.engine.state.borrow();
        (
            state.dom.structure_version,
            state.dom.write_version,
            state.dom.name_version,
        )
    }
}
