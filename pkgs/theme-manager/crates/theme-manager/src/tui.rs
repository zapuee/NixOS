use crate::runtime::{Runtime, TargetStatus};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap,
    },
};
use std::path::PathBuf;
use theme_core::ProfileSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Profiles,
    Targets,
}

struct Snapshot {
    profiles: Vec<ProfileSummary>,
    targets: Vec<TargetStatus>,
    active_profile: Option<String>,
    provider: String,
    config_path: PathBuf,
    vim_keys: bool,
}

struct App {
    requested_config: Option<PathBuf>,
    no_vim: bool,
    snapshot: Snapshot,
    pane: Pane,
    profile_index: usize,
    target_index: usize,
    message: String,
    message_is_error: bool,
}

impl App {
    fn load(requested_config: Option<PathBuf>, no_vim: bool) -> Result<Self> {
        let snapshot = load_snapshot(requested_config.as_deref(), no_vim)?;
        let profile_index = snapshot
            .profiles
            .iter()
            .position(|profile| profile.active)
            .unwrap_or(0);
        let message = format!(
            "Ready. {} navigation is active.",
            if snapshot.vim_keys { "Vim" } else { "Standard" }
        );

        Ok(Self {
            requested_config,
            no_vim,
            snapshot,
            pane: Pane::Profiles,
            profile_index,
            target_index: 0,
            message,
            message_is_error: false,
        })
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            let Event::Key(key) = event::read().context("failed to read terminal input")? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if self.handle_key(key)? {
                return Ok(());
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }

        match key.code {
            KeyCode::Esc => return Ok(true),
            KeyCode::Char('q') if key.modifiers.is_empty() => return Ok(true),
            KeyCode::Tab | KeyCode::BackTab => {
                self.pane = match self.pane {
                    Pane::Profiles => Pane::Targets,
                    Pane::Targets => Pane::Profiles,
                };
            }
            KeyCode::Left => self.pane = Pane::Profiles,
            KeyCode::Right => self.pane = Pane::Targets,
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Home => self.set_selection(0),
            KeyCode::End => self.set_selection(usize::MAX),
            KeyCode::Enter => self.activate(),
            KeyCode::Char(' ') if key.modifiers.is_empty() && self.pane == Pane::Targets => {
                self.toggle_target();
            }
            KeyCode::Char('d') if key.modifiers.is_empty() && self.pane == Pane::Profiles => {
                self.preview_profile();
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => self.check_configuration(),
            KeyCode::Char('r') if key.modifiers.is_empty() => self.refresh_with_message(),
            _ if self.snapshot.vim_keys => self.handle_vim_key(key),
            _ => {}
        }

        Ok(false)
    }

    fn handle_vim_key(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('h'), KeyModifiers::NONE) => self.pane = Pane::Profiles,
            (KeyCode::Char('l'), KeyModifiers::NONE) => self.pane = Pane::Targets,
            (KeyCode::Char('k'), KeyModifiers::NONE) => self.move_selection(-1),
            (KeyCode::Char('j'), KeyModifiers::NONE) => self.move_selection(1),
            (KeyCode::Char('g'), KeyModifiers::NONE) => self.set_selection(0),
            (KeyCode::Char('G'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.set_selection(usize::MAX);
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.move_selection(-self.vim_jump());
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.move_selection(self.vim_jump());
            }
            _ => {}
        }
    }

    fn vim_jump(&self) -> isize {
        (self.item_count() / 2).max(1) as isize
    }

    fn selection(&self) -> usize {
        match self.pane {
            Pane::Profiles => self.profile_index,
            Pane::Targets => self.target_index,
        }
    }

    fn item_count(&self) -> usize {
        match self.pane {
            Pane::Profiles => self.snapshot.profiles.len(),
            Pane::Targets => self.snapshot.targets.len(),
        }
    }

    fn set_selection(&mut self, requested: usize) {
        let last = self.item_count().saturating_sub(1);
        let selected = requested.min(last);
        match self.pane {
            Pane::Profiles => self.profile_index = selected,
            Pane::Targets => self.target_index = selected,
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let count = self.item_count();
        if count == 0 {
            return;
        }
        let current = self.selection();
        let selected = if delta < 0 {
            current.saturating_sub(delta.unsigned_abs())
        } else {
            current.saturating_add(delta as usize).min(count - 1)
        };
        self.set_selection(selected);
    }

    fn activate(&mut self) {
        match self.pane {
            Pane::Profiles => self.apply_profile(),
            Pane::Targets => self.toggle_target(),
        }
    }

    fn apply_profile(&mut self) {
        let Some(profile) = self.snapshot.profiles.get(self.profile_index) else {
            self.set_message("No profile is available to apply.", true);
            return;
        };
        let name = profile.name.clone();

        let result = (|| -> Result<String> {
            let runtime = Runtime::load(self.requested_config.as_deref())?;
            let report = runtime.engine.apply(&name, false)?;
            let changed = report
                .targets
                .iter()
                .map(|target| target.changed.len())
                .sum::<usize>();
            Ok(format!("Applied '{name}' ({changed} output(s) changed)."))
        })();
        self.finish_action(result);
    }

    fn preview_profile(&mut self) {
        let Some(profile) = self.snapshot.profiles.get(self.profile_index) else {
            self.set_message("No profile is available to preview.", true);
            return;
        };
        let name = profile.name.clone();
        let result = (|| -> Result<String> {
            let runtime = Runtime::load(self.requested_config.as_deref())?;
            let report = runtime.engine.apply(&name, true)?;
            let changed = report
                .targets
                .iter()
                .map(|target| target.changed.len())
                .sum::<usize>();
            Ok(format!(
                "Preview '{name}': {changed} output(s) would change; nothing was written."
            ))
        })();
        match result {
            Ok(message) => self.set_message(message, false),
            Err(err) => self.set_message(format!("{err:#}"), true),
        }
    }

    fn check_configuration(&mut self) {
        let result = crate::doctor::validation_summary(self.requested_config.as_deref());
        match result {
            Ok(message) => self.set_message(message, false),
            Err(err) => self.set_message(format!("{err:#}"), true),
        }
    }

    fn toggle_target(&mut self) {
        let Some(target) = self.snapshot.targets.get(self.target_index) else {
            self.set_message("No target is available to toggle.", true);
            return;
        };
        if !target.configured {
            self.set_message(
                format!(
                    "Target '{}' is not configured. Add [targets.{}] to config.toml first.",
                    target.target, target.target
                ),
                true,
            );
            return;
        }
        if !target.toggleable {
            self.set_message(
                format!(
                    "Target '{}' is disabled in config.toml. Set enabled = true to allow runtime control.",
                    target.target
                ),
                true,
            );
            return;
        }

        let target_name = target.target.clone();
        let enabled = !target.enabled;
        let result = (|| -> Result<String> {
            let runtime = Runtime::load(self.requested_config.as_deref())?;
            let report = runtime.set_target_enabled(&target_name, enabled)?;
            let paths = report.changed.len() + report.removed.len();
            Ok(format!(
                "{} target '{}' ({paths} output(s) updated).",
                if enabled { "Enabled" } else { "Disabled" },
                target_name
            ))
        })();
        self.finish_action(result);
    }

    fn finish_action(&mut self, result: Result<String>) {
        match result {
            Ok(message) => match self.refresh() {
                Ok(()) => self.set_message(message, false),
                Err(err) => self.set_message(
                    format!("Action succeeded, but refresh failed: {err:#}"),
                    true,
                ),
            },
            Err(err) => self.set_message(format!("{err:#}"), true),
        }
    }

    fn refresh(&mut self) -> Result<()> {
        self.snapshot = load_snapshot(self.requested_config.as_deref(), self.no_vim)?;
        self.profile_index = self
            .snapshot
            .profiles
            .iter()
            .position(|profile| profile.active)
            .unwrap_or(self.profile_index)
            .min(self.snapshot.profiles.len().saturating_sub(1));
        self.target_index = self
            .target_index
            .min(self.snapshot.targets.len().saturating_sub(1));
        Ok(())
    }

    fn refresh_with_message(&mut self) {
        match self.refresh() {
            Ok(()) => self.set_message("Reloaded configuration and state.", false),
            Err(err) => self.set_message(format!("{err:#}"), true),
        }
    }

    fn set_message(&mut self, message: impl Into<String>, is_error: bool) {
        self.message = message.into();
        self.message_is_error = is_error;
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(2),
            ])
            .split(area);

        self.render_header(frame, layout[0]);
        self.render_body(frame, layout[1]);
        self.render_message(frame, layout[2]);
        self.render_help(frame, layout[3]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let active = self.snapshot.active_profile.as_deref().unwrap_or("<none>");
        let title = Line::from(vec![
            Span::styled(
                " Theme Manager ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Profile: ", Style::default().fg(Color::DarkGray)),
            Span::raw(active),
            Span::raw("  "),
            Span::styled("Provider: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&self.snapshot.provider),
        ]);
        frame.render_widget(
            Paragraph::new(title)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Left),
            area,
        );
    }

    fn render_body(&self, frame: &mut Frame, area: Rect) {
        if area.width >= 76 {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
                .split(area);
            self.render_profiles(frame, columns[0]);
            self.render_targets(frame, columns[1]);
        } else {
            match self.pane {
                Pane::Profiles => self.render_profiles(frame, area),
                Pane::Targets => self.render_targets(frame, area),
            }
        }
    }

    fn render_profiles(&self, frame: &mut Frame, area: Rect) {
        let items = self.snapshot.profiles.iter().map(|profile| {
            let marker = if profile.active { "●" } else { " " };
            let style = if profile.active {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(format!(" {marker} {}", profile.name)).style(style)
        });
        let list = List::new(items)
            .block(self.pane_block("Profiles", Pane::Profiles))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" › ")
            .highlight_spacing(HighlightSpacing::Always);
        let mut state = ListState::default()
            .with_selected((!self.snapshot.profiles.is_empty()).then_some(self.profile_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_targets(&self, frame: &mut Frame, area: Rect) {
        let items = self.snapshot.targets.iter().map(|target| {
            let (marker, state, color) = if target.enabled {
                ("[on] ", "Enabled", Color::Green)
            } else if !target.configured {
                ("[--] ", "Not configured", Color::DarkGray)
            } else if !target.toggleable {
                ("[cfg]", "Disabled in config", Color::Yellow)
            } else {
                ("[off]", "Disabled", Color::Red)
            };
            let label = if target.target == target.adapter {
                target.target.clone()
            } else {
                format!("{} · {}", target.target, target.adapter)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {marker} "), Style::default().fg(color)),
                Span::styled(
                    format!("{label:<12}"),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(state, Style::default().fg(color)),
            ]))
        });
        let list = List::new(items)
            .block(self.pane_block("Components", Pane::Targets))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" › ")
            .highlight_spacing(HighlightSpacing::Always);
        let mut state = ListState::default()
            .with_selected((!self.snapshot.targets.is_empty()).then_some(self.target_index));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn pane_block(&self, title: &'static str, pane: Pane) -> Block<'static> {
        let active = self.pane == pane;
        Block::default()
            .title(format!(" {title} "))
            .title_style(if active {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            })
    }

    fn render_message(&self, frame: &mut Frame, area: Rect) {
        let color = if self.message_is_error {
            Color::Red
        } else {
            Color::Green
        };
        frame.render_widget(
            Paragraph::new(self.message.as_str())
                .style(Style::default().fg(color))
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(" Status ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                ),
            area,
        );
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let config = self.snapshot.config_path.display();
        let navigation = if self.snapshot.vim_keys {
            "Tab/h/l panes  ↑↓/j/k move  g/G ends  Ctrl-u/d jump"
        } else {
            "Tab/←→ panes  ↑↓ move  Home/End"
        };
        let actions = if area.width >= 120 {
            format!("Enter apply/toggle  d preview  c check  r reload  q quit  •  {config}")
        } else {
            "Enter apply/toggle  d preview  c check  r reload  q quit".to_string()
        };
        frame.render_widget(
            Paragraph::new(vec![Line::from(navigation), Line::from(actions)])
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            area,
        );
    }
}

fn load_snapshot(config: Option<&std::path::Path>, no_vim: bool) -> Result<Snapshot> {
    let runtime = Runtime::load(config).context("failed to load theme-manager")?;
    Ok(Snapshot {
        profiles: runtime.engine.list_profiles()?,
        targets: runtime.target_statuses(),
        active_profile: runtime.engine.active_profile()?,
        provider: runtime.engine.provider_id().to_string(),
        config_path: runtime.config_path,
        vim_keys: runtime.engine.config().tui.vim_keys && !no_vim,
    })
}

pub fn run(config: Option<PathBuf>, no_vim: bool) -> Result<()> {
    let mut app = App::load(config, no_vim)?;
    ratatui::run(|terminal| app.run(terminal)).context("failed to run terminal interface")
}

#[cfg(test)]
mod tests {
    use super::{App, Pane, Snapshot};
    use crate::runtime::TargetStatus;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;
    use theme_core::ProfileSummary;

    fn app() -> App {
        App {
            requested_config: None,
            no_vim: false,
            snapshot: Snapshot {
                profiles: vec![
                    ProfileSummary {
                        name: "glass".into(),
                        active: true,
                    },
                    ProfileSummary {
                        name: "paper".into(),
                        active: false,
                    },
                ],
                targets: vec![TargetStatus {
                    target: "foot".into(),
                    adapter: "foot".into(),
                    configured: true,
                    toggleable: true,
                    enabled: true,
                }],
                active_profile: Some("glass".into()),
                provider: "static".into(),
                config_path: PathBuf::from("/tmp/config.toml"),
                vim_keys: true,
            },
            pane: Pane::Profiles,
            profile_index: 0,
            target_index: 0,
            message: "Ready.".into(),
            message_is_error: false,
        }
    }

    #[test]
    fn renders_wide_and_narrow_terminals() {
        for (width, height) in [(120, 30), (60, 15)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            let app = app();
            terminal.draw(|frame| app.render(frame)).unwrap();
        }
    }

    #[test]
    fn selection_is_bounded_and_pane_specific() {
        let mut app = app();
        app.move_selection(10);
        assert_eq!(app.profile_index, 1);
        app.pane = Pane::Targets;
        app.move_selection(10);
        assert_eq!(app.target_index, 0);
        app.move_selection(-10);
        assert_eq!(app.target_index, 0);
    }

    #[test]
    fn vim_navigation_is_enabled_by_default() {
        let mut app = app();
        app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.profile_index, 1);

        app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.profile_index, 0);

        app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.pane, Pane::Targets);
        app.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.pane, Pane::Profiles);
    }

    #[test]
    fn disabling_vim_navigation_keeps_standard_navigation() {
        let mut app = app();
        app.snapshot.vim_keys = false;

        app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.profile_index, 0);

        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.profile_index, 1);

        app.pane = Pane::Targets;
        app.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.pane, Pane::Targets);
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.pane, Pane::Profiles);
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.pane, Pane::Targets);
    }

    #[test]
    fn control_d_is_navigation_not_profile_preview() {
        let mut app = app();
        app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL))
            .unwrap();
        assert_eq!(app.profile_index, 1);
        assert_eq!(app.message, "Ready.");
    }
}
