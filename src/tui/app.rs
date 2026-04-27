use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::pack::Pack;
use crossterm::event::KeyCode;

/// Top-level TUI application state, holding the loaded pack and the current
/// UI selection and mode.
pub struct App {
    /// The macro pack loaded from the first profile directory found on disk,
    /// or `None` if no profiles exist.
    pub pack: Option<Pack>,
    /// Index of the currently highlighted category in the browser view.
    pub selected_category: usize,
    /// Index of the currently highlighted macro within the selected category.
    pub selected_macro: usize,
    /// Whether the app is browsing macros or editing one.
    pub mode: AppMode,
}

/// The two top-level UI modes: browsing the macro list vs. editing a single macro.
#[derive(Debug, PartialEq)]
pub enum AppMode {
    /// Macro browser — arrow keys navigate categories and macros.
    Browser,
    /// Macro editor — focused on a single macro's phrase, condition, and key sequence.
    Editor,
}

impl App {
    /// Create a new `App`, loading the first pack found under the profiles directory.
    ///
    /// Returns `Ok` with `pack: None` when no profile directories exist yet.
    pub fn new() -> Result<Self> {
        // Load first profile found
        let profiles_dir = crate::pack::get_profiles_dir()?;
        let first_profile = std::fs::read_dir(profiles_dir)?
            .filter_map(|e| e.ok())
            .find(|e| e.path().is_dir());

        let pack = if let Some(p) = first_profile {
            crate::pack::Pack::load_from_dir(&p.path()).ok()
        } else {
            None
        };

        Ok(Self {
            pack,
            selected_category: 0,
            selected_macro: 0,
            mode: AppMode::Browser,
        })
    }

    /// Render the full TUI layout into `f`: header bar, categories/macros panes, and footer hints.
    pub fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(f.area());

        // Header
        let header = Paragraph::new(" VIBE-ATTACK // TACTICAL MACRO EDITOR ")
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Thick))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Main content
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        // Categories List
        let categories = Block::default()
            .title(" Categories ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        f.render_widget(categories, main_chunks[0]);

        // Macros List
        let macros = Block::default()
            .title(" Macros ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        f.render_widget(macros, main_chunks[1]);

        // Footer
        let footer = Paragraph::new(" [q] Quit | [Arrows] Navigate | [Enter] Edit | [t] Test ")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, chunks[2]);
    }

    /// Dispatch a key press to the appropriate mode handler.
    ///
    /// In [`AppMode::Browser`] the arrow keys scroll the macro list and `Enter`
    /// switches to [`AppMode::Editor`].  In [`AppMode::Editor`] `Esc` returns to
    /// the browser.
    pub fn handle_key(&mut self, code: KeyCode) {
        match self.mode {
            AppMode::Browser => {
                if let Some(pack) = &self.pack {
                    match code {
                        KeyCode::Up if self.selected_macro > 0 => {
                            self.selected_macro -= 1;
                        }
                        KeyCode::Down => {
                            let cat = &pack.categories[self.selected_category];
                            if self.selected_macro < cat.macros.len() - 1 {
                                self.selected_macro += 1;
                            }
                        }
                        KeyCode::Left if self.selected_category > 0 => {
                            self.selected_category -= 1;
                            self.selected_macro = 0;
                        }
                        KeyCode::Right if self.selected_category < pack.categories.len() - 1 => {
                            self.selected_category += 1;
                            self.selected_macro = 0;
                        }
                        KeyCode::Enter => {
                            self.mode = AppMode::Editor;
                        }
                        _ => {}
                    }
                }
            }
            AppMode::Editor => {
                if code == KeyCode::Esc {
                    self.mode = AppMode::Browser;
                }
            }
        }
    }
}
