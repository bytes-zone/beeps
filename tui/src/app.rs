use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::Paragraph, Frame};
use std::process::ExitCode;
use tokio::task::JoinHandle;

pub struct App {
    pub exit: Option<ExitCode>,
    pub status_line: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: None,
            status_line: None,
        }
    }

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

    pub fn init(&mut self) -> Effect {
        Effect::None
    }

    pub fn handle(&mut self, action: Action) -> Effect {
        match action {
            Action::Key(key)
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') =>
            {
                self.exit = Some(ExitCode::SUCCESS);
            }
            Action::Key(key) => {
                self.status_line = Some(format!("Unknown key {:?}", key));
            }
        }

        Effect::None
    }

    pub fn exit(&self) -> Option<ExitCode> {
        return self.exit;
    }
}

pub enum Effect {
    None,
    Await(JoinHandle<Action>),
}

pub enum Action {
    Key(KeyEvent),
}
