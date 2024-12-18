//! A simple TUI app to collect pings on the command line

/// The "functional core" to the main module's "imperative shell"
mod app;
use app::App;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use std::{io, process::ExitCode};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> io::Result<ExitCode> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let res = run(terminal).await;
    ratatui::restore();
    res
}

/// Manage the lifecycle of the app
async fn run(mut terminal: DefaultTerminal) -> io::Result<ExitCode> {
    let mut app = App::new();

    let (tx, mut rx) = unbounded_channel();

    let event_tx = tx.clone();
    tokio::spawn(async move {
        let mut stream = EventStream::new();

        loop {
            match stream.next().await {
                Some(Err(err)) => {
                    // TODO: what's actually the right thing to do here?
                    eprintln!("error reading event: {err:?}");
                }
                Some(Ok(Event::Key(key_event))) => {
                    // TODO: log if we can't send at a trace level
                    let _ = event_tx.send(app::Action::Key(key_event));
                }
                Some(Ok(_)) => continue,
                None => break,
            }
        }
    });

    if let Some(effect) = app.init() {
        let init_result = tx.clone();
        tokio::spawn(async move {
            // TODO: what do we do if the channel is closed?
            let _ = init_result.send(effect.run().await);
        });
    }

    loop {
        terminal.draw(|frame| app.render(frame))?;

        match rx.recv().await {
            None => return Ok(ExitCode::SUCCESS),
            Some(action) => {
                if let Some(effect) = app.handle(&action) {
                    let effect_result = tx.clone();
                    tokio::spawn(async move {
                        // TODO: what do we do if the channel is closed?
                        let _ = effect_result.send(effect.run().await);
                    });
                }
            }
        }

        if let Some(code) = app.exit() {
            return Ok(code);
        }
    }
}
