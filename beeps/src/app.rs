use beeps_core::{NodeId, Replica};
use chrono::{DateTime, Local, Utc};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use layout::Flex;
use notify_rust::Notification;
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    Frame,
};
use std::{io, mem, process::ExitCode, sync::Arc};
use tokio::fs;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::config::Config;

/// The "functional core" of the app.
pub struct App {
    /// Status to display (visible at the bottom of the screen)
    status_line: Option<String>,

    /// Where the app is in its lifecycle
    state: AppState,
}

impl App {
    /// Create a new instance of the app
    pub fn new() -> Self {
        Self {
            status_line: None,
            state: AppState::Unloaded,
        }
    }

    /// Render the app's UI to the screen
    pub fn render(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
        let [body_area, status_area] = vertical.areas(frame.area());

        self.state.render(body_area, frame);

        let status = Paragraph::new(match &self.status_line {
            Some(line) => line,
            None => "All good!",
        });

        frame.render_widget(status, status_area);
    }

    /// Produce any side effects as needed to initialize the app.
    #[expect(clippy::unused_self)]
    pub fn init(&self) -> Effect {
        Effect::Load
    }

    /// Handle an `Action`, updating the app's state and producing some side effect(s)
    pub fn handle(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::LoadedReplica(replica) => {
                self.state = AppState::Loaded(Loaded {
                    replica,
                    table_state: TableState::new().with_selected(0),
                    editing: None,
                    copied: None,
                });
                self.status_line = Some("Loaded replica".to_owned());

                vec![]
            }
            Action::Saved => {
                self.status_line = Some("Saved replica".to_owned());

                vec![]
            }
            Action::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return vec![];
                }

                self.state.handle_key(key)
            }
            Action::Problem(problem) => {
                self.status_line = Some(problem.clone());

                vec![]
            }
            Action::TimePassed => self.state.handle_time_passed(),
        }
    }

    /// Let the TUI manager know whether we're all wrapped up and can exit.
    pub fn should_exit(&self) -> Option<ExitCode> {
        if let AppState::Exiting(code) = &self.state {
            Some(*code)
        } else {
            None
        }
    }
}

/// App lifecycle
#[derive(Debug)]
enum AppState {
    /// We haven't loaded anything yet
    Unloaded,

    /// We have loaded a replica from disk
    Loaded(Loaded),

    /// We're done and want the following exit code after final effects
    Exiting(ExitCode),
}

impl AppState {
    /// Handle a key press
    fn handle_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        match self {
            Self::Unloaded => self.handle_key_unloaded(key),
            Self::Loaded(loaded) => {
                let (effects, exit_code) = loaded.handle_key(key);
                exit_code.map(|code| self.quit(code));

                effects
            }
            Self::Exiting(_) => vec![],
        }
    }

    /// Handle a key press when we're in the unloaded state
    fn handle_key_unloaded(&mut self, key: KeyEvent) -> Vec<Effect> {
        if key.code == KeyCode::Char('q') {
            self.quit(ExitCode::SUCCESS)
        } else {
            vec![]
        }
    }

    /// Handle time passing
    fn handle_time_passed(&mut self) -> Vec<Effect> {
        match self {
            AppState::Loaded(loaded) => loaded.handle_time_passed(),
            _ => vec![],
        }
    }

    /// Start cleaning up and move into the exiting state.
    fn quit(&mut self, exit_code: ExitCode) -> Vec<Effect> {
        let pre_quit_state = mem::replace(self, Self::Exiting(exit_code));

        match pre_quit_state {
            AppState::Loaded(Loaded { replica, .. }) => {
                vec![Effect::Save(replica)]
            }
            _ => vec![],
        }
    }

    /// Render the app state
    fn render(&mut self, body_area: Rect, frame: &mut Frame<'_>) {
        match self {
            AppState::Unloaded => frame.render_widget(Paragraph::new("Loading…"), body_area),
            AppState::Loaded(loaded) => loaded.render(body_area, frame),
            AppState::Exiting(_) => frame.render_widget(Paragraph::new("Exiting…"), body_area),
        };
    }
}

/// State when we have successfully loaded and are running
#[derive(Debug)]
struct Loaded {
    /// The replica we're working with
    replica: Replica,

    /// State of the pings table
    table_state: TableState,

    /// What we're editing, and the current value.
    editing: Option<(DateTime<Utc>, Input)>,

    /// The value that's currently copied, for copy/paste.
    copied: Option<String>,
}

impl Loaded {
    /// Get the pings that we can display currently
    fn current_pings(&self) -> impl Iterator<Item = &DateTime<Utc>> {
        let now = Utc::now();

        self.replica.pings().rev().filter(move |ping| **ping <= now)
    }

    /// Handle a key press
    fn handle_key(&mut self, key: KeyEvent) -> (Vec<Effect>, Option<ExitCode>) {
        let mut effects = Vec::new();
        let mut exit_code = None;

        match &mut self.editing {
            None => {
                match key.code {
                    KeyCode::Char('q') => exit_code = Some(ExitCode::SUCCESS),
                    KeyCode::Char('j') | KeyCode::Down => self.table_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.table_state.select_previous(),
                    KeyCode::Char('c') => {
                        self.copied = self
                            .selected_ping()
                            .and_then(|ping| self.replica.get_tag(ping).cloned());
                    }
                    KeyCode::Char('v') => {
                        if let Some((ping, tag)) = self.selected_ping().zip(self.copied.as_ref()) {
                            self.replica.tag_ping(*ping, tag.clone());

                            effects.push(Effect::Save(self.replica.clone()));
                        }
                    }
                    KeyCode::Char('e') | KeyCode::Enter => {
                        self.editing = self.selected_ping().map(|ping| {
                            (
                                *ping,
                                Input::new(self.replica.get_tag(ping).cloned().unwrap_or_default()),
                            )
                        });
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        if let Some(idx) = self.table_state.selected() {
                            let ping = self.current_pings().nth(idx).unwrap();
                            self.replica.untag_ping(*ping);

                            effects.push(Effect::Save(self.replica.clone()));
                        }
                    }
                    _ => (),
                };
            }
            Some(editing) => match key.code {
                KeyCode::Enter => {
                    let (ping, tag_input) = editing;
                    self.replica.tag_ping(*ping, tag_input.value().to_string());

                    self.editing = None;
                    effects.push(Effect::Save(self.replica.clone()));
                }
                KeyCode::Esc => self.editing = None,
                _ => {
                    editing.1.handle_event(&Event::Key(key));
                }
            },
        }

        (effects, exit_code)
    }

    /// Get the currently-selected ping, if any
    fn selected_ping(&self) -> Option<&DateTime<Utc>> {
        self.table_state
            .selected()
            .and_then(|idx| self.current_pings().nth(idx))
    }

    /// Handle time passing
    fn handle_time_passed(&mut self) -> Vec<Effect> {
        if self.replica.schedule_pings() {
            vec![
                Effect::NotifyAboutNewPing,
                Effect::Save(self.replica.clone()),
            ]
        } else {
            vec![]
        }
    }

    /// Render the table and editing popover
    fn render(&mut self, body_area: Rect, frame: &mut Frame<'_>) {
        self.render_table(frame, body_area);
        self.render_editing_popover(body_area, frame);
    }

    /// Render the table of pings
    fn render_table(&mut self, frame: &mut Frame<'_>, body_area: Rect) {
        let rows: Vec<Row> = self
            .current_pings()
            .map(|ping| {
                Row::new(vec![
                    Cell::new(
                        ping.with_timezone(&Local)
                            .format("%a, %b %-d, %-I:%M %p")
                            .to_string(),
                    ),
                    match self.replica.get_tag(ping) {
                        Some(tag) => Cell::new(tag.clone()),
                        _ => Cell::new("<unknown>".to_string()).fg(Color::DarkGray),
                    },
                ])
            })
            .collect();

        let num_rows = rows.len();

        let table = Table::new(rows, [Constraint::Min(21), Constraint::Min(9)])
            .header(
                Row::new(["Ping", "Tag"])
                    .bg(Color::DarkGray)
                    .fg(Color::White),
            )
            .column_spacing(2)
            .highlight_symbol("● ")
            .row_highlight_style(Style::new().add_modifier(Modifier::BOLD))
            .flex(Flex::Legacy);

        let scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .thumb_symbol("┃")
            .thumb_style(Style::new().fg(Color::White))
            .track_symbol(Some("┆"))
            .track_style(Style::new().fg(Color::Gray));

        let mut scroll_state =
            ScrollbarState::new(num_rows).position(self.table_state.selected().unwrap_or(0));

        frame.render_stateful_widget(table, body_area, &mut self.table_state);
        frame.render_stateful_widget(
            scroll,
            body_area.inner(Margin::new(1, 1)),
            &mut scroll_state,
        );
    }

    /// Render the editing popover
    #[expect(clippy::cast_possible_truncation)]
    fn render_editing_popover(&mut self, body_area: Rect, frame: &mut Frame<'_>) {
        if let Some((ping, tag_input)) = &self.editing {
            let popup_vert = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
            let popup_horiz = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);

            let [popup_area] = popup_vert.areas(body_area);
            let [popup_area] = popup_horiz.areas(popup_area);

            let width = popup_area.width - 2 - 1; // -2 for the border, -1 for the cursor

            let input_scroll = tag_input.visual_scroll(width as usize);

            let popup = Paragraph::new(tag_input.value())
                .scroll((0, input_scroll as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Edit tag for {}", ping.to_rfc2822())),
                )
                .style(Style::default().fg(Color::Blue));

            frame.render_widget(Clear, popup_area);
            frame.render_widget(popup, popup_area);

            frame.set_cursor_position((
                popup_area.x
                            + (tag_input.visual_cursor().max(input_scroll) - input_scroll) as u16 // current end of text
                            + 1, // just past the end of the text
                popup_area.y + 1, // +1 row for the border/title
            ));
        }
    }
}

/// Things that can happen to this app
#[derive(Debug)]
pub enum Action {
    /// We loaded replica data from disk
    LoadedReplica(Replica),

    /// We successfully saved the replica
    Saved,

    /// The user did something on the keyboard
    Key(KeyEvent),

    /// Something bad happened; display it to the user
    Problem(String),

    /// Some amount of time passed and we should do clock things
    TimePassed,
}

/// Things that can happen as a result of user input. Side effects!
#[derive(Debug)]
pub enum Effect {
    /// Load replica state from disk
    Load,

    /// Save replica to disk
    Save(Replica),

    /// Notify that a new ping is available
    NotifyAboutNewPing,
}

impl Effect {
    /// Perform the side-effectful portions of this effect, returning the next
    /// `Action` the application needs to handle
    pub async fn run(&self, config: Arc<Config>) -> Option<Action> {
        match self.run_inner(config).await {
            Ok(action) => action,
            Err(problem) => Some(Action::Problem(problem.to_string())),
        }
    }

    /// The actual implementation of `run`, but with a `Result` wrapper to make
    /// it more ergonomic to write.
    async fn run_inner(&self, config: Arc<Config>) -> Result<Option<Action>, io::Error> {
        match self {
            Self::Load => {
                let store = config.data_dir().join("store.json");

                if fs::try_exists(&store).await? {
                    let data = fs::read(&store).await?;
                    let replica: Replica = serde_json::from_slice(&data)?;

                    Ok(Some(Action::LoadedReplica(replica)))
                } else {
                    Ok(Some(Action::LoadedReplica(Replica::new(NodeId::random()))))
                }
            }

            Self::Save(replica) => {
                let base = config.data_dir();
                fs::create_dir_all(&base).await?;

                let store = base.join("store.json");

                let data = serde_json::to_vec(replica)?;
                fs::write(&store, &data).await?;

                Ok(Some(Action::Saved))
            }

            Self::NotifyAboutNewPing => {
                // We don't care if the notification failed to show.
                let _ = Notification::new()
                    .summary("New ping!")
                    .body("What are you up to? Tag it!")
                    .show();

                Ok(None)
            }
        }
    }
}
