use common::{NodeId, Replica};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::Paragraph, Frame};
use std::{io, process::ExitCode, sync::Arc};
use tokio::fs;

use crate::config::Config;

/// The "functional core" of the app.
pub struct App {
    /// Status to display (visible at the bottom of the screen)
    status_line: Option<String>,

    /// Where the app is in its lifecycle
    state: AppState,
}

/// App lifecycle
#[derive(Debug)]
enum AppState {
    /// We haven't loaded anything yet
    Unloaded,

    /// We have loaded a replica from disk
    Loaded(Replica),

    /// We're done and want the following exit code after final effects
    Exiting(ExitCode),
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

        let greeting = Paragraph::new(format!("{:#?}", self.state))
            .white()
            .on_blue();

        let status = Paragraph::new(match &self.status_line {
            Some(line) => line,
            None => "All good!",
        });

        frame.render_widget(greeting, body_area);
        frame.render_widget(status, status_area);
    }

    /// Produce any side effects as needed to initialize the app.
    #[expect(clippy::unused_self)]
    pub fn init(&mut self) -> Effect {
        Effect::Load
    }

    /// Handle an `Action`, updating the app's state and producing some side effect(s)
    pub fn handle(&mut self, action: Action) -> Option<Effect> {
        match action {
            Action::LoadedReplica(replica) => {
                self.state = AppState::Loaded(replica);
                self.status_line = Some("Loaded replica".to_owned());

                None
            }
            Action::Key(key)
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') =>
            {
                self.state = AppState::Exiting(ExitCode::SUCCESS);

                None
            }
            Action::Key(key) => {
                self.status_line = Some(format!("Unknown key {key:?}"));

                None
            }
            Action::Problem(problem) => {
                self.status_line = Some(problem.clone());

                None
            }
        }
    }

    /// Let the TUI manager know whether we're all wrapped up and can exit.
    pub fn exit(&self) -> Option<ExitCode> {
        if let AppState::Exiting(code) = &self.state {
            Some(*code)
        } else {
            None
        }
    }
}

/// Things that can happen to this app
pub enum Action {
    /// We loaded replica data from disk
    LoadedReplica(Replica),

    /// The user did something on the keyboard
    Key(KeyEvent),

    /// Something bad happened; display it to the user
    Problem(String),
}

/// Things that can happen as a result of user input. Side effects!
pub enum Effect {
    /// Load replica state from disk
    Load,
}

impl Effect {
    pub async fn run(&self, config: Arc<Config>) -> Action {
        match self.run_inner(config).await {
            Ok(action) => action,
            Err(problem) => Action::Problem(problem.to_string()),
        }
    }

    async fn run_inner(&self, config: Arc<Config>) -> Result<Action, io::Error> {
        match self {
            Self::Load => {
                let base = config.data_dir();

                if !fs::try_exists(&base).await? {
                    fs::create_dir_all(&base).await?;
                }

                let store = base.join("store.json");

                if fs::try_exists(&store).await? {
                    let data = fs::read(&store).await?;
                    let replica: Replica = serde_json::from_slice(&data)?;

                    Ok(Action::LoadedReplica(replica))
                } else {
                    Ok(Action::LoadedReplica(Replica::new(NodeId::random())))
                }
            }
        }
    }
}
