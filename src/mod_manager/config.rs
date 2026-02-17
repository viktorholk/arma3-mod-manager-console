use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::errors::AppError;
use crate::errors::AppResult;

use super::utils;

fn default_active_preset() -> String {
    "Default".to_string()
}

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
    #[serde(default)]
    presets: HashMap<String, Vec<String>>,
    #[serde(default = "default_active_preset")]
    active_preset: String,
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

fn get_config_path() -> AppResult<PathBuf> {
    let home_path = utils::get_home_path()?;

    // Define OS-specific config paths
    let config_path = match std::env::consts::OS {
        "windows" => Path::new(&home_path).join("arma3-mod-manager-console-config.json"),
        "macos" => Path::new(&home_path)
            .join(".config")
            .join("arma3-mod-manager-console")
            .join("config.json"),
        "linux" => Path::new(&home_path)
            .join(".config")
            .join("arma3-mod-manager-console")
            .join("config.json"),
        _ => return Err(AppError::UnsupportedPlatform),
    };
    return Ok(config_path);
}

impl Config {
    pub fn get_save_path() -> AppResult<PathBuf> {
        let config_path = get_config_path()?;
        Ok(config_path)
    }

    pub fn new(
        game_path: String,
        workshop_path: String,
        custom_mods_path: Option<String>,
    ) -> AppResult<Self> {
        let mut presets = HashMap::new();
        presets.insert("Default".to_string(), Vec::new());

        let new_config = Config {
            game_path,
            workshop_path,
            custom_mods_path,
            executable_name: default_executable_name(),
            enabled_mods: Vec::new(),
            default_args: "-noSplash -skipIntro -world=empty".to_string(),
            presets,
            active_preset: default_active_preset(),
        };

        Ok(new_config)
    }

    /// Migrate old configs that don't have presets yet.
    fn migrate_if_needed(&mut self) {
        if self.presets.is_empty() {
            if self.enabled_mods.is_empty() {
                self.presets.insert("Default".to_string(), Vec::new());
            } else {
                self.presets
                    .insert("Default".to_string(), self.enabled_mods.clone());
            }
        }

        // Ensure active_preset points to an existing preset
        if !self.presets.contains_key(&self.active_preset) {
            self.active_preset = self
                .presets
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "Default".to_string());
        }
    }

    pub fn is_valid(&self) -> bool {
        Path::new(&self.workshop_path).exists() && Path::new(&self.game_path).exists()
    }

    pub fn get_enabled_mods(&self) -> Vec<String> {
        self.presets
            .get(&self.active_preset)
            .cloned()
            .unwrap_or_default()
    }

    pub fn update_mods(&mut self, mods: Vec<String>) {
        self.presets
            .insert(self.active_preset.clone(), mods.clone());
        self.enabled_mods = mods;
    }

    pub fn get_game_path(&self) -> &Path {
        Path::new(&self.game_path)
    }

    pub fn get_workshop_path(&self) -> &Path {
        Path::new(&self.workshop_path)
    }

    pub fn set_game_path(&mut self, path: String) {
        self.game_path = path;
    }

    pub fn set_workshop_path(&mut self, path: String) {
        self.workshop_path = path;
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

    // Preset methods

    pub fn get_active_preset_name(&self) -> &str {
        &self.active_preset
    }

    pub fn get_preset_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.presets.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get_preset_mod_count(&self, name: &str) -> usize {
        self.presets.get(name).map(|v| v.len()).unwrap_or(0)
    }

    pub fn set_active_preset(&mut self, name: &str) {
        if self.presets.contains_key(name) {
            self.active_preset = name.to_string();
            // Keep enabled_mods in sync
            self.enabled_mods = self.presets.get(name).cloned().unwrap_or_default();
        }
    }

    pub fn save_preset(&mut self, name: String, mods: Vec<String>) {
        self.presets.insert(name, mods);
    }

    pub fn rename_preset(&mut self, old: &str, new: String) -> bool {
        if let Some(mods) = self.presets.remove(old) {
            let was_active = self.active_preset == old;
            self.presets.insert(new.clone(), mods);
            if was_active {
                self.active_preset = new;
            }
            true
        } else {
            false
        }
    }

    pub fn delete_preset(&mut self, name: &str) -> bool {
        // Guard against deleting the last preset
        if self.presets.len() <= 1 {
            return false;
        }
        if self.presets.remove(name).is_some() {
            // If we deleted the active preset, switch to another one
            if self.active_preset == name {
                self.active_preset = self
                    .presets
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "Default".to_string());
                self.enabled_mods = self
                    .presets
                    .get(&self.active_preset)
                    .cloned()
                    .unwrap_or_default();
            }
            true
        } else {
            false
        }
    }

    pub fn save(&self) -> AppResult<()> {
        let config_path = &Config::get_save_path()?;
        if let Some(parent) = config_path.parent() {
            utils::ensure_directory_exists(&parent.to_path_buf())?;
        }
        super::file_handler::write_json(config_path, self)?;
        Ok(())
    }

    pub fn read() -> AppResult<Self> {
        let mut config: Config = super::file_handler::read_json(&Config::get_save_path()?)?;
        config.migrate_if_needed();
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a Config for testing without touching the filesystem.
    fn test_config() -> Config {
        Config::new(String::new(), String::new(), None).unwrap()
    }

    // ── Preset CRUD ──

    #[test]
    fn new_config_has_default_preset() {
        let config = test_config();
        assert_eq!(config.get_active_preset_name(), "Default");
        assert_eq!(config.get_preset_names(), vec!["Default"]);
        assert_eq!(config.get_preset_mod_count("Default"), 0);
    }

    #[test]
    fn save_preset_and_read_back() {
        let mut config = test_config();
        config.save_preset("Milsim".to_string(), vec!["mod_a".into(), "mod_b".into()]);

        assert_eq!(config.get_preset_mod_count("Milsim"), 2);
        assert!(config.get_preset_names().contains(&"Milsim".to_string()));
    }

    #[test]
    fn set_active_preset_switches_enabled_mods() {
        let mut config = test_config();
        config.save_preset("A".to_string(), vec!["1".into(), "2".into()]);
        config.save_preset("B".to_string(), vec!["3".into()]);

        config.set_active_preset("B");
        assert_eq!(config.get_active_preset_name(), "B");
        assert_eq!(config.get_enabled_mods(), vec!["3".to_string()]);

        config.set_active_preset("A");
        assert_eq!(config.get_enabled_mods(), vec!["1".to_string(), "2".to_string()]);
    }

    #[test]
    fn set_active_preset_ignores_nonexistent() {
        let mut config = test_config();
        config.set_active_preset("DoesNotExist");
        assert_eq!(config.get_active_preset_name(), "Default");
    }

    #[test]
    fn update_mods_writes_to_active_preset() {
        let mut config = test_config();
        config.update_mods(vec!["x".into(), "y".into()]);

        assert_eq!(config.get_enabled_mods(), vec!["x".to_string(), "y".to_string()]);
        assert_eq!(config.get_preset_mod_count("Default"), 2);
    }

    #[test]
    fn rename_preset_updates_active() {
        let mut config = test_config();
        assert_eq!(config.get_active_preset_name(), "Default");

        let ok = config.rename_preset("Default", "Main".to_string());
        assert!(ok);
        assert_eq!(config.get_active_preset_name(), "Main");
        assert!(!config.get_preset_names().contains(&"Default".to_string()));
        assert!(config.get_preset_names().contains(&"Main".to_string()));
    }

    #[test]
    fn rename_non_active_preset_keeps_active() {
        let mut config = test_config();
        config.save_preset("Other".to_string(), vec![]);

        let ok = config.rename_preset("Other", "Renamed".to_string());
        assert!(ok);
        assert_eq!(config.get_active_preset_name(), "Default");
    }

    #[test]
    fn rename_nonexistent_returns_false() {
        let mut config = test_config();
        assert!(!config.rename_preset("Nope", "X".to_string()));
    }

    #[test]
    fn delete_preset_guards_last() {
        let mut config = test_config();
        assert!(!config.delete_preset("Default"));
        assert_eq!(config.get_preset_names().len(), 1);
    }

    #[test]
    fn delete_preset_switches_active_if_needed() {
        let mut config = test_config();
        config.save_preset("Other".to_string(), vec!["z".into()]);
        config.set_active_preset("Other");

        assert!(config.delete_preset("Other"));
        // Active should have switched to the remaining preset
        assert_eq!(config.get_preset_names().len(), 1);
        assert!(config.get_preset_names().contains(&"Default".to_string()));
        assert_eq!(config.get_active_preset_name(), "Default");
    }

    #[test]
    fn delete_non_active_preset_keeps_active() {
        let mut config = test_config();
        config.save_preset("Extra".to_string(), vec![]);

        assert!(config.delete_preset("Extra"));
        assert_eq!(config.get_active_preset_name(), "Default");
        assert_eq!(config.get_preset_names(), vec!["Default"]);
    }

    #[test]
    fn get_preset_names_sorted() {
        let mut config = test_config();
        config.save_preset("Zebra".to_string(), vec![]);
        config.save_preset("Alpha".to_string(), vec![]);

        let names = config.get_preset_names();
        assert_eq!(names, vec!["Alpha", "Default", "Zebra"]);
    }

    // ── Serde roundtrip ──

    #[test]
    fn deserialize_old_config_no_presets_field() {
        let json = r#"{
            "game_path": "/game",
            "workshop_path": "/workshop",
            "custom_mods_path": null,
            "executable_name": "arma3",
            "enabled_mods": ["111", "222"],
            "default_args": ""
        }"#;

        let mut config: Config = serde_json::from_str(json).unwrap();
        config.migrate_if_needed();

        assert_eq!(config.get_active_preset_name(), "Default");
        assert_eq!(config.get_enabled_mods(), vec!["111".to_string(), "222".to_string()]);
        assert_eq!(config.get_preset_mod_count("Default"), 2);
    }

    #[test]
    fn deserialize_config_with_presets() {
        let json = r#"{
            "game_path": "/game",
            "workshop_path": "/workshop",
            "custom_mods_path": null,
            "executable_name": "arma3",
            "enabled_mods": ["a"],
            "default_args": "",
            "presets": {
                "MyPreset": ["a", "b"],
                "Empty": []
            },
            "active_preset": "MyPreset"
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.get_active_preset_name(), "MyPreset");
        assert_eq!(config.get_enabled_mods(), vec!["a".to_string(), "b".to_string()]);
        assert_eq!(config.get_preset_mod_count("Empty"), 0);
    }

    #[test]
    fn deserialize_numeric_mod_ids_compat() {
        let json = r#"{
            "game_path": "/game",
            "workshop_path": "/workshop",
            "custom_mods_path": null,
            "executable_name": "arma3",
            "enabled_mods": [123, 456, "mixed"],
            "default_args": ""
        }"#;

        let mut config: Config = serde_json::from_str(json).unwrap();
        config.migrate_if_needed();

        assert_eq!(
            config.get_enabled_mods(),
            vec!["123".to_string(), "456".to_string(), "mixed".to_string()]
        );
    }

    #[test]
    fn serialize_then_deserialize_roundtrip() {
        let mut config = test_config();
        config.save_preset("Test".to_string(), vec!["m1".into()]);
        config.set_active_preset("Test");

        let json = serde_json::to_string(&config).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.get_active_preset_name(), "Test");
        assert_eq!(restored.get_enabled_mods(), vec!["m1".to_string()]);
        assert_eq!(restored.get_preset_names().len(), 2);
    }
}
