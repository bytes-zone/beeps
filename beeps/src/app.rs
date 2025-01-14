/// The form to register or log in
mod auth_form;

/// Information displayed above the main layout
mod popover;
use popover::Popover;

use crate::config::Config;
use beeps_core::{
    sync::{self, register, Client},
    NodeId, Replica,
};
use chrono::{DateTime, Local, Utc};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use layout::Flex;
use notify_rust::Notification;
use ratatui::{
    prelude::*,
    widgets::{
        Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
    Frame,
};
use std::{io, process::ExitCode};
use tokio::fs;
use tui_input::{backend::crossterm::EventHandler, Input};

/// The "functional core" of the app.
pub struct App {
    /// Status to display (visible at the bottom of the screen)
    status_line: Option<String>,

    /// The replica we're working with
    replica: Replica,

    /// State of the pings table
    table_state: TableState,

    /// Modal views above the table
    popover: Option<Popover>,

    /// The value that's currently copied, for copy/paste.
    copied: Option<String>,

    /// Sync client info
    client: Option<Client>,

    /// Exit code
    exiting: Option<ExitCode>,
}

impl App {
    /// Create a new instance of the app
    #[tracing::instrument]
    pub async fn init(config: &Config) -> Result<Self, Problem> {
        tracing::info!("initializing");

        let auth_path = config.data_dir().join("auth.json");
        let auth: Option<Client> = if fs::try_exists(&auth_path).await? {
            let data = fs::read(&auth_path).await?;
            Some(serde_json::from_slice(&data)?)
        } else {
            None
        };

        tracing::debug!(found = auth.is_some(), "tried to load client auth");

        let store_path = config.data_dir().join("store.json");
        if fs::try_exists(&store_path).await? {
            tracing::debug!(found = true, "tried to load store");

            let data = fs::read(&store_path).await?;
            let replica: Replica = serde_json::from_slice(&data)?;

            Ok(Self {
                status_line: Some("Loaded replica".to_string()),
                replica,
                client: auth,
                table_state: TableState::new().with_selected(0),
                popover: None,
                copied: None,
                exiting: None,
            })
        } else {
            tracing::debug!(found = false, "tried to load store");

            Ok(Self {
                status_line: None,
                replica: Replica::new(NodeId::random()),
                client: auth,
                table_state: TableState::new().with_selected(0),
                popover: None,
                copied: None,
                exiting: None,
            })
        }
    }

    /// Render the app's UI to the screen
    pub fn render(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
        let [body_area, status_area] = vertical.areas(frame.area());

        self.render_table(frame, body_area);
        self.render_status(frame, status_area);
        if let Some(popover) = &mut self.popover {
            popover.render(frame, body_area);
        }
    }

    /// Render the status line
    fn render_status(&self, frame: &mut Frame<'_>, status_area: Rect) {
        let status = Paragraph::new(self.status_line.as_deref().unwrap_or_default());
        frame.render_widget(status, status_area);
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

    /// Get the pings that we can display currently
    fn current_pings(&self) -> impl Iterator<Item = &DateTime<Utc>> {
        let now = Utc::now();

        self.replica.pings().rev().filter(move |ping| **ping <= now)
    }

    /// Get the currently-selected ping, if any
    fn selected_ping(&self) -> Option<&DateTime<Utc>> {
        self.table_state
            .selected()
            .and_then(|idx| self.current_pings().nth(idx))
    }

    /// Handle an `Action`, updating the app's state and producing some side effect(s)
    pub fn handle(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::SavedReplica => {
                tracing::info!("saved replica");
                self.status_line = Some("Saved replica".to_owned());

                vec![]
            }
            Action::SavedSyncClientAuth => {
                tracing::info!("saved auth");
                self.status_line = Some("Saved auth".to_owned());

                vec![]
            }
            Action::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return vec![];
                }

                self.handle_key(key)
            }
            Action::Problem(problem) => {
                tracing::info!(?problem, "displaying a problem");
                self.status_line = Some(problem.clone());

                vec![]
            }
            Action::TimePassed => {
                if self.replica.schedule_pings() {
                    tracing::debug!("handling new ping(s)");
                    vec![
                        Effect::NotifyAboutNewPing,
                        Effect::SaveReplica(self.replica.clone()),
                    ]
                } else {
                    vec![]
                }
            }
            Action::LoggedIn(jwt) => {
                if let Some(client) = &mut self.client {
                    tracing::debug!("setting JWT for existing client");
                    client.auth = Some(jwt);

                    vec![Effect::SaveSyncClientAuth(client.clone())]
                } else {
                    tracing::error!(
                        "got a JWT when I didn't have a client to go with it. What's going on?"
                    );
                    vec![]
                }
            }
        }
    }

    /// Handle a key press
    fn handle_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        let mut effects = Vec::new();

        match &mut self.popover {
            None => {
                match key.code {
                    KeyCode::Char('q') => {
                        self.exiting = Some(ExitCode::SUCCESS);

                        effects.push(Effect::SaveReplica(self.replica.clone()));
                        if let Some(client) = &self.client {
                            effects.push(Effect::SaveSyncClientAuth(client.clone()));
                        }
                    }
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

                            effects.push(Effect::SaveReplica(self.replica.clone()));
                        }
                    }
                    KeyCode::Char('e') | KeyCode::Enter => {
                        self.popover = self.selected_ping().map(|ping| {
                            Popover::Editing(
                                *ping,
                                Input::new(self.replica.get_tag(ping).cloned().unwrap_or_default()),
                            )
                        });
                    }
                    KeyCode::Char('?') | KeyCode::F(1) => self.popover = Some(Popover::Help),
                    KeyCode::Backspace | KeyCode::Delete => {
                        if let Some(idx) = self.table_state.selected() {
                            let ping = self.current_pings().nth(idx).unwrap();
                            self.replica.untag_ping(*ping);

                            effects.push(Effect::SaveReplica(self.replica.clone()));
                        }
                    }
                    KeyCode::Char('r') => {
                        self.popover = Some(Popover::Registering(auth_form::AuthForm::default()));
                    }
                    _ => (),
                };
            }
            Some(Popover::Help) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.popover = None,
                _ => (),
            },
            Some(Popover::Editing(ping, tag_input)) => match key.code {
                KeyCode::Enter => {
                    self.replica.tag_ping(*ping, tag_input.value().to_string());

                    self.popover = None;
                    effects.push(Effect::SaveReplica(self.replica.clone()));
                }
                KeyCode::Esc => self.popover = None,
                _ => {
                    tag_input.handle_event(&Event::Key(key));
                }
            },
            Some(Popover::Registering(auth)) => match key.code {
                KeyCode::Esc => self.popover = None,
                KeyCode::Enter => {
                    let finished = auth.finish();
                    let client = Client::new(finished.server);

                    effects.push(Effect::Register(
                        client.clone(),
                        register::Req {
                            email: finished.email,
                            password: finished.password,
                        },
                    ));

                    self.client = Some(client);
                    self.popover = None;
                }
                _ => auth.handle_event(key),
            },
        }

        effects
    }

    /// Let the TUI manager know whether we're all wrapped up and can exit.
    pub fn should_exit(&self) -> Option<ExitCode> {
        self.exiting
    }
}

/// Things that can happen to this app
#[derive(Debug)]
pub enum Action {
    /// We successfully saved the replica
    SavedReplica,

    /// We successfully saved the sync client
    SavedSyncClientAuth,

    /// We logged in successfully and got a new JWT
    LoggedIn(String),

    /// The user did something on the keyboard
    Key(KeyEvent),

    /// Something bad happened; display it to the user
    Problem(String),

    /// Some amount of time passed and we should do clock things
    TimePassed,
}

/// Connections to external services that effect use. We keep these around to
/// have some level of connection sharing for the app as a whole.
pub struct EffectConnections {
    /// an HTTP client with reqwest
    http: reqwest::Client,
}

impl EffectConnections {
    /// Get a new `EffectConnections`
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

/// Things that can happen as a result of user input. Side effects!
#[derive(Debug)]
pub enum Effect {
    /// Save replica to disk
    SaveReplica(Replica),

    /// Save sync client auth to disk
    SaveSyncClientAuth(Client),

    /// Notify that a new ping is available
    NotifyAboutNewPing,

    /// Register a new account on the server and log into it
    Register(Client, register::Req),
}

impl Effect {
    /// Perform the side-effectful portions of this effect, returning the next
    /// `Action` the application needs to handle
    pub async fn run(&self, conn: &EffectConnections, config: &Config) -> Option<Action> {
        match self.run_inner(conn, config).await {
            Ok(action) => action,
            Err(problem) => {
                tracing::error!(?problem, "problem running effect");
                Some(Action::Problem(problem.to_string()))
            }
        }
    }

    /// The actual implementation of `run`, but with a `Result` wrapper to make
    /// it more ergonomic to write.
    async fn run_inner(
        &self,
        conn: &EffectConnections,
        config: &Config,
    ) -> Result<Option<Action>, Problem> {
        match self {
            Self::SaveReplica(replica) => {
                tracing::debug!("saving replica");

                let base = config.data_dir();
                fs::create_dir_all(&base).await?;

                let store = base.join("store.json");

                let data = serde_json::to_vec(replica)?;
                fs::write(&store, &data).await?;

                Ok(Some(Action::SavedReplica))
            }

            Self::SaveSyncClientAuth(client) => {
                tracing::info!("saving client auth");

                let base = config.data_dir();
                fs::create_dir_all(&base).await?;

                let store = base.join("auth.json");

                let data = serde_json::to_vec(client)?;
                fs::write(&store, &data).await?;

                Ok(Some(Action::SavedSyncClientAuth))
            }

            Self::NotifyAboutNewPing => {
                tracing::debug!("notifying about new ping");

                // We don't care if the notification failed to show.
                let _ = Notification::new()
                    .summary("New ping!")
                    .body("What are you up to? Tag it!")
                    .show();

                Ok(None)
            }

            Self::Register(client, req) => {
                tracing::info!("registering");

                let resp = client.register(&conn.http, req).await?;

                Ok(Some(Action::LoggedIn(resp.jwt)))
            }
        }
    }
}

/// Problems that can happen while running an `Effect`.
#[derive(Debug, thiserror::Error)]
pub enum Problem {
    /// We had a problem writing to disk, for example with permissions or
    /// missing files.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    /// We had a problem loading or saving JSON.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// We had a problem communicating with the server, for example due to a bad
    /// URL or expired credentials.
    #[error("Problem communicating with the server: {0}")]
    Server(#[from] sync::Error),
}
