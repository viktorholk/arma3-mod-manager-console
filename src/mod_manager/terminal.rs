use std::{
    io::{self, Stdout, Write},
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

use crate::errors::{AppError, AppResult};

use super::ModManager;

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

    pub fn run(&mut self) -> AppResult<()> {
        let mut stdout = io::stdout();

        execute!(stdout, cursor::SavePosition)?;
        execute!(stdout, terminal::EnterAlternateScreen)?;
        execute!(stdout, crossterm::cursor::Hide)?;

        terminal::enable_raw_mode()?;

        self.main_loop(&mut stdout)?;

        terminal::disable_raw_mode()?;

        execute!(stdout, terminal::LeaveAlternateScreen)?;
        execute!(stdout, cursor::RestorePosition)?;
        execute!(stdout, crossterm::cursor::Show)?;

        Ok(())
    }

    fn render(&self, stdout: &mut Stdout) -> AppResult<()> {
        self.clear_screen(stdout)?;

        let mut top_offset = 0;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            cursor::MoveTo(0, top_offset),
            Print("Arma 3 Mod Manager Console"),
            SetForegroundColor(Color::Reset)
        )?;

        top_offset += 2;

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

            //str.truncate(36);

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
            ("Refresh Mods", "R"),
            ("Set Custom Parameters", "F"),
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
                            let value = if self
                                .mod_manager
                                .loaded_mods
                                .all_items()
                                .iter()
                                .all(|m| m.enabled)
                            {
                                false
                            } else {
                                true
                            };

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
                        KeyCode::Char('f') => {
                            self.set_custom_parameters_screen(stdout)?;
                        }
                        KeyCode::Char('p') => {
                            self.start_game()?;
                        }

                        KeyCode::Esc => break,

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

        let game_app_path = game_path.join("arma3.app");
        let game_app_path_str = game_app_path.to_string_lossy().to_string();

        if !game_app_path.exists() {
            return Err(AppError::InvalidPath(game_app_path_str.to_owned()));
        }

        let mut command = Command::new("open");

        command.args(["-a", &game_app_path_str]);

        // Remove existing symlinks from the game directory
        super::file_handler::remove_dir_symlinks(game_path)?;

        if !enabled_mods.is_empty() {
            command.arg("--args");

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
            if default_args.len() > 0 {
                command.arg(default_args);
            }

            let mod_list = enabled_mods
                .iter()
                .map(|m| m.identifier.as_str())
                .collect::<Vec<_>>()
                .join(";");

            command.arg(&format!("-mod={}", mod_list));
        }

        command.output()?;

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
}
