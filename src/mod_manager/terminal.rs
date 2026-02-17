use std::{
    io::{self, Stdout, Write},
    path::PathBuf,
    process::Command,
    time::Duration,
};

use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{self, poll, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal,
};

use crate::{
    errors::{AppError, AppResult},
    mod_manager::config::Config,
};

use super::{dependency_manager, ModManager};

pub struct Terminal<'a> {
    mod_manager: &'a mut ModManager,
    selected_index: usize,
}

impl<'a> Terminal<'a> {
    pub fn new(mod_manager: &'a mut ModManager) -> Self {
        Terminal {
            mod_manager,
            selected_index: 0,
        }
    }

    /// Get the executable path based on the current platform
    #[cfg(target_os = "macos")]
    fn get_executable_path(game_path: &std::path::Path, executable_name: &str) -> PathBuf {
        let game_app_path = game_path.join(format!("{}.app", executable_name));
        game_app_path.join("Contents/MacOS").join(executable_name)
    }

    #[cfg(target_os = "linux")]
    fn get_executable_path(game_path: &std::path::Path, executable_name: &str) -> PathBuf {
        game_path.join(executable_name)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn get_executable_path(game_path: &std::path::Path, executable_name: &str) -> PathBuf {
        // Fallback for other platforms - assume direct executable
        game_path.join(executable_name)
    }

    pub fn run(&mut self) -> AppResult<()> {
        let mut stdout = io::stdout();

        execute!(stdout, cursor::SavePosition)?;
        execute!(stdout, terminal::EnterAlternateScreen)?;
        execute!(stdout, crossterm::cursor::Hide)?;

        terminal::enable_raw_mode()?;

        if !self.mod_manager.config.is_valid() {
            self.run_setup_wizard(&mut stdout)?;
        } else {
            self.main_loop(&mut stdout)?;
        }

        terminal::disable_raw_mode()?;

        execute!(stdout, terminal::LeaveAlternateScreen)?;
        execute!(stdout, cursor::RestorePosition)?;
        execute!(stdout, crossterm::cursor::Show)?;

        Ok(())
    }

    fn run_setup_wizard(&mut self, stdout: &mut Stdout) -> AppResult<()> {
        // Try to auto-detect defaults
        let (default_workshop, default_game) = match super::utils::setup_steam_paths() {
            Ok((w, g)) => (w, g),
            Err(_) => (String::new(), String::new()),
        };

        let mut workshop_path = self
            .mod_manager
            .config
            .get_workshop_path()
            .to_string_lossy()
            .to_string();
        let mut game_path = self
            .mod_manager
            .config
            .get_game_path()
            .to_string_lossy()
            .to_string();

        // If current config is empty, populate with defaults (auto-detected)
        if workshop_path.is_empty() {
            workshop_path = default_workshop;
        }
        if game_path.is_empty() {
            game_path = default_game;
        }

        loop {
            self.clear_screen(stdout)?;

            execute!(
                stdout,
                cursor::MoveTo(0, 0),
                SetForegroundColor(Color::Cyan),
                Print("Arma 3 Mod Manager - First Time Setup"),
                SetForegroundColor(Color::Reset),
                cursor::MoveTo(0, 2),
                Print("It seems your configuration is invalid or missing."),
                cursor::MoveTo(0, 3),
                Print("Please verify your Steam paths below."),
                cursor::MoveTo(0, 5),
                Print(format!("1. Workshop Path: {}", workshop_path)),
                cursor::MoveTo(0, 6),
                Print(format!("2. Game Path:     {}", game_path)),
                cursor::MoveTo(0, 8),
                Print("Press <1> to edit Workshop Path"),
                cursor::MoveTo(0, 9),
                Print("Press <2> to edit Game Path"),
                cursor::MoveTo(0, 11),
                Print(format!(
                    "Press <ENTER> to Save ({}) and Continue",
                    match Config::get_save_path() {
                        Ok(path) => path.display().to_string(),
                        Err(e) => format!("Error: {}", e),
                    }
                )),
                cursor::MoveTo(0, 12),
                Print("Press <ESC> or <Q> to Quit"),
            )?;
            stdout.flush()?;

            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Char('1') => {
                            workshop_path = self.input_screen(
                                stdout,
                                "Edit Workshop Path",
                                "Enter Path:",
                                &workshop_path,
                            )?;
                        }
                        KeyCode::Char('2') => {
                            game_path = self.input_screen(
                                stdout,
                                "Edit Game Path",
                                "Enter Path:",
                                &game_path,
                            )?;
                        }
                        KeyCode::Enter => {
                            self.mod_manager
                                .config
                                .set_workshop_path(workshop_path.clone());
                            self.mod_manager.config.set_game_path(game_path.clone());

                            if self.mod_manager.config.is_valid() {
                                self.mod_manager.config.save()?;
                                self.mod_manager.refresh_mods()?;
                                break;
                            } else {
                                self.clear_screen(stdout)?;
                                execute!(
                                    stdout,
                                    cursor::MoveTo(0, 0),
                                    SetForegroundColor(Color::Red),
                                    Print("Error: Paths are invalid! check if directories exist."),
                                    SetForegroundColor(Color::Reset),
                                    cursor::MoveTo(0, 2),
                                    Print("Press any key to try again...")
                                )?;
                                stdout.flush()?;
                                loop {
                                    if event::poll(Duration::from_millis(500))? {
                                        if let Event::Key(_) = event::read()? {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            return Ok(()); // Exit app essentially, or return to empty loop which exits
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn input_screen(
        &self,
        stdout: &mut Stdout,
        title: &str,
        prompt: &str,
        initial_value: &str,
    ) -> AppResult<String> {
        let mut input_string = initial_value.to_string();
        let mut current_pos = input_string.len() as u16;

        // Set up the terminal
        execute!(stdout, cursor::Show)?;
        execute!(stdout, SetCursorStyle::BlinkingUnderScore)?;
        stdout.flush()?;

        self.clear_screen(stdout)?;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            cursor::MoveTo(0, 0),
            Print(title),
            SetForegroundColor(Color::Reset),
        )?;

        execute!(
            stdout,
            cursor::MoveTo(0, 2),
            Print("Press <ENTER> to confirm, <ESC> to cancel"),
        )?;

        let prompt_left = 4;
        let prompt_top = 4;
        let prompt_left_padding = prompt_left - 3;

        execute!(
            stdout,
            SetForegroundColor(Color::Red),
            cursor::MoveTo(prompt_left_padding, prompt_top),
            Print(">"),
            SetForegroundColor(Color::Reset)
        )?;

        execute!(
            stdout,
            cursor::MoveTo(prompt_left, prompt_top),
            Print(format!("{} ", prompt))
        )?;

        let input_start_col = prompt_left + prompt.len() as u16 + 1;

        // Initial render
        execute!(
            stdout,
            cursor::MoveTo(input_start_col, prompt_top),
            Print(&input_string)
        )?;

        execute!(
            stdout,
            cursor::MoveTo(input_start_col + current_pos, prompt_top)
        )?;

        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Esc => {
                            // Restore terminal state before returning
                            execute!(stdout, cursor::Hide)?;
                            execute!(stdout, SetCursorStyle::DefaultUserShape)?;
                            return Ok(initial_value.to_string());
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        KeyCode::Backspace => {
                            if !input_string.is_empty() && current_pos > 0 {
                                input_string.pop();
                                current_pos -= 1;
                            }
                        }
                        KeyCode::Char(c) => {
                            input_string.push(c);
                            current_pos += 1;
                        }
                        _ => {}
                    }

                    execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;

                    execute!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        cursor::MoveTo(prompt_left_padding, prompt_top),
                        Print(">"),
                        SetForegroundColor(Color::Reset)
                    )?;

                    execute!(
                        stdout,
                        cursor::MoveTo(prompt_left, prompt_top),
                        Print(format!("{} ", prompt))
                    )?;

                    execute!(
                        stdout,
                        cursor::MoveTo(input_start_col, prompt_top),
                        Print(&input_string)
                    )?;

                    // Move cursor to the new position
                    execute!(
                        stdout,
                        cursor::MoveTo(input_start_col + current_pos, prompt_top)
                    )?;

                    stdout.flush()?;
                }
            }
        }
        // Restore terminal state
        execute!(stdout, cursor::Hide)?;
        execute!(stdout, SetCursorStyle::DefaultUserShape)?;

        Ok(input_string)
    }

    fn render(&self, stdout: &mut Stdout) -> AppResult<()> {
        self.clear_screen(stdout)?;

        let mut top_offset = 0;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            cursor::MoveTo(0, top_offset),
            Print(format!(
                "Arma 3 Mod Manager Console ({})",
                env!("CARGO_PKG_VERSION")
            )),
            cursor::MoveTo(0, top_offset + 1),
            Print(format!(
                "Config file: {}",
                match Config::get_save_path() {
                    Ok(path) => path.display().to_string(),
                    Err(e) => format!("Error: {}", e),
                }
            )),
            SetForegroundColor(Color::Reset)
        )?;

        top_offset += 3;

        let enabled_mods = self.mod_manager.loaded_mods.filter(|m| m.enabled).len();
        let total_mods = self.mod_manager.loaded_mods.all_items().len();

        let page_number = self.mod_manager.loaded_mods.current_page + 1;
        let total_pages = self.mod_manager.loaded_mods.total_pages();

        execute!(
            stdout,
            cursor::MoveTo(0, top_offset),
            Print(&format!(
                "Mods: {:<2}/{:<2}{:^25}Page: {:<2}/{:<2}",
                enabled_mods, total_mods, " ", page_number, total_pages
            )),
        )?;

        top_offset += 2;

        for (i, m) in self
            .mod_manager
            .loaded_mods
            .current_page_items()
            .iter()
            .enumerate()
        {
            let mut str: String = String::new();

            let cursor = if i == self.selected_index {
                " > "
            } else {
                "   "
            };

            execute!(
                stdout,
                cursor::MoveTo(0, top_offset),
                SetForegroundColor(Color::Red),
                Print(cursor),
                SetForegroundColor(Color::Reset)
            )?;

            let mut color = Color::Grey;

            if m.enabled {
                color = Color::White;
                str += "[X]";
            } else {
                str += "[ ]";
            }

            str += &format!(" {}", m.name);

            str.truncate(36);

            execute!(
                stdout,
                cursor::MoveTo(3, top_offset),
                SetForegroundColor(color),
                Print(str),
                SetForegroundColor(Color::Reset)
            )?;

            if m.is_cdlc {
                execute!(
                    stdout,
                    cursor::MoveTo(41, top_offset),
                    SetForegroundColor(Color::Blue),
                    Print("CDLC"),
                    SetForegroundColor(Color::Reset)
                )?;
            }

            top_offset += 1;
        }

        // Show pagination direction
        if (page_number < total_pages) && (page_number > 1) {
            execute!(
                stdout,
                cursor::MoveTo(0, top_offset),
                Print(&format!("{}{:^38}{}", "<--", "", "-->")),
            )?;
        } else if page_number < total_pages {
            execute!(
                stdout,
                cursor::MoveTo(0, top_offset),
                Print(&format!("{}{:^38}{}", "   ", "", "-->")),
            )?;
        } else if page_number > 1 {
            execute!(
                stdout,
                cursor::MoveTo(0, top_offset),
                Print(&format!("{}{:^38}{}", "<--", "", "   ")),
            )?;
        }

        top_offset = 2;
        let info_left_offset = 50;
        let info_text_padding = 25;

        execute!(
            stdout,
            cursor::MoveTo(info_left_offset, top_offset),
            Print(&format!(
                "{:<padding$}{}",
                "Action",
                "Keybindings",
                padding = info_text_padding
            )),
        )?;

        let actions_keybindings = vec![
            ("Navigation", "<WASD>, <HJKL> or <ARROW KEYS>"),
            ("Toggle Selected Mod", "<SPACE>"),
            ("Toggle All Mods", "<CTRL> + <SPACE>"),
            ("Check Dependencies", "C"),
            ("Refresh Mods", "R"),
            ("Set Custom Parameters", "F"),
            ("Set Executable Name", "E"),
            ("Save Config", "<ENTER>"),
            ("Launch Game", "P"),
        ];

        for (i, (action, keybinding)) in actions_keybindings.iter().enumerate() {
            let y_offset = top_offset + 2 + i as u16; // Adjust starting y offset as needed

            execute!(
                stdout,
                cursor::MoveTo(info_left_offset, y_offset),
                SetForegroundColor(Color::Cyan),
                Print(&format!(
                    "{:<padding$}{}",
                    action,
                    keybinding,
                    padding = info_text_padding
                )),
                SetForegroundColor(Color::Reset),
            )?;
        }

        stdout.flush()?;

        Ok(())
    }

    fn main_loop(&mut self, stdout: &mut Stdout) -> AppResult<()> {
        self.render(stdout)?;
        stdout.flush()?;

        loop {
            if poll(Duration::from_millis(1000))? {
                match event::read()? {
                    Event::Key(event) => match event.code {
                        KeyCode::Char('w') | KeyCode::Char('k') | KeyCode::Up => {
                            if self.selected_index > 0 {
                                self.selected_index -= 1;
                            }
                        }
                        KeyCode::Char('s') | KeyCode::Char('j') | KeyCode::Down => {
                            let length = self.mod_manager.loaded_mods.current_page_items().len();

                            if self.selected_index < length - 1 {
                                self.selected_index += 1;
                            }
                        }

                        KeyCode::Char('a') | KeyCode::Char('h') | KeyCode::Left => {
                            self.mod_manager.loaded_mods.prev_page();
                            self.selected_index = 0;
                        }

                        KeyCode::Char('d') | KeyCode::Char('l') | KeyCode::Right => {
                            self.mod_manager.loaded_mods.next_page();
                            self.selected_index = 0;
                        }

                        KeyCode::Char(' ') if event.modifiers == KeyModifiers::CONTROL => {
                            let value = !self
                                .mod_manager
                                .loaded_mods
                                .all_items()
                                .iter()
                                .all(|m| m.enabled);

                            self.mod_manager
                                .loaded_mods
                                .all_items_mut()
                                .iter_mut()
                                .for_each(|m| m.enabled = value);
                        }

                        KeyCode::Char(' ') => {
                            let current_page = self.mod_manager.loaded_mods.current_page;
                            let page_size = self.mod_manager.loaded_mods.page_size;
                            let index = self.selected_index + (current_page * page_size);

                            let selected_mod =
                                &mut self.mod_manager.loaded_mods.all_items_mut()[index];
                            selected_mod.enabled = !selected_mod.enabled;
                        }

                        KeyCode::Char('r') => {
                            self.mod_manager.refresh_mods()?;
                        }
                        KeyCode::Char('c') => {
                            self.check_dependencies_screen(stdout)?;
                        }
                        KeyCode::Char('f') => {
                            self.set_custom_parameters_screen(stdout)?;
                        }
                        KeyCode::Char('e') => {
                            self.set_executable_name_screen(stdout)?;
                        }
                        KeyCode::Char('p') => {
                            self.start_game()?;
                        }

                        KeyCode::Enter => {
                            let enabled_mods = self.mod_manager.loaded_mods.filter(|m| m.enabled);
                            self.mod_manager.config.update_mods(
                                enabled_mods.iter().map(|m| m.identifier.clone()).collect(),
                            );
                            self.mod_manager.config.save()?;
                        }

                        KeyCode::Esc | KeyCode::Char('q') => break,

                        _ => continue,
                    },

                    _ => continue,
                }
                self.render(stdout)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    fn start_game(&mut self) -> AppResult<()> {
        let enabled_mods = self.mod_manager.loaded_mods.filter(|m| m.enabled);
        let game_path = self.mod_manager.config.get_game_path();
        let workshop_path = self.mod_manager.config.get_workshop_path();
        let custom_mods_path = self.mod_manager.config.get_custom_mods_path();

        let executable_name = self.mod_manager.config.get_executable_name();
        let executable_path = Self::get_executable_path(game_path, executable_name);
        let executable_path_str = executable_path.to_string_lossy().to_string();

        if !executable_path.exists() {
            return Err(AppError::InvalidPath(executable_path_str.to_owned()));
        }

        let mut command = Command::new(&executable_path_str);
        command.current_dir(game_path);

        // Remove existing symlinks from the game directory
        super::file_handler::remove_dir_symlinks(game_path)?;

        if !enabled_mods.is_empty() {
            // Exclude CDLCS when creating sym links since they already are in the game folder
            // only for workshop + custom mods
            let mod_paths: Vec<_> = enabled_mods
                .iter()
                .filter_map(|m| {
                    if m.is_cdlc {
                        None
                    } else if m.is_custom {
                        custom_mods_path.map(|cmp| m.get_path(cmp))
                    } else {
                        Some(m.get_path(workshop_path))
                    }
                })
                .collect();

            super::file_handler::create_sym_links(game_path, mod_paths)?;

            // Save the enabled mods so it loads next time
            self.mod_manager
                .config
                .update_mods(enabled_mods.iter().map(|m| m.identifier.clone()).collect());
            self.mod_manager.config.save()?;

            // Build args
            let default_args = self.mod_manager.config.get_default_args();
            if !default_args.is_empty() {
                command.arg(default_args);
            }

            let mod_list = enabled_mods
                .iter()
                .map(|m| m.identifier.as_str())
                .collect::<Vec<_>>()
                .join(";");

            command.arg(format!("-mod={}", mod_list));
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(overlay_path) = super::utils::get_steam_overlay_path() {
                command.env("DYLD_INSERT_LIBRARIES", overlay_path);
                command.env("DYLD_FORCE_FLAT_NAMESPACE", "1");
            }
            command.env("SteamAppId", "107410");
        }

        command.spawn()?;

        Ok(())
    }

    fn clear_screen(&self, stdout: &mut Stdout) -> AppResult<()> {
        execute!(stdout, cursor::MoveTo(0, 0))?;
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        Ok(())
    }

    fn set_custom_parameters_screen(&mut self, stdout: &mut Stdout) -> AppResult<()> {
        let mut args_string = self.mod_manager.config.get_default_args().to_string();
        let mut current_pos = args_string.len() as u16;

        // Set up the terminal
        execute!(stdout, cursor::Show)?;
        execute!(stdout, SetCursorStyle::BlinkingUnderScore)?;
        stdout.flush()?;

        self.clear_screen(stdout)?;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            cursor::MoveTo(0, 0),
            Print("Arma 3 Mod Manager Console"),
            SetForegroundColor(Color::Reset),
        )?;

        execute!(stdout, cursor::MoveTo(0, 2), Print("Press <ENTER> to save"),)?;

        let arg_string_left = 4;
        let arg_string_top = 4;
        let arg_string_left_padding = arg_string_left - 3;

        execute!(
            stdout,
            SetForegroundColor(Color::Red),
            cursor::MoveTo(arg_string_left_padding, arg_string_top),
            Print(">"),
            SetForegroundColor(Color::Reset)
        )?;

        execute!(
            stdout,
            cursor::MoveTo(arg_string_left, arg_string_top),
            Print(&args_string)
        )?;

        execute!(
            stdout,
            cursor::MoveTo(0, arg_string_top + 2),
            Print("For more information visit: https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters")
        )?;

        execute!(
            stdout,
            cursor::MoveTo(current_pos + arg_string_left, arg_string_top)
        )?;

        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Enter => {
                            self.mod_manager.config.set_default_args(args_string);
                            self.mod_manager.config.save()?;

                            break;
                        }
                        KeyCode::Backspace => {
                            if !args_string.is_empty() && current_pos > 0 {
                                args_string.pop();
                                current_pos -= 1;
                            }
                        }
                        KeyCode::Char(c) => {
                            args_string.push(c);
                            current_pos += 1;
                        }
                        _ => {}
                    }

                    execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;

                    execute!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        cursor::MoveTo(arg_string_left_padding, arg_string_top),
                        Print(">"),
                        SetForegroundColor(Color::Reset)
                    )?;

                    // Clear the previous line and update display
                    execute!(
                        stdout,
                        cursor::MoveTo(arg_string_left, arg_string_top),
                        Print(&args_string)
                    )?;

                    execute!(
                        stdout,
                        cursor::MoveTo(0, arg_string_top + 2),
                        Print("For more information visit: https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters")
                    )?;

                    // Move cursor to the new position
                    execute!(
                        stdout,
                        cursor::MoveTo(current_pos + arg_string_left, arg_string_top)
                    )?;

                    stdout.flush()?;
                }
            }
        }
        // Restore terminal state
        execute!(stdout, cursor::Hide)?;
        execute!(stdout, SetCursorStyle::DefaultUserShape)?;

        Ok(())
    }

    fn set_executable_name_screen(&mut self, stdout: &mut Stdout) -> AppResult<()> {
        let mut executable_name = self.mod_manager.config.get_executable_name().to_string();
        let mut current_pos = executable_name.len() as u16;

        // Set up the terminal
        execute!(stdout, cursor::Show)?;
        execute!(stdout, SetCursorStyle::BlinkingUnderScore)?;
        stdout.flush()?;

        self.clear_screen(stdout)?;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            cursor::MoveTo(0, 0),
            Print("Arma 3 Mod Manager Console - Executable Name"),
            SetForegroundColor(Color::Reset),
        )?;

        execute!(
            stdout,
            cursor::MoveTo(0, 2),
            Print("Press <ENTER> to save, <ESC> to cancel"),
        )?;

        let name_left = 4;
        let name_top = 4;
        let name_left_padding = name_left - 3;

        execute!(
            stdout,
            SetForegroundColor(Color::Red),
            cursor::MoveTo(name_left_padding, name_top),
            Print(">"),
            SetForegroundColor(Color::Reset)
        )?;

        execute!(
            stdout,
            cursor::MoveTo(name_left, name_top),
            Print(&executable_name)
        )?;

        let instruction_text = if cfg!(target_os = "macos") {
            "Enter the name of the Arma 3 executable (without .app extension)"
        } else {
            "Enter the name of the Arma 3 executable"
        };

        execute!(
            stdout,
            cursor::MoveTo(0, name_top + 2),
            Print(instruction_text)
        )?;

        execute!(stdout, cursor::MoveTo(current_pos + name_left, name_top))?;

        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Enter => {
                            self.mod_manager.config.set_executable_name(executable_name);
                            self.mod_manager.config.save()?;
                            break;
                        }
                        KeyCode::Backspace => {
                            if !executable_name.is_empty() && current_pos > 0 {
                                executable_name.pop();
                                current_pos -= 1;
                            }
                        }
                        KeyCode::Char(c) => {
                            executable_name.push(c);
                            current_pos += 1;
                        }
                        _ => {}
                    }

                    execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;

                    execute!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        cursor::MoveTo(name_left_padding, name_top),
                        Print(">"),
                        SetForegroundColor(Color::Reset)
                    )?;

                    // Clear the previous line and update display
                    execute!(
                        stdout,
                        cursor::MoveTo(name_left, name_top),
                        Print(&executable_name)
                    )?;

                    execute!(
                        stdout,
                        cursor::MoveTo(0, name_top + 2),
                        Print(instruction_text)
                    )?;

                    // Move cursor to the new position
                    execute!(stdout, cursor::MoveTo(current_pos + name_left, name_top))?;

                    stdout.flush()?;
                }
            }
        }
        // Restore terminal state
        execute!(stdout, cursor::Hide)?;
        execute!(stdout, SetCursorStyle::DefaultUserShape)?;

        Ok(())
    }

    fn check_dependencies_screen(&mut self, stdout: &mut Stdout) -> AppResult<()> {
        let current_page = self.mod_manager.loaded_mods.current_page;
        let page_size = self.mod_manager.loaded_mods.page_size;
        let index = self.selected_index + (current_page * page_size);
        let selected_mod = &self.mod_manager.loaded_mods.all_items()[index];

        if selected_mod.is_custom || selected_mod.is_cdlc {
            // Show message that we can't check dependencies for custom/CDLC mods
            self.clear_screen(stdout)?;
            execute!(
                stdout,
                cursor::MoveTo(0, 0),
                SetForegroundColor(Color::Yellow),
                Print("Cannot check dependencies for Local/CDLC mods."),
                SetForegroundColor(Color::Reset),
                cursor::MoveTo(0, 2),
                Print("Press any key to return...")
            )?;
            stdout.flush()?;

            loop {
                if event::poll(Duration::from_millis(500))? {
                    if let Event::Key(_) = event::read()? {
                        break;
                    }
                }
            }
            return Ok(());
        }

        let mod_id = selected_mod.identifier.clone();
        let mod_name = selected_mod.name.clone();

        self.clear_screen(stdout)?;
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print(format!("Checking dependencies for: {}", mod_name)),
            SetForegroundColor(Color::Reset),
            cursor::MoveTo(0, 2),
            Print("Fetching data from Steam Workshop... Please wait."),
        )?;
        stdout.flush()?;

        let dependencies = match dependency_manager::fetch_dependencies(&mod_id) {
            Ok(deps) => deps,
            Err(e) => {
                execute!(
                    stdout,
                    cursor::MoveTo(0, 4),
                    SetForegroundColor(Color::Red),
                    Print(format!("Error fetching dependencies: {}", e)),
                    SetForegroundColor(Color::Reset),
                    cursor::MoveTo(0, 6),
                    Print("Press any key to return...")
                )?;
                stdout.flush()?;
                loop {
                    if event::poll(Duration::from_millis(500))? {
                        if let Event::Key(_) = event::read()? {
                            break;
                        }
                    }
                }
                return Ok(());
            }
        };

        // Process dependencies status
        // We need to know which ones are installed, enabled, etc.
        // We'll map them to a struct or tuple
        #[derive(Clone)]
        struct DepStatus {
            id: String,
            name: String,
            installed: bool,
            enabled: bool,
        }

        let mut dep_statuses = Vec::new();
        let installed_mods = self.mod_manager.loaded_mods.all_items();

        for dep in dependencies {
            let found_mod = installed_mods.iter().find(|m| m.identifier == dep.id);
            dep_statuses.push(DepStatus {
                id: dep.id,
                name: dep.name,
                installed: found_mod.is_some(),
                enabled: found_mod.map(|m| m.enabled).unwrap_or(false),
            });
        }

        loop {
            self.clear_screen(stdout)?;
            execute!(
                stdout,
                cursor::MoveTo(0, 0),
                SetForegroundColor(Color::Cyan),
                Print(format!("Dependencies for: {}", mod_name)),
                SetForegroundColor(Color::Reset),
            )?;

            if dep_statuses.is_empty() {
                execute!(
                    stdout,
                    cursor::MoveTo(0, 2),
                    Print("No dependencies found or required items not listed."),
                )?;
            } else {
                let mut y_offset = 2;
                execute!(
                    stdout,
                    cursor::MoveTo(0, y_offset),
                    Print(format!("{:<15} {:<40} {:<15}", "ID", "Name", "Status")),
                )?;
                y_offset += 2;

                for dep in &dep_statuses {
                    let status_str = if !dep.installed {
                        "MISSING"
                    } else if dep.enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    };

                    let color = if !dep.installed {
                        Color::Red
                    } else if dep.enabled {
                        Color::Green
                    } else {
                        Color::Yellow
                    };

                    execute!(
                        stdout,
                        cursor::MoveTo(0, y_offset),
                        SetForegroundColor(color),
                        Print(format!(
                            "{:<15} {:<40} {:<15}",
                            dep.id, dep.name, status_str
                        )),
                        SetForegroundColor(Color::Reset),
                    )?;
                    y_offset += 1;
                }
            }

            let info_y = if dep_statuses.is_empty() {
                4
            } else {
                dep_statuses.len() as u16 + 5
            };
            execute!(
                stdout,
                cursor::MoveTo(0, info_y),
                Print("Press <E> to Enable all installed, <ESC> to return."),
            )?;
            stdout.flush()?;

            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Esc => break,
                        KeyCode::Char('e') => {
                            // Enable all installed dependencies
                            let ids_to_enable: Vec<String> = dep_statuses
                                .iter()
                                .filter(|d| d.installed && !d.enabled)
                                .map(|d| d.id.clone())
                                .collect();

                            if !ids_to_enable.is_empty() {
                                for m in self.mod_manager.loaded_mods.all_items_mut() {
                                    if ids_to_enable.contains(&m.identifier) {
                                        m.enabled = true;
                                    }
                                }
                                // Update statuses locally for the loop
                                for d in &mut dep_statuses {
                                    if d.installed {
                                        d.enabled = true;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}
