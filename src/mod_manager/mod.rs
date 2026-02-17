use std::path::{Path, PathBuf};

use phf::phf_map;

use self::{config::Config, paginator::Paginator, terminal::Terminal};

use crate::errors::{AppError, AppResult};

mod config;
pub mod dependency_manager;
mod file_handler;
mod paginator;
mod terminal;
mod utils;

/// Arma 3 Creator DLCs
/// Unlike base game DLCs - CDLCS needs to be included in the startup arguments
static ARMA3_CDLCS: phf::Map<&'static str, &'static str> = phf_map! {
    "GM" => "Global Mobilization",
    "VN" => "S.O.G. Prairie Fire",
    "CSLA" => "CSLA Iron Curtain",
    "WS" => "Western Sahara",
    "SPE" => "Spearhead 1944",
    "RF" => "Reaction Forces",
    "EF" => "Expeditionary Forces",
};

#[derive(Debug, Clone)]
pub struct Mod {
    pub identifier: String,
    pub name: String,
    pub enabled: bool,
    pub is_cdlc: bool,
    pub is_custom: bool,
}

impl Mod {
    fn new(identifier: String, name: String, is_cdlc: bool, is_custom: bool) -> Mod {
        Mod {
            identifier,
            name,
            enabled: false,
            is_cdlc,
            is_custom,
        }
    }

    pub fn get_path(&self, path: &Path) -> PathBuf {
        path.join(&self.identifier)
    }
}

#[derive(Debug)]
pub struct ModManager {
    pub config: Config,
    pub loaded_mods: Paginator<Mod>,
}

impl ModManager {
    pub fn new(page_size: usize) -> AppResult<Self> {
        // Try to read config. If it fails (NotFound), create a default empty one.
        let config = match Config::read() {
            Ok(c) => c,
            Err(AppError::IoError(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                // Create a default config (empty paths) to start with.
                // We don't save it yet; the Wizard will handle that.
                Config::new(String::new(), String::new(), None)?
            }
            Err(e) => return Err(e),
        };

        // Attempt to load mods if config is somewhat valid, otherwise just start with empty list
        let loaded_mods_vec = if config.is_valid() {
            match ModManager::get_installed_mods(&config) {
                Ok(mut mods) => {
                     for l_mod in &mut mods {
                        if config.get_enabled_mods().contains(&l_mod.identifier) {
                            l_mod.enabled = true;
                        }
                    }
                    mods
                },
                Err(_) => Vec::new(), // If path reading fails, just return empty list
            }
        } else {
            Vec::new()
        };

        Ok(ModManager {
            config,
            loaded_mods: Paginator::new(loaded_mods_vec, page_size),
        })
    }

    pub fn start(&mut self) -> AppResult<()> {
        let mut term = Terminal::new(self);

        term.run()?;

        Ok(())
    }

    pub fn refresh_mods(&mut self) -> AppResult<()> {
        let installed_mods = ModManager::get_installed_mods(&self.config)?;
        self.loaded_mods = Paginator::new(installed_mods, self.loaded_mods.page_size);
        self.apply_active_preset();

        Ok(())
    }

    /// Sets `mod.enabled` for all loaded mods based on the active preset.
    pub fn apply_active_preset(&mut self) {
        let enabled = self.config.get_enabled_mods();
        for m in self.loaded_mods.all_items_mut() {
            m.enabled = enabled.contains(&m.identifier);
        }
    }

    /// Switches to the given preset and applies it.
    pub fn switch_preset(&mut self, name: &str) {
        self.config.set_active_preset(name);
        self.apply_active_preset();
    }

    fn get_installed_mods(config: &Config) -> AppResult<Vec<Mod>> {
        let mut mods: Vec<Mod> = Vec::new();

        // Process workshop mods
        if let Ok(paths) = utils::yield_path_dirs(config.get_workshop_path()) {
            mods.extend(
                paths
                    .into_iter()
                    .filter_map(|path_buf| utils::process_mod_dir(path_buf, false)),
            );
        }

        // Process custom mods folder
        if let Some(custom_mods_path) = config.get_custom_mods_path() {
            if let Ok(paths) = utils::yield_path_dirs(custom_mods_path) {
                mods.extend(
                    paths
                        .into_iter()
                        .filter_map(|path_buf| utils::process_mod_dir(path_buf, true)),
                );
            }
        }

        // Process CDLCS
        if let Ok(paths) = utils::yield_path_dirs(config.get_game_path()) {
            mods.extend(paths.into_iter().filter_map(|path_buf| {
                let dir_name = path_buf.file_name()?.to_str()?.to_string();
                ARMA3_CDLCS
                    .get_entry(&dir_name)
                    .map(|(key, value)| Mod::new(key.to_string(), value.to_string(), true, false))
            }));
        }

        mods.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(mods)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_mod_manager_full_flow() {
        // Setup paths
        let mut fake_home = env::current_dir().unwrap();
        fake_home.push("fake_home_test"); // Changed name to avoid conflict/confusion

        let workshop_path =
            fake_home.join("Library/Application Support/Steam/steamapps/workshop/content/107410");
        let game_path = fake_home.join("Library/Application Support/Steam/steamapps/common/Arma 3");
        let mod_dir = workshop_path.join("123456");
        let meta_file = mod_dir.join("meta.cpp");

        // Cleanup previous run if exists
        if fake_home.exists() {
            let _ = fs::remove_dir_all(&fake_home);
        }

        // Create directories
        fs::create_dir_all(&mod_dir).expect("Failed to create mod directory");
        fs::create_dir_all(&game_path).expect("Failed to create game directory");

        // Create fake mod meta.cpp
        fs::write(&meta_file, "name = \"Test Mod\";").expect("Failed to write meta.cpp");

        // Set the HOME env var for this test
        env::set_var("HOME", &fake_home);

        // Initialize ModManager
        let manager = ModManager::new(10).expect("Failed to initialize ModManager");

        // Verify Config was created
        let config_path = fake_home.join("arma3-mod-manager-console-config.json");
        assert!(config_path.exists(), "Config file was not created");

        // Verify "Test Mod" was loaded
        let mods = manager.loaded_mods.all_items();
        let found_mod = mods.iter().find(|m| m.name == "Test Mod");

        assert!(
            found_mod.is_some(),
            "Test Mod was not found in loaded mods. Found: {:?}",
            mods.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
        let found_mod = found_mod.unwrap();

        // Test Symlink Creation
        // We must fetch paths from config to ensure it picked up the right ones
        let config_workshop_path = manager.config.get_workshop_path();
        let config_game_path = manager.config.get_game_path();

        let mod_path = found_mod.get_path(config_workshop_path);
        let mod_paths = vec![mod_path];

        // Create symlinks
        file_handler::create_sym_links(config_game_path, mod_paths)
            .expect("Failed to create symlinks");

        // Verify symlink exists
        let symlink_path = config_game_path.join(&found_mod.identifier);
        assert!(
            symlink_path.exists(),
            "Symlink was not created at {:?}",
            symlink_path
        );
        assert!(symlink_path.is_symlink(), "Created file is not a symlink");

        // Test Symlink Removal
        file_handler::remove_dir_symlinks(config_game_path).expect("Failed to remove symlinks");

        // Verify symlink is gone
        assert!(!symlink_path.exists(), "Symlink was not removed");

        // Cleanup
        let _ = fs::remove_dir_all(&fake_home);
    }
}
