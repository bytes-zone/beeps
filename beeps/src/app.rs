use beeps_core::{Lww, NodeId, Replica};
use chrono::{Local, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Row, Table},
    Frame,
};
use std::{io, mem, process::ExitCode, sync::Arc};
use tokio::fs;

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
    pub fn render(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
        let [body_area, status_area] = vertical.areas(frame.area());

        match &self.state {
            AppState::Unloaded => frame.render_widget(Paragraph::new("Loading…"), body_area),
            AppState::Loaded(Loaded { replica }) => {
                let state = replica.state();

                let now = Utc::now();

                let rows: Vec<Row> = state
                    .pings
                    .iter()
                    .filter(|ping| **ping <= now)
                    .map(|ping| {
                        Row::new(vec![
                            ping.with_timezone(&Local).to_rfc2822(),
                            match state.tags.get(ping).map(Lww::value) {
                                Some(tag) => tag.clone(),
                                _ => "<unknown>".to_string(),
                            },
                        ])
                    })
                    .collect();

                let table = Table::new(rows, [Constraint::Min(31), Constraint::Min(9)])
                    .header(
                        Row::new(vec!["Ping".to_string(), "Tag".to_string()])
                            .bg(Color::DarkGray)
                            .fg(Color::White),
                    )
                    .column_spacing(1);
                frame.render_widget(table, body_area);
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
    pub fn handle(&mut self, action: Action) -> Option<Effect> {
        match action {
            Action::LoadedReplica(replica) => {
                self.state = AppState::Loaded(Loaded { replica });
                self.status_line = Some("Loaded replica".to_owned());

                None
            }
            Action::Saved => {
                self.status_line = Some("Saved replica".to_owned());

                None
            }
            Action::Key(key)
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') =>
            {
                let pre_quit_state =
                    mem::replace(&mut self.state, AppState::Exiting(ExitCode::SUCCESS));

                match pre_quit_state {
                    AppState::Loaded(Loaded { replica }) => Some(Effect::Save(replica)),
                    _ => None,
                }
            }
            Action::Key(key) => {
                self.status_line = Some(format!("Unknown key {key:?}"));

                None
            }
            Action::Problem(problem) => {
                self.status_line = Some(problem.clone());

                None
            }
            Action::TimePassed => self
                .state
                .mut_loaded(|loaded| {
                    if loaded.replica.schedule_pings() {
                        Some(Effect::Save(loaded.replica.clone()))
                    } else {
                        None
                    }
                })
                .flatten(),
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

/// State when we have successfully loaded and are running
#[derive(Debug)]
struct Loaded {
    /// The replica we're working with
    replica: Replica,
}

impl AppState {
    fn mut_loaded<T>(&mut self, edit: fn(&mut Loaded) -> T) -> Option<T> {
        if let Self::Loaded(loaded) = self {
            Some(edit(loaded))
        } else {
            None
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
}

impl Effect {
    /// Perform the side-effectful portions of this effect, returning the next
    /// `Action` the application needs to handle
    pub async fn run(&self, config: Arc<Config>) -> Action {
        match self.run_inner(config).await {
            Ok(action) => action,
            Err(problem) => Action::Problem(problem.to_string()),
        }
    }

    /// The actual implementation of `run`, but with a `Result` wrapper to make
    /// it more ergonomic to write.
    async fn run_inner(&self, config: Arc<Config>) -> Result<Action, io::Error> {
        match self {
            Self::Load => {
                let store = config.data_dir().join("store.json");

                if fs::try_exists(&store).await? {
                    let data = fs::read(&store).await?;
                    let replica: Replica = serde_json::from_slice(&data)?;

                    Ok(Action::LoadedReplica(replica))
                } else {
                    Ok(Action::LoadedReplica(Replica::new(NodeId::random())))
                }
            }

            Self::Save(replica) => {
                let base = config.data_dir();
                fs::create_dir_all(&base).await?;

                let store = base.join("store.json");

                let data = serde_json::to_vec(replica)?;
                fs::write(&store, &data).await?;

                Ok(Action::Saved)
            }
        }
    }
}
