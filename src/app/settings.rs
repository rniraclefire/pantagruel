use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Settings {
    pub studio_content: Option<PathBuf>,
    pub offline: bool,
    pub last_place: Option<PathBuf>,
}

impl Settings {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|storage| eframe::get_value(storage, "settings"))
            .unwrap_or_default()
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "settings", self);
    }

    pub fn studio_content(&self) -> Option<PathBuf> {
        self.studio_content
            .clone()
            .or_else(|| std::env::var_os("ROR_ROBLOX_CONTENT").map(PathBuf::from))
            .or_else(|| {
                default_studio_paths()
                    .into_iter()
                    .find(|path| path.is_dir())
            })
    }
}

fn default_studio_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if cfg!(target_os = "macos") {
        paths.push("/Applications/RobloxStudio.app/Contents/Resources/content".into());
        if let Some(home) = std::env::var_os("HOME") {
            paths.push(
                PathBuf::from(home)
                    .join("Applications/RobloxStudio.app/Contents/Resources/content"),
            );
        }
    }
    paths
}
