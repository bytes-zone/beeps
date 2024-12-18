use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::Paragraph, Frame};
use std::process::ExitCode;
use tokio::task::JoinHandle;

pub struct App {
    pub exit: Option<ExitCode>,
}

impl App {
    pub fn new() -> Self {
        Self { exit: None }
    }

    pub fn render(&self, frame: &mut Frame) {
        let greeting = Paragraph::new("Hello Ratatui! (press 'q' to quit)")
            .white()
            .on_blue();
        frame.render_widget(greeting, frame.area());
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
                eprintln!("key pressed: {key:#?}");
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
