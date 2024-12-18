#![warn(
    clippy::pedantic,
    clippy::allow_attributes,
    clippy::absolute_paths,
    clippy::alloc_instead_of_core,
    clippy::decimal_literal_representation
)]
#![allow(clippy::must_use_candidate)]

use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    prelude::*,
    widgets::Paragraph,
    DefaultTerminal,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let res = run(terminal);
    ratatui::restore();
    res
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let greeting = Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                .white()
                .on_blue();
            frame.render_widget(greeting, frame.area());
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}
