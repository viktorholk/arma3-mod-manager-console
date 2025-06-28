use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::errors::{AppError, AppResult};

use super::utils;

const SAVE_FILE: &str = "arma3-mod-manager-console-config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    game_path: String,
    workshop_path: String,
    custom_mods_path: Option<String>, // Optional
    #[serde(default = "default_executable_name")]
    executable_name: String,
    #[serde(deserialize_with = "deserialize_mods")]
    enabled_mods: Vec<String>,
    default_args: String,
}

// Backwards compatibility supports
// Since previous configs the enabled mods array was only numbers.
fn deserialize_mods<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Vec::<Value>::deserialize(deserializer)?;

    let mut mods = Vec::new();
    for item in v {
        match item {
            Value::String(s) => mods.push(s),
            Value::Number(n) => mods.push(n.to_string()),
            _ => {}
        }
    }
    Ok(mods)
}

fn default_executable_name() -> String {
    "arma3".to_string()
}

impl Config {
    fn get_save_path() -> AppResult<PathBuf> {
        let home_path = utils::get_home_path()?;

        Ok(Path::new(&home_path).join(SAVE_FILE))
    }

    pub fn new(
        game_path: String,
        workshop_path: String,
        custom_mods_path: Option<String>,
    ) -> AppResult<Self> {
        let new_config = Config {
            game_path,
            workshop_path,
            custom_mods_path,
            executable_name: default_executable_name(),
            enabled_mods: Vec::new(),
            default_args: "-noSplash -skipIntro -world=empty".to_string(),
        };

        Ok(new_config)
    }

    fn valid(&self) -> AppResult<()> {
        if !Path::new(&self.workshop_path).exists() {
            return Err(AppError::InvalidPath(self.workshop_path.to_owned()));
        }

        if !Path::new(&self.game_path).exists() {
            return Err(AppError::InvalidPath(self.game_path.to_owned()));
        }

        Ok(())
    }

    pub fn get_enabled_mods(&self) -> Vec<String> {
        self.enabled_mods.clone()
    }

    pub fn update_mods(&mut self, mods: Vec<String>) {
        self.enabled_mods = mods;
    }

    pub fn get_game_path(&self) -> &Path {
        Path::new(&self.game_path)
    }

    pub fn get_workshop_path(&self) -> &Path {
        Path::new(&self.workshop_path)
    }

    pub fn get_custom_mods_path(&self) -> Option<&Path> {
        self.custom_mods_path.as_deref().map(Path::new)
    }

    pub fn get_executable_name(&self) -> &str {
        &self.executable_name
    }

    pub fn set_executable_name(&mut self, name: String) {
        self.executable_name = name;
    }

    pub fn get_default_args(&self) -> &str {
        &self.default_args
    }

    pub fn set_default_args(&mut self, args: String) {
        self.default_args = args;
    }

    pub fn save(&self) -> AppResult<()> {
        super::file_handler::write_json(&Config::get_save_path()?, &self)?;
        Ok(())
    }

    pub fn read() -> AppResult<Self> {
        let config: Config = super::file_handler::read_json(&Config::get_save_path()?)?;

        config.valid()?;

        Ok(config)
    }
}
