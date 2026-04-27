use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::config::MacroConfig;

/// A read-only preview widget that renders a single macro's phrase, condition,
/// and ordered key sequence inside a bordered pane.
pub struct MacroEditor {
    /// The macro configuration being displayed (phrase, condition flag, key list).
    pub macro_config: MacroConfig,
    /// Cursor position for future edit interactions; currently unused in rendering.
    pub cursor: usize,
}

impl MacroEditor {
    /// Wrap a [`MacroConfig`] in a new `MacroEditor` with cursor at position 0.
    pub fn new(config: MacroConfig) -> Self {
        Self {
            macro_config: config,
            cursor: 0,
        }
    }

    /// Render the macro detail view into `area`: shows phrase, condition flag, and
    /// the numbered key-sequence list.
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!(" Editing: {} ", self.macro_config.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        
        let text = vec![
            Line::from(vec![
                Span::raw("Phrase: "),
                Span::styled(
                    self.macro_config.phrase.as_deref().unwrap_or("<none>"),
                    Style::default().fg(Color::Cyan)
                ),
            ]),
            Line::from(vec![
                Span::raw("Condition: "),
                Span::styled(
                    self.macro_config.if_flag.as_deref().unwrap_or("<none>"),
                    Style::default().fg(Color::Cyan)
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled("Key Sequence:", Style::default().add_modifier(Modifier::UNDERLINED))),
        ];

        let mut items = text;
        for (i, key) in self.macro_config.keys.iter().enumerate() {
            items.push(Line::from(format!("  {}. {}", i + 1, key.key)));
        }

        let paragraph = Paragraph::new(items).block(block);
        f.render_widget(paragraph, area);
    }
}
