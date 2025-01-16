/// Things that can happen in the app
mod action;
pub use action::Action;

/// The form to register or log in
mod auth_form;

/// Side effects the app can do
pub mod effect;
pub use effect::{Effect, Problem};

/// Information displayed above the main layout
mod popover;
use popover::{AuthIntent, Popover};

use crate::config::Config;
use beeps_core::{
    sync::{login, register, Client},
    NodeId, Replica,
};
use chrono::{DateTime, Local, Utc};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use layout::Flex;
use ratatui::{
    prelude::*,
    widgets::{
        Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
    Frame,
};
use std::process::ExitCode;
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
            Action::LoggedIn(client) => {
                self.client = Some(client.clone());

                vec![Effect::SaveSyncClientAuth(client)]
            }
        }
    }

    /// Handle a key press
    #[expect(clippy::too_many_lines)]
    fn handle_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        let mut effects = Vec::new();

        match &mut self.popover {
            None => {
                match key.code {
                    KeyCode::Char('q') => effects.append(&mut self.quit()),
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
                        self.popover = Some(Popover::Authenticating(
                            auth_form::AuthForm::default(),
                            AuthIntent::Register,
                        ));
                    }
                    KeyCode::Char('l') => {
                        self.popover = Some(Popover::Authenticating(
                            auth_form::AuthForm::default(),
                            AuthIntent::LogIn,
                        ));
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
            Some(Popover::Authenticating(auth, intent)) => match key.code {
                KeyCode::Esc => self.popover = None,
                KeyCode::Enter => {
                    let finished = auth.finish();
                    let client = Client::new(finished.server);

                    match intent {
                        AuthIntent::LogIn => {
                            effects.push(Effect::LogIn(
                                client.clone(),
                                login::Req {
                                    email: finished.email,
                                    password: finished.password,
                                },
                            ));
                        }
                        AuthIntent::Register => {
                            effects.push(Effect::Register(
                                client.clone(),
                                register::Req {
                                    email: finished.email,
                                    password: finished.password,
                                },
                            ));
                        }
                    }

                    self.popover = None;
                }
                _ => auth.handle_event(key),
            },
        }

        effects
    }

    fn quit(&mut self) -> Vec<Effect> {
        self.exiting = Some(ExitCode::SUCCESS);

        let mut effects = Vec::with_capacity(2);
        effects.push(Effect::SaveReplica(self.replica.clone()));

        if let Some(client) = &self.client {
            effects.push(Effect::SaveSyncClientAuth(client.clone()));
        }

        effects
    }

    /// Let the TUI manager know whether we're all wrapped up and can exit.
    pub fn should_exit(&self) -> Option<ExitCode> {
        self.exiting
    }
}
