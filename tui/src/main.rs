//! A simple TUI app to collect pings on the command line

/// The "functional core" to the main module's "imperative shell"
mod app;
use app::App;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use std::{io, process::ExitCode};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

#[tokio::main]
async fn main() -> io::Result<ExitCode> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let res = run(terminal).await;
    ratatui::restore();
    res
}

/// Handle a single effect from `App`, sending the result back to the app
async fn handle_effect(tx: UnboundedSender<app::Action>, eff: app::Effect) {
    match eff {
        app::Effect::None => {}
        app::Effect::Await(handle) => {
            tx.send(handle.await.expect("should not have panicked"))
                .expect("should not have blocked on unbounded channel");
        }
    }
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

    tokio::spawn(handle_effect(tx.clone(), app.init()));

    loop {
        terminal.draw(|frame| app.render(frame))?;

        match rx.recv().await {
            None => return Ok(ExitCode::SUCCESS),
            Some(action) => {
                tokio::spawn(handle_effect(tx.clone(), app.handle(&action)));
            }
        }

        if let Some(code) = app.exit() {
            return Ok(code);
        }
    }
}
