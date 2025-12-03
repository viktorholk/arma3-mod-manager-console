use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::errors::{AppError, AppResult};

use super::Mod;

pub fn get_home_path() -> AppResult<OsString> {
    match env::var_os("HOME") {
        Some(home_path) => Ok(home_path),
        None => Err(AppError::InvalidHomePath),
    }
}

pub fn setup_steam_paths() -> AppResult<(String, String)> {
    let home_path = get_home_path()?;

    // Define OS-specific base paths
    let base_path = match std::env::consts::OS {
        "macos" => Path::new(&home_path).join("Library/Application Support"),
        "linux" => Path::new(&home_path).join(".local/share"),
        _ => return Err(AppError::UnsupportedPlatform),
    };

    // Define relative paths
    let steam_workshop_path = "Steam/steamapps/workshop/content/107410";
    let steam_game_path = "Steam/steamapps/common/Arma 3";

    // Construct full paths
    let workshop_path = construct_path_string(&base_path, steam_workshop_path)?;
    let game_path = construct_path_string(&base_path, steam_game_path)?;

    Ok((workshop_path, game_path))
}

pub fn construct_path_string(base_path: &Path, relative_path: &str) -> AppResult<String> {
    let full_path = base_path.join(relative_path);
    full_path
        .to_str()
        .ok_or_else(|| AppError::PathConversionError(full_path.to_string_lossy().into()))
        .map(|s| s.to_string())
}

pub fn yield_path_dirs(path: &Path) -> AppResult<impl Iterator<Item = PathBuf>> {
    let dirs = fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir());

    Ok(dirs)
}

pub fn process_mod_dir(path_buf: PathBuf, is_custom: bool) -> Option<Mod> {
    let file_name = path_buf.file_name()?.to_str()?.to_string();

    // Ensure "meta.cpp" exists
    let meta_file = path_buf.join("meta.cpp");
    let meta_content = fs::read(&meta_file).ok()?;

    // Convert file content to a UTF-8 string
    let content_str = String::from_utf8_lossy(&meta_content);

    // Extract the `name` using regex
    // We look into the meta.cpp file
    // for Workshop mods it is common for the name to be there
    // Both for custom mods its not always the case, so just fallback to the file_name instead
    let name = match Regex::new(r#"name\s*=\s*"([^"]+)""#)
        .ok()?
        .captures(&content_str)
        .and_then(|caps| caps.get(1))
        .map(|m| titleize(m.as_str()))
    {
        Some(name) => name,
        None => titleize(&file_name),
    };

    Some(Mod::new(file_name, name, false, is_custom))
}

fn titleize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

#[cfg(target_os = "macos")]
pub fn get_steam_overlay_path() -> Option<PathBuf> {
    let home_path = get_home_path().ok()?;

    let paths = vec![
        // Default location for Steam on macOS
        Path::new(&home_path).join(
            "Library/Application Support/Steam/Steam.AppBundle/Steam/Contents/MacOS/gameoverlayrenderer.dylib",
        ),
        // Alternative location
        Path::new(&home_path).join(
            "Library/Application Support/Steam/Contents/MacOS/gameoverlayrenderer.dylib",
        ),
        // Global application location
        PathBuf::from("/Applications/Steam.app/Contents/MacOS/gameoverlayrenderer.dylib"),
    ];

    paths.into_iter().find(|path| path.exists())
}
