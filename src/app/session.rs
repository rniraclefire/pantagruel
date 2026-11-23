use std::path::Path;

use crate::{
    app::dock::{Dock, default_dock},
    place::Place,
};

pub struct Session {
    pub place: Place,
    pub dock: Dock,
}

impl Session {
    pub fn open(path: &Path, options: ree::Options) -> anyhow::Result<Self> {
        return Ok(Self {
            place: Place::open(path, options)?,
            dock: default_dock(),
        });
    }
}
