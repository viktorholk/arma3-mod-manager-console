use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::errors::AppResult;

pub fn write_json<T>(path: &Path, data: T) -> AppResult<()>
where
    T: Serialize,
{
    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(&file);
    serde_json::to_writer_pretty(&mut writer, &data)?;
    writer.flush()?;

    Ok(())
}

pub fn read_json<T>(path: &Path) -> AppResult<T>
where
    T: DeserializeOwned,
{
    let file = fs::File::open(path)?;

    let data = serde_json::from_reader(file)?;

    Ok(data)
}

pub fn remove_dir_symlinks(path: &Path) -> AppResult<()> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let path = entry?.path();
            if path.is_symlink() {
                fs::remove_file(path)?;
            }
        }
    }

    Ok(())
}

pub fn create_sym_links(path: &Path, entries: Vec<PathBuf>) -> AppResult<()> {
    for entry in entries {
        let to_path = path.join(entry.file_name().unwrap());
        if to_path.exists() {
            continue;
        }
        std::os::unix::fs::symlink(entry, to_path)?;
    }
    Ok(())
}
