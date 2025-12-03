use errors::AppResult;
use std::env;

use mod_manager::ModManager;

mod errors;
mod mod_manager;

fn main() -> AppResult<()> {
    let mut manager = ModManager::new(15)?;

    let args: Vec<String> = env::args().collect();
    if args.contains(&"--list".to_string()) {
        println!("Found {} mods:", manager.loaded_mods.all_items().len());
        for mod_item in manager.loaded_mods.all_items() {
            println!("- {}", mod_item.name);
        }
        return Ok(());
    }

    manager.start()?;

    Ok(())
}
