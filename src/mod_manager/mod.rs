use std::{
    fs,
    path::{Path, PathBuf},
};

use phf::phf_map;

use self::{config::Config, paginator::Paginator, terminal::Terminal};

use crate::errors::{AppError, AppResult};

mod config;
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
        path.join(self.identifier.to_string())
    }
}

#[derive(Debug)]
pub struct ModManager {
    config: Config,
    loaded_mods: Paginator<Mod>,
}

impl ModManager {
    pub fn new(page_size: usize) -> AppResult<Self> {
        match Config::read() {
            Ok(config) => {
                let mut loaded_mods = ModManager::get_installed_mods(&config)?;

                for l_mod in &mut loaded_mods {
                    if config.get_enabled_mods().contains(&l_mod.identifier) {
                        l_mod.enabled = true;
                    }
                }

                Ok(ModManager {
                    config,
                    loaded_mods: Paginator::new(loaded_mods, page_size),
                })
            }

            Err(AppError::IoError(io_error)) if io_error.kind() == std::io::ErrorKind::NotFound => {
                let (workshop_path, game_path) = utils::setup_steam_paths()?;
                // Setup the customs mod folder
                let custom_mods_path = utils::construct_path_string(
                    &Path::new(&utils::get_home_path()?),
                    "arma3-mod-manager-console-custom-mods",
                )?;

                if !Path::new(&custom_mods_path).exists() {
                    fs::create_dir(&custom_mods_path)?;
                }

                let config = Config::new(game_path, workshop_path, Some(custom_mods_path))?;
                config.save()?;

                let loaded_mods = ModManager::get_installed_mods(&config)?;

                Ok(ModManager {
                    config,
                    loaded_mods: Paginator::new(loaded_mods, page_size),
                })
            }
            Err(e) => Err(e),
        }
    }

    pub fn start(&mut self) -> AppResult<()> {
        let mut term = Terminal::new(self);

        term.run()?;

        Ok(())
    }

    pub fn refresh_mods(&mut self) -> AppResult<()> {
        let installed_mods = ModManager::get_installed_mods(&self.config)?;
        self.loaded_mods = Paginator::new(installed_mods, self.loaded_mods.page_size);

        Ok(())
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
