use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::Paragraph, Frame};
use std::{error::Error, process::ExitCode, sync::Arc};

use crate::config::Config;

/// The "functional core" of the app.
pub struct App {
    /// Status to display (visible at the bottom of the screen)
    pub status_line: Option<String>,

    /// Exit code to return to the shell when we're done. If this is `Some`, we'll exit.
    pub exit: Option<ExitCode>,
}

impl App {
    /// Create a new instance of the app
    pub fn new() -> Self {
        Self {
            exit: None,
            status_line: None,
        }
    }

    /// Render the app's UI to the screen
    pub fn render(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
        let [body_area, status_area] = vertical.areas(frame.area());

        let greeting = Paragraph::new("Hello Ratatui! (press 'q' to quit)")
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
    pub fn init(&mut self) -> Option<Effect> {
        Some(Effect::Load)
    }

    /// Handle an `Action`, updating the app's state and producing some side effect(s)
    pub fn handle(&mut self, action: &Action) -> Option<Effect> {
        match action {
            Action::Key(key)
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') =>
            {
                self.exit = Some(ExitCode::SUCCESS);
            }
            Action::Key(key) => {
                self.status_line = Some(format!("Unknown key {key:?}"));
            }
            Action::Problem(problem) => {
                self.status_line = Some(problem.clone());
            }
        }

        None
    }

    /// Let the TUI manager know whether we're all wrapped up and can exit.
    pub fn exit(&self) -> Option<ExitCode> {
        self.exit
    }
}

/// Things that can happen to this app
pub enum Action {
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
        match self {
            Self::Load => Action::Problem("Load is unimplemented".to_string()),
        }
    }
}
