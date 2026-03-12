use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    config::AppConfig,
    core::catalog::{Catalog, CatalogEntry},
};

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let catalog = Catalog::load_all()?;
    let config = AppConfig::load()?;

    let result = run_app(&mut terminal, catalog, config);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    catalog: Catalog,
    config: AppConfig,
) -> Result<()> {
    let mut app = App::new(catalog, config);

    loop {
        terminal.draw(|frame| {
            let preview = build_preview(app.selected_entry().cloned(), app.catalog.is_empty());
            let result_count = app.filtered.len();
            let title = format!(
                "bindfinder  {} entries  {} matches",
                app.catalog.len(),
                result_count
            );
            let areas = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

            let content = if app.query.is_empty() {
                "Search: ".to_string()
            } else {
                format!("Search: {}", app.query)
            };
            frame.render_widget(
                Paragraph::new(content)
                    .block(Block::default().borders(Borders::ALL).title(title)),
                areas[0],
            );

            let left = app.config.settings.result_list_width_percent;
            let right = 100 - left;
            let body = Layout::horizontal([Constraint::Percentage(left), Constraint::Percentage(right)])
                .split(areas[1]);

            frame.render_stateful_widget(
                List::new(build_items(&app.filtered))
                    .block(Block::default().borders(Borders::ALL).title("Results"))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    .highlight_symbol("> "),
                body[0],
                &mut app.list_state,
            );

            let preview_widget =
                Paragraph::new(preview).block(Block::default().borders(Borders::ALL).title("Preview"));
            if app.config.settings.wrap_preview {
                frame.render_widget(preview_widget.wrap(Wrap { trim: false }), body[1]);
            } else {
                frame.render_widget(preview_widget, body[1]);
            }

            if app.config.settings.show_footer {
                frame.render_widget(
                    Paragraph::new(format!(
                        "Type to search  {}: up  {}: down  {}: clear  {}: quit",
                        display_bindings(&app.config.keybindings.move_up),
                        display_bindings(&app.config.keybindings.move_down),
                        display_bindings(&app.config.keybindings.clear_query),
                        display_bindings(&app.config.keybindings.quit),
                    )),
                    areas[2],
                );
            }
        })?;

        if let Event::Key(key) = event::read()? {
            let should_quit = app.config.keybindings.matches_quit(key);

            if should_quit {
                return Ok(());
            }

            app.handle_key(key);
        }
    }
}

struct App {
    config: AppConfig,
    catalog: Catalog,
    query: String,
    filtered: Vec<CatalogEntry>,
    list_state: ListState,
}

impl App {
    fn new(catalog: Catalog, config: AppConfig) -> Self {
        let mut app = Self {
            config,
            catalog,
            query: String::new(),
            filtered: Vec::new(),
            list_state: ListState::default(),
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.filtered = self
            .catalog
            .filter(&self.query)
            .into_iter()
            .cloned()
            .collect();

        let next_index = match self.list_state.selected() {
            Some(index) if index < self.filtered.len() => Some(index),
            _ if self.filtered.is_empty() => None,
            _ => Some(0),
        };
        self.list_state.select(next_index);
    }

    fn selected_entry(&self) -> Option<&CatalogEntry> {
        self.list_state
            .selected()
            .and_then(|index| self.filtered.get(index))
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if self.config.keybindings.matches_clear_query(key) {
            self.query.clear();
            self.refresh();
            return;
        }
        if self.config.keybindings.matches_move_up(key) {
            self.move_selection(-1);
            return;
        }
        if self.config.keybindings.matches_move_down(key) {
            self.move_selection(1);
            return;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.query.push(ch);
                self.refresh();
            }
            (KeyCode::Backspace, _) => {
                self.query.pop();
                self.refresh();
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.filtered.is_empty() {
            self.list_state.select(None);
            return;
        }

        let current = self.list_state.selected().unwrap_or(0) as isize;
        let max = self.filtered.len() as isize - 1;
        let next = (current + delta).clamp(0, max) as usize;
        self.list_state.select(Some(next));
    }
}

fn build_items(entries: &[CatalogEntry]) -> Vec<ListItem<'_>> {
    if entries.is_empty() {
        return vec![ListItem::new("No matches")];
    }

    entries
        .iter()
        .map(|item| {
            let meta = match (&item.entry.keys, &item.entry.command) {
                (Some(keys), Some(command)) => format!("{} | {}", keys, command),
                (Some(keys), None) => keys.to_string(),
                (None, Some(command)) => command.to_string(),
                (None, None) => item.entry.entry_type.as_str().to_string(),
            };

            ListItem::new(vec![
                Line::raw(format!("{}: {}", item.tool, item.entry.title)),
                Line::raw(format!("  {}", meta)),
            ])
        })
        .collect()
}

fn build_preview(selected: Option<CatalogEntry>, catalog_is_empty: bool) -> Text<'static> {
    if catalog_is_empty {
        return Text::from("No built-in packs are available.");
    }

    let Some(item) = selected else {
        return Text::from("No matching entries.");
    };

    let mut lines = vec![
        Line::raw(format!("Tool: {}", item.tool)),
        Line::raw(format!("Title: {}", item.entry.title)),
        Line::raw(format!("Type: {}", item.entry.entry_type.as_str())),
    ];

    if let Some(keys) = &item.entry.keys {
        lines.push(Line::raw(format!("Keys: {}", keys)));
    }

    if let Some(command) = &item.entry.command {
        lines.push(Line::raw(format!("Command: {}", command)));
    }

    lines.push(Line::raw(String::new()));
    lines.push(Line::raw(item.entry.description.clone()));

    if !item.entry.examples.is_empty() {
        lines.push(Line::raw(String::new()));
        lines.push(Line::raw("Examples:"));
        for example in &item.entry.examples {
            lines.push(Line::raw(format!("- {}", example)));
        }
    }

    if !item.entry.tags.is_empty() {
        lines.push(Line::raw(String::new()));
        lines.push(Line::raw(format!("Tags: {}", item.entry.tags.join(", "))));
    }

    if !item.entry.aliases.is_empty() {
        lines.push(Line::raw(format!("Aliases: {}", item.entry.aliases.join(", "))));
    }

    lines.push(Line::raw(String::new()));
    lines.push(Line::raw(format!("Pack: {}", item.pack_title)));
    lines.push(Line::raw(format!("Pack ID: {}", item.pack_id)));
    lines.push(Line::raw(format!("Source: {}", item.source)));

    Text::from(lines)
}

fn display_bindings(bindings: &[crate::config::KeyBinding]) -> String {
    bindings
        .iter()
        .map(display_binding)
        .collect::<Vec<_>>()
        .join("/")
}

fn display_binding(binding: &crate::config::KeyBinding) -> String {
    let mut parts = Vec::new();
    if binding.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if binding.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if binding.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift".to_string());
    }

    let key = match binding.code {
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Char(ch) => ch.to_string(),
        _ => "?".to_string(),
    };

    parts.push(key);
    parts.join("+")
}
