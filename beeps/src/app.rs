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
    #[expect(clippy::cast_possible_truncation)]
    pub fn render(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
        let [body_area, status_area] = vertical.areas(frame.area());

        match &mut self.state {
            AppState::Unloaded => frame.render_widget(Paragraph::new("Loading…"), body_area),
            AppState::Loaded(loaded) => {
                let rows: Vec<Row> = loaded
                    .current_pings()
                    .map(|ping| {
                        Row::new(vec![
                            Cell::new(ping.with_timezone(&Local).to_rfc2822()),
                            match loaded.replica.get_tag(ping) {
                                Some(tag) => Cell::new(tag.clone()),
                                _ => Cell::new("<unknown>".to_string()).fg(Color::DarkGray),
                            },
                        ])
                    })
                    .collect();

                let num_rows = rows.len();

                let table = Table::new(rows, [Constraint::Min(31), Constraint::Min(9)])
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
                let mut scroll_state = ScrollbarState::new(num_rows)
                    .position(loaded.table_state.selected().unwrap_or(0));

                frame.render_stateful_widget(table, body_area, &mut loaded.table_state);
                frame.render_stateful_widget(
                    scroll,
                    body_area.inner(Margin::new(1, 1)),
                    &mut scroll_state,
                );

                // Editing popover
                if let Some((ping, tag_input)) = &loaded.editing {
                    let popup_vert = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
                    let popup_horiz =
                        Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);

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
            AppState::Exiting(_) => frame.render_widget(Paragraph::new("Exiting…"), body_area),
        };

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
    #[expect(clippy::too_many_lines)]
    pub fn handle(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::LoadedReplica(replica) => {
                self.state = AppState::Loaded(Loaded {
                    replica,
                    table_state: TableState::new().with_selected(0),
                    editing: None,
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

                if self.state.is_editing() {
                    self.state
                        .map_loaded_mut(|loaded| match loaded.editing {
                            Some(ref mut editing) => match key.code {
                                KeyCode::Enter => {
                                    let (ping, tag_input) = editing;
                                    loaded
                                        .replica
                                        .tag_ping(*ping, tag_input.value().to_string());

                                    loaded.editing = None;

                                    vec![Effect::Save(loaded.replica.clone())]
                                }
                                KeyCode::Esc => {
                                    loaded.editing = None;

                                    vec![]
                                }
                                _ => {
                                    editing.1.handle_event(&Event::Key(key));

                                    vec![]
                                }
                            },

                            None => vec![],
                        })
                        .unwrap_or_default()
                } else {
                    match key.code {
                        KeyCode::Char('q') => {
                            let pre_quit_state =
                                mem::replace(&mut self.state, AppState::Exiting(ExitCode::SUCCESS));

                            match pre_quit_state {
                                AppState::Loaded(Loaded { replica, .. }) => {
                                    vec![Effect::Save(replica)]
                                }
                                _ => vec![],
                            }
                        }
                        KeyCode::Char('j') => {
                            self.state.map_loaded_mut(|loaded| {
                                loaded.table_state.select_next();
                            });

                            vec![]
                        }
                        KeyCode::Char('k') => {
                            self.state.map_loaded_mut(|loaded| {
                                loaded.table_state.select_previous();
                            });

                            vec![]
                        }
                        KeyCode::Enter | KeyCode::Char('e') => {
                            self.state.map_loaded_mut(|loaded| {
                                loaded.editing = loaded
                                    .table_state
                                    .selected()
                                    .and_then(|idx| loaded.current_pings().nth(idx))
                                    .map(|ping| {
                                        (
                                            *ping,
                                            Input::new(
                                                loaded
                                                    .replica
                                                    .get_tag(ping)
                                                    .cloned()
                                                    .unwrap_or_default(),
                                            ),
                                        )
                                    });
                            });

                            vec![]
                        }
                        _ => {
                            self.status_line = Some(format!("Unknown key {key:?}"));

                            vec![]
                        }
                    }
                }
            }
            Action::Problem(problem) => {
                self.status_line = Some(problem.clone());

                vec![]
            }
            Action::TimePassed => self
                .state
                .map_loaded_mut(|loaded| {
                    if loaded.replica.schedule_pings() {
                        vec![
                            Effect::NotifyAboutNewPing,
                            Effect::Save(loaded.replica.clone()),
                        ]
                    } else {
                        vec![]
                    }
                })
                .unwrap_or_default(),
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
    /// Do something to the inner loaded state, if the app is indeed in that state.
    fn map_loaded_mut<T>(&mut self, edit: impl Fn(&mut Loaded) -> T) -> Option<T> {
        if let Self::Loaded(loaded) = self {
            Some(edit(loaded))
        } else {
            None
        }
    }

    /// Convenience method to check if we're editing text
    fn is_editing(&self) -> bool {
        if let Self::Loaded(loaded) = self {
            loaded.editing.is_some()
        } else {
            false
        }
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
}

impl Loaded {
    /// Get the pings that we can display currently
    fn current_pings(&self) -> impl Iterator<Item = &DateTime<Utc>> {
        let now = Utc::now();

        self.replica.pings().rev().filter(move |ping| **ping <= now)
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
