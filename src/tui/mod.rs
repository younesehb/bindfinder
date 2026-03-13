use std::{env, fs, io};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};

use crate::{
    config::AppConfig,
    core::catalog::{Catalog, CatalogEntry},
    integration::{detect::EnvironmentInfo, install::effective_hotkey},
    state::UserState,
    update::UpdateInfo,
};

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let catalog = Catalog::load_all()?;
    let config = load_config_for_tui()?;
    let state = UserState::load().unwrap_or_default();
    let update_notice = crate::update::cached_or_fetch(env!("CARGO_PKG_VERSION"));
    let environment = EnvironmentInfo::detect();

    let result = run_app(
        &mut terminal,
        catalog,
        config,
        state,
        update_notice,
        environment,
    );

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let selected = result?;
    if let Some(command) = selected {
        if let Some(path) = env::var_os("BINDFINDER_OUTPUT_FILE") {
            fs::write(path, &command)?;
        } else {
            println!("{command}");
        }
    }

    Ok(())
}

fn load_config_for_tui() -> Result<AppConfig> {
    AppConfig::load_with_path()
        .map(|(config, _)| config)
        .map_err(|err| {
            if let Some(path) = AppConfig::default_path() {
                err.context(format!("invalid config: {}", path.display()))
            } else {
                err.context("invalid config")
            }
        })
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    catalog: Catalog,
    config: AppConfig,
    state: UserState,
    update_notice: Option<UpdateInfo>,
    environment: EnvironmentInfo,
) -> Result<Option<String>> {
    let mut app = App::new(catalog, config, state, update_notice, environment);

    loop {
        terminal.draw(|frame| {
            let areas = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

            if let Some(prompt) = app.argument_prompt.as_ref() {
                let title = format!("bindfinder  arguments  {} field(s)", prompt.fields.len());
                let selected_field = prompt
                    .fields
                    .get(prompt.selected)
                    .map(|field| field.name.as_str())
                    .unwrap_or("none");
                let content = format!("Argument: {selected_field}");
                frame.render_widget(
                    Paragraph::new(content)
                        .block(Block::default().borders(Borders::ALL).title(title)),
                    areas[0],
                );

                let left = app.config.settings.result_list_width_percent;
                let right = 100 - left;
                let body =
                    Layout::horizontal([Constraint::Percentage(left), Constraint::Percentage(right)])
                        .split(areas[1]);

                let mut argument_state = ListState::default();
                argument_state.select(Some(prompt.selected));
                frame.render_stateful_widget(
                    List::new(build_argument_items(prompt))
                        .block(Block::default().borders(Borders::ALL).title("Arguments"))
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                        .highlight_symbol("> "),
                    body[0],
                    &mut argument_state,
                );

                let preview_widget = Paragraph::new(build_argument_preview(prompt))
                    .block(Block::default().borders(Borders::ALL).title("Command"));
                if app.config.settings.wrap_preview {
                    frame.render_widget(preview_widget.wrap(Wrap { trim: false }), body[1]);
                } else {
                    frame.render_widget(preview_widget, body[1]);
                }

                if app.config.settings.show_footer {
                    frame.render_widget(
                        Paragraph::new(
                            "Arguments: type to replace  Tab/Down next  Shift-Tab/Up prev  Ctrl-u clear  Enter insert  Esc cancel",
                        ),
                        areas[2],
                    );
                }
            } else {
                let preview =
                    build_preview(app.selected_entry().cloned(), app.catalog.is_empty(), &app.state);
                let result_count = app.filtered.len();
                let hidden_flag = if app.show_hidden { "  [hidden visible]" } else { "" };
                let favorites_flag = if app.favorites_only { "  [favorites only]" } else { "" };
                let update_flag = app
                    .update_notice
                    .as_ref()
                    .map(|info| {
                        format!(
                            "  [update {} -> {}: bindfinder update]",
                            info.current_version, info.latest_version
                        )
                    })
                    .unwrap_or_default();
                let title = format!(
                    "bindfinder  {} entries  {} matches  open: {}{}{}{}",
                    app.catalog.len(),
                    result_count,
                    app.launch_hint,
                    hidden_flag,
                    favorites_flag,
                    update_flag
                );

                let prefix = match app.input_mode {
                    InputMode::Normal => "Search",
                    InputMode::Search => "Search *",
                    InputMode::Arguments => "Argument",
                };
                let content = if app.query.is_empty() {
                    format!("{prefix}: ")
                } else {
                    format!("{prefix}: {}", app.query)
                };
                frame.render_widget(
                    Paragraph::new(content)
                        .block(Block::default().borders(Borders::ALL).title(title)),
                    areas[0],
                );

                let left = app.config.settings.result_list_width_percent;
                let right = 100 - left;
                let body =
                    Layout::horizontal([Constraint::Percentage(left), Constraint::Percentage(right)])
                        .split(areas[1]);

                frame.render_stateful_widget(
                    List::new(build_items(&app.filtered, &app.state))
                        .block(Block::default().borders(Borders::ALL).title("Results"))
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                        .highlight_symbol("> "),
                    body[0],
                    &mut app.list_state,
                );

                let preview_widget = Paragraph::new(preview)
                    .block(Block::default().borders(Borders::ALL).title("Preview"));
                if app.config.settings.wrap_preview {
                    frame.render_widget(preview_widget.wrap(Wrap { trim: false }), body[1]);
                } else {
                    frame.render_widget(preview_widget, body[1]);
                }

                if app.config.settings.show_footer {
                    let footer = match app.input_mode {
                        InputMode::Normal => {
                            "Normal: j/k move  Ctrl-d/Ctrl-u page  gg/G ends  / search  z hidden  m favorites  f/x entry  F/X tool  Enter select"
                        }
                        InputMode::Search => {
                            "Search: type filter  Up/Down move  Ctrl-d/Ctrl-u page  Enter select  Esc normal"
                        }
                        InputMode::Arguments => "",
                    };
                    let footer = if let Some(info) = app.update_notice.as_ref() {
                        format!(
                            "{}  Open: {}  Update available: {} -> {}. Run `bindfinder update`",
                            footer, app.launch_hint, info.current_version, info.latest_version
                        )
                    } else {
                        format!("{footer}  Open: {}", app.launch_hint)
                    };
                    frame.render_widget(Paragraph::new(footer), areas[2]);
                }
            }
        })?;

        if let Event::Key(key) = event::read()? {
            let should_quit = app.config.keybindings.matches_quit(key)
                && !(app.input_mode == InputMode::Search
                    && matches!(key.code, KeyCode::Esc)
                    && key.modifiers == KeyModifiers::NONE);

            if should_quit {
                return Ok(None);
            }

            if let Some(selected) = app.handle_key(key) {
                return Ok(Some(selected));
            }
        }
    }
}

struct App {
    config: AppConfig,
    catalog: Catalog,
    state: UserState,
    query: String,
    filtered: Vec<CatalogEntry>,
    list_state: ListState,
    input_mode: InputMode,
    pending_sequence: Vec<crate::config::KeyBinding>,
    show_hidden: bool,
    favorites_only: bool,
    argument_prompt: Option<ArgumentPrompt>,
    update_notice: Option<UpdateInfo>,
    launch_hint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Normal,
    Search,
    Arguments,
}

#[derive(Clone)]
struct ArgumentPrompt {
    template: Vec<CommandPart>,
    fields: Vec<ArgumentField>,
    selected: usize,
    previous_mode: InputMode,
}

#[derive(Clone)]
struct ArgumentField {
    name: String,
    value: String,
    edited: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CommandPart {
    Text(String),
    Placeholder(String),
}

impl App {
    fn new(
        catalog: Catalog,
        config: AppConfig,
        state: UserState,
        update_notice: Option<UpdateInfo>,
        environment: EnvironmentInfo,
    ) -> Self {
        let target = environment.choose_target(&config);
        let launch_hint = launch_hint_label(&target, &config);
        let mut app = Self {
            config,
            catalog,
            state,
            query: String::new(),
            filtered: Vec::new(),
            list_state: ListState::default(),
            input_mode: InputMode::Search,
            pending_sequence: Vec::new(),
            show_hidden: false,
            favorites_only: false,
            argument_prompt: None,
            update_notice,
            launch_hint,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.filtered = self
            .catalog
            .filter_with_state(
                &self.query,
                &self.state,
                self.show_hidden,
                self.favorites_only,
            )
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

    fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Search => self.handle_search_key(key),
            InputMode::Arguments => self.handle_argument_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> Option<String> {
        if self.try_sequence_action(key) {
            return None;
        }

        if self.config.keybindings.matches_select(key) {
            return self.selected_output();
        }

        match (key.code, key.modifiers) {
            _ if self.config.keybindings.matches_search_mode(key) => {
                self.query.clear();
                self.refresh();
                self.input_mode = InputMode::Search;
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_move_down(key) => {
                self.move_selection(1);
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_move_up(key) => {
                self.move_selection(-1);
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_page_down(key) => {
                self.move_page(1);
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_page_up(key) => {
                self.move_page(-1);
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_toggle_hidden(key) => {
                self.show_hidden = !self.show_hidden;
                self.pending_sequence.clear();
                self.refresh();
                return None;
            }
            _ if self.config.keybindings.matches_toggle_favorites_only(key) => {
                self.favorites_only = !self.favorites_only;
                self.pending_sequence.clear();
                self.refresh();
                return None;
            }
            _ if self.config.keybindings.matches_favorite_entry(key) => {
                self.toggle_selected_favorite();
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_hide_entry(key) => {
                self.toggle_selected_hidden();
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_favorite_tool(key) => {
                self.toggle_selected_tool_favorite();
                self.pending_sequence.clear();
                return None;
            }
            _ if self.config.keybindings.matches_hide_tool(key) => {
                self.toggle_selected_tool_hidden();
                self.pending_sequence.clear();
                return None;
            }
            _ => {
                self.pending_sequence.clear();
            }
        }

        None
    }

    fn try_sequence_action(&mut self, key: KeyEvent) -> bool {
        let binding = self.config.keybindings.key_from_event(key);

        if !self.pending_sequence.is_empty() {
            self.pending_sequence.push(binding.clone());

            if self
                .config
                .keybindings
                .goto_top
                .iter()
                .any(|sequence| sequence.matches_exact(&self.pending_sequence))
            {
                self.select_index(0);
                self.pending_sequence.clear();
                return true;
            }
            if self
                .config
                .keybindings
                .goto_bottom
                .iter()
                .any(|sequence| sequence.matches_exact(&self.pending_sequence))
            {
                self.select_last();
                self.pending_sequence.clear();
                return true;
            }
            if self
                .config
                .keybindings
                .goto_top
                .iter()
                .chain(self.config.keybindings.goto_bottom.iter())
                .any(|sequence| sequence.matches_prefix(&self.pending_sequence))
            {
                return true;
            }

            self.pending_sequence.clear();
        }

        let single = [binding];
        if self
            .config
            .keybindings
            .goto_top
            .iter()
            .any(|sequence| sequence.matches_exact(&single))
        {
            self.select_index(0);
            return true;
        }
        if self
            .config
            .keybindings
            .goto_bottom
            .iter()
            .any(|sequence| sequence.matches_exact(&single))
        {
            self.select_last();
            return true;
        }
        if self
            .config
            .keybindings
            .goto_top
            .iter()
            .chain(self.config.keybindings.goto_bottom.iter())
            .any(|sequence| sequence.matches_prefix(&single))
        {
            self.pending_sequence = single.to_vec();
            return true;
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Option<String> {
        if self.config.keybindings.matches_clear_query(key) {
            self.query.clear();
            self.refresh();
            return None;
        }

        if self.config.keybindings.matches_select(key) {
            return self.selected_output();
        }

        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => {
                self.input_mode = InputMode::Normal;
                self.pending_sequence.clear();
            }
            (KeyCode::Backspace, _) => {
                self.query.pop();
                self.refresh();
            }
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.query.push(ch);
                self.refresh();
            }
            _ if self.config.keybindings.matches_move_up(key) => self.move_selection(-1),
            _ if self.config.keybindings.matches_move_down(key) => self.move_selection(1),
            _ if self.config.keybindings.matches_page_down(key) => self.move_page(1),
            _ if self.config.keybindings.matches_page_up(key) => self.move_page(-1),
            _ => {}
        }

        None
    }

    fn handle_argument_key(&mut self, key: KeyEvent) -> Option<String> {
        let Some(prompt) = self.argument_prompt.as_mut() else {
            self.input_mode = InputMode::Search;
            return None;
        };

        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => {
                self.input_mode = prompt.previous_mode;
                self.argument_prompt = None;
            }
            _ if self.config.keybindings.matches_select(key) => {
                return self.submit_argument_prompt();
            }
            (KeyCode::Tab, _) | (KeyCode::Down, _) => {
                if !prompt.fields.is_empty() {
                    prompt.selected = (prompt.selected + 1) % prompt.fields.len();
                }
            }
            (KeyCode::BackTab, _) | (KeyCode::Up, _) => {
                if !prompt.fields.is_empty() {
                    prompt.selected = if prompt.selected == 0 {
                        prompt.fields.len() - 1
                    } else {
                        prompt.selected - 1
                    };
                }
            }
            (KeyCode::Backspace, _) => {
                if let Some(field) = prompt.fields.get_mut(prompt.selected) {
                    if !field.edited {
                        field.value.clear();
                        field.edited = true;
                    } else {
                        field.value.pop();
                    }
                }
            }
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                if let Some(field) = prompt.fields.get_mut(prompt.selected) {
                    if !field.edited {
                        field.value.clear();
                        field.edited = true;
                    }
                    field.value.push(ch);
                }
            }
            _ if self.config.keybindings.matches_clear_query(key) => {
                if let Some(field) = prompt.fields.get_mut(prompt.selected) {
                    field.value.clear();
                    field.edited = true;
                }
            }
            _ => {}
        }

        None
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

    fn move_page(&mut self, pages: isize) {
        const PAGE_SIZE: isize = 10;
        self.move_selection(pages * PAGE_SIZE);
    }

    fn select_index(&mut self, index: usize) {
        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state
                .select(Some(index.min(self.filtered.len() - 1)));
        }
    }

    fn select_last(&mut self) {
        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(self.filtered.len() - 1));
        }
    }

    fn toggle_selected_favorite(&mut self) {
        let Some(selected) = self.selected_entry().cloned() else {
            return;
        };
        let qualified_id = selected.qualified_id();
        self.state.toggle_entry_favorite(&qualified_id);
        let _ = self.state.save();
        self.refresh();
        self.restore_selection(&qualified_id);
    }

    fn toggle_selected_hidden(&mut self) {
        let Some(selected) = self.selected_entry().cloned() else {
            return;
        };
        self.state.toggle_entry_hidden(&selected.qualified_id());
        let _ = self.state.save();
        self.refresh();
    }

    fn toggle_selected_tool_favorite(&mut self) {
        let Some(selected) = self.selected_entry().cloned() else {
            return;
        };
        let qualified_id = selected.qualified_id();
        self.state.toggle_tool_favorite(&selected.tool);
        let _ = self.state.save();
        self.refresh();
        self.restore_selection(&qualified_id);
    }

    fn toggle_selected_tool_hidden(&mut self) {
        let Some(selected) = self.selected_entry().cloned() else {
            return;
        };
        self.state.toggle_tool_hidden(&selected.tool);
        let _ = self.state.save();
        self.refresh();
    }

    fn restore_selection(&mut self, qualified_id: &str) {
        if let Some(index) = self
            .filtered
            .iter()
            .position(|entry| entry.qualified_id() == qualified_id)
        {
            self.list_state.select(Some(index));
        }
    }

    fn selected_output(&mut self) -> Option<String> {
        let item = self.selected_entry()?.clone();
        let command = item
            .entry
            .command
            .clone()
            .unwrap_or_else(|| item.entry.title.clone());
        let template = parse_command_template(&command);
        let fields: Vec<ArgumentField> = template
            .iter()
            .filter_map(|part| match part {
                CommandPart::Placeholder(name) => Some(ArgumentField {
                    name: name.clone(),
                    value: name.clone(),
                    edited: false,
                }),
                CommandPart::Text(_) => None,
            })
            .collect();

        if fields.is_empty() {
            Some(render_command_template(&template, &[]))
        } else {
            self.argument_prompt = Some(ArgumentPrompt {
                template,
                fields,
                selected: 0,
                previous_mode: self.input_mode,
            });
            self.input_mode = InputMode::Arguments;
            None
        }
    }

    fn submit_argument_prompt(&mut self) -> Option<String> {
        let prompt = self.argument_prompt.take()?;
        let values: Vec<String> = prompt.fields.into_iter().map(|field| field.value).collect();
        self.input_mode = prompt.previous_mode;
        Some(render_command_template(&prompt.template, &values))
    }
}

fn launch_hint_label(
    target: &crate::integration::detect::IntegrationTarget,
    config: &AppConfig,
) -> String {
    effective_hotkey(config, target)
}

fn build_items<'a>(entries: &'a [CatalogEntry], state: &'a UserState) -> Vec<ListItem<'a>> {
    if entries.is_empty() {
        return vec![ListItem::new("No matches")];
    }

    entries
        .iter()
        .map(|item| {
            let marker = item_marker(item, state);
            let meta = match (&item.entry.keys, &item.entry.command) {
                (Some(keys), Some(command)) => format!("{} | {}", keys, command),
                (Some(keys), None) => keys.to_string(),
                (None, Some(command)) => command.to_string(),
                (None, None) => item.entry.entry_type.as_str().to_string(),
            };

            ListItem::new(vec![
                Line::raw(format!("{} {}: {}", marker, item.tool, item.entry.title)),
                Line::raw(format!("  {}", meta)),
            ])
        })
        .collect()
}

fn build_preview(
    selected: Option<CatalogEntry>,
    catalog_is_empty: bool,
    state: &UserState,
) -> Text<'static> {
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

    let qualified_id = item.qualified_id();
    let mut flags = Vec::new();
    if state.is_entry_favorite(&qualified_id) {
        flags.push("favorite entry");
    }
    if state.is_tool_favorite(&item.tool) {
        flags.push("favorite tool");
    }
    if state.is_entry_hidden(&qualified_id) {
        flags.push("hidden entry");
    }
    if state.is_tool_hidden(&item.tool) {
        flags.push("hidden tool");
    }
    if !flags.is_empty() {
        lines.push(Line::raw(format!("Flags: {}", flags.join(", "))));
    }

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
        lines.push(Line::raw(format!(
            "Aliases: {}",
            item.entry.aliases.join(", ")
        )));
    }

    lines.push(Line::raw(String::new()));
    lines.push(Line::raw(format!("Pack: {}", item.pack_title)));
    lines.push(Line::raw(format!("Pack ID: {}", item.pack_id)));
    lines.push(Line::raw(format!("Entry ID: {}", item.qualified_id())));
    lines.push(Line::raw(format!("Source: {}", item.source)));

    Text::from(lines)
}

fn build_argument_items(prompt: &ArgumentPrompt) -> Vec<ListItem<'static>> {
    prompt
        .fields
        .iter()
        .map(|field| {
            ListItem::new(vec![
                Line::raw(field.name.clone()),
                Line::raw(format!("  {}", field.value)),
            ])
        })
        .collect()
}

fn build_argument_preview(prompt: &ArgumentPrompt) -> Text<'static> {
    let rendered = render_command_template(
        &prompt.template,
        &prompt
            .fields
            .iter()
            .map(|field| field.value.clone())
            .collect::<Vec<_>>(),
    );

    let mut lines = vec![
        Line::raw("Fill arguments, then press Enter to insert the command."),
        Line::raw("Leave values unchanged to keep the default placeholder text."),
        Line::raw(String::new()),
        Line::raw("Rendered command:"),
        Line::raw(rendered),
    ];

    if !prompt.fields.is_empty() {
        lines.push(Line::raw(String::new()));
        lines.push(Line::raw("Current fields:"));
        for field in &prompt.fields {
            lines.push(Line::raw(format!("- {} = {}", field.name, field.value)));
        }
    }

    Text::from(lines)
}

fn item_marker(item: &CatalogEntry, state: &UserState) -> &'static str {
    let qualified_id = item.qualified_id();
    if state.is_entry_favorite(&qualified_id) || state.is_tool_favorite(&item.tool) {
        "*"
    } else {
        " "
    }
}

fn parse_command_template(input: &str) -> Vec<CommandPart> {
    let mut parts = Vec::new();
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut placeholder = String::new();
            let mut found_end = false;

            while let Some(&next) = chars.peek() {
                chars.next();
                if next == '>' {
                    found_end = true;
                    break;
                }
                placeholder.push(next);
            }

            if found_end && !placeholder.is_empty() {
                if !current_text.is_empty() {
                    parts.push(CommandPart::Text(std::mem::take(&mut current_text)));
                }
                parts.push(CommandPart::Placeholder(placeholder));
            } else {
                current_text.push('<');
                current_text.push_str(&placeholder);
                if found_end {
                    current_text.push('>');
                }
            }
        } else {
            current_text.push(ch);
        }
    }

    if !current_text.is_empty() {
        parts.push(CommandPart::Text(current_text));
    }

    parts
}

fn render_command_template(template: &[CommandPart], values: &[String]) -> String {
    let mut output = String::new();
    let mut next_value = 0usize;

    for part in template {
        match part {
            CommandPart::Text(text) => output.push_str(text),
            CommandPart::Placeholder(name) => {
                let value = values
                    .get(next_value)
                    .cloned()
                    .unwrap_or_else(|| name.clone());
                output.push_str(&value);
                next_value += 1;
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pack::{Entry, EntryType, Pack, PackMeta};

    fn test_environment() -> EnvironmentInfo {
        EnvironmentInfo {
            inside_tmux: false,
            over_ssh: false,
            shell: None,
            terminal: None,
        }
    }

    fn sample_catalog() -> Catalog {
        Catalog::from_packs(vec![Pack {
            pack: PackMeta {
                id: "test-pack".to_string(),
                tool: "tmux".to_string(),
                title: "tmux".to_string(),
                version: "0.1.0".to_string(),
                source: "test".to_string(),
            },
            entries: vec![Entry {
                id: "split-pane".to_string(),
                entry_type: EntryType::Command,
                title: "Split pane".to_string(),
                keys: Some("%".to_string()),
                command: Some("tmux split-window".to_string()),
                description: "Split the current pane".to_string(),
                examples: Vec::new(),
                tags: Vec::new(),
                aliases: vec!["split".to_string()],
            }],
        }])
        .expect("catalog should build")
    }

    #[test]
    fn app_starts_in_search_mode_and_types_immediately() {
        let mut app = App::new(
            sample_catalog(),
            AppConfig::default(),
            UserState::default(),
            None,
            test_environment(),
        );
        assert_eq!(app.input_mode, InputMode::Search);

        let result = app.handle_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE));
        assert!(result.is_none());
        assert_eq!(app.query, "m");
        assert!(!app.favorites_only);
    }

    #[test]
    fn render_command_template_replaces_placeholders() {
        let template = parse_command_template("apt install <package-name>");
        assert_eq!(
            render_command_template(&template, &["package-name".to_string()]),
            "apt install package-name"
        );
    }

    #[test]
    fn selected_output_enters_argument_mode_for_placeholder_commands() {
        let catalog = Catalog::from_packs(vec![Pack {
            pack: PackMeta {
                id: "test-pack".to_string(),
                tool: "apt".to_string(),
                title: "apt".to_string(),
                version: "0.1.0".to_string(),
                source: "test".to_string(),
            },
            entries: vec![Entry {
                id: "install-package".to_string(),
                entry_type: EntryType::Command,
                title: "Install package".to_string(),
                keys: None,
                command: Some("apt install <package-name>".to_string()),
                description: "Install a package".to_string(),
                examples: Vec::new(),
                tags: Vec::new(),
                aliases: Vec::new(),
            }],
        }])
        .expect("catalog should build");

        let mut app = App::new(
            catalog,
            AppConfig::default(),
            UserState::default(),
            None,
            test_environment(),
        );
        let result = app.selected_output();

        assert!(result.is_none());
        assert_eq!(app.input_mode, InputMode::Arguments);
        let prompt = app.argument_prompt.expect("argument prompt should exist");
        assert_eq!(prompt.fields.len(), 1);
        assert_eq!(prompt.fields[0].value, "package-name");
    }

    #[test]
    fn render_command_template_handles_multiple_placeholders() {
        let template = parse_command_template("docker exec -it <container> <command>");
        assert_eq!(
            render_command_template(&template, &["container".to_string(), "command".to_string()]),
            "docker exec -it container command"
        );
    }

    #[test]
    fn submit_argument_prompt_uses_edited_values() {
        let catalog = Catalog::from_packs(vec![Pack {
            pack: PackMeta {
                id: "test-pack".to_string(),
                tool: "docker".to_string(),
                title: "docker".to_string(),
                version: "0.1.0".to_string(),
                source: "test".to_string(),
            },
            entries: vec![Entry {
                id: "exec".to_string(),
                entry_type: EntryType::Command,
                title: "Docker exec".to_string(),
                keys: None,
                command: Some("docker exec -it <container> <command>".to_string()),
                description: "Run a command in a container".to_string(),
                examples: Vec::new(),
                tags: Vec::new(),
                aliases: Vec::new(),
            }],
        }])
        .expect("catalog should build");

        let mut app = App::new(
            catalog,
            AppConfig::default(),
            UserState::default(),
            None,
            test_environment(),
        );
        assert!(app.selected_output().is_none());

        let prompt = app
            .argument_prompt
            .as_mut()
            .expect("argument prompt should exist");
        prompt.fields[0].value = "web".to_string();
        prompt.fields[0].edited = true;
        prompt.fields[1].value = "bash".to_string();
        prompt.fields[1].edited = true;

        let output = app.submit_argument_prompt();
        assert_eq!(output.as_deref(), Some("docker exec -it web bash"));
        assert_eq!(app.input_mode, InputMode::Search);
        assert!(app.argument_prompt.is_none());
    }

    #[test]
    fn argument_prompt_cancel_restores_previous_mode() {
        let catalog = Catalog::from_packs(vec![Pack {
            pack: PackMeta {
                id: "test-pack".to_string(),
                tool: "apt".to_string(),
                title: "apt".to_string(),
                version: "0.1.0".to_string(),
                source: "test".to_string(),
            },
            entries: vec![Entry {
                id: "install-package".to_string(),
                entry_type: EntryType::Command,
                title: "Install package".to_string(),
                keys: None,
                command: Some("apt install <package-name>".to_string()),
                description: "Install a package".to_string(),
                examples: Vec::new(),
                tags: Vec::new(),
                aliases: Vec::new(),
            }],
        }])
        .expect("catalog should build");

        let mut app = App::new(
            catalog,
            AppConfig::default(),
            UserState::default(),
            None,
            test_environment(),
        );
        app.input_mode = InputMode::Normal;
        assert!(app.selected_output().is_none());
        assert_eq!(app.input_mode, InputMode::Arguments);

        let result = app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(result.is_none());
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.argument_prompt.is_none());
    }
}
