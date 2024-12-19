//! A simple TUI app to collect pings on the command line

/// The "functional core" to the main module's "imperative shell"
mod app;

/// Configuration and argument parsing
mod config;

use app::App;
use clap::Parser;
use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use std::{io, process::ExitCode, sync::Arc};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

#[tokio::main]
async fn main() -> io::Result<ExitCode> {
    let config = config::Config::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let res = run(terminal, Arc::new(config)).await;
    ratatui::restore();
    res
}

/// Manage the lifecycle of the app
async fn run(mut terminal: DefaultTerminal, config: Arc<config::Config>) -> io::Result<ExitCode> {
    let mut app = App::new();

    let (tx, mut rx) = unbounded_channel();

    let event_tx = tx.clone();
    tokio::spawn(async move {
        let mut stream = EventStream::new();

        loop {
            match stream.next().await {
                Some(Err(err)) => {
                    // TODO: log if we can't send at a trace level
                    let _ = event_tx.send(app::Action::Problem(err.to_string()));
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

    handle_effect(&tx, app.init(), config.clone());

    loop {
        terminal.draw(|frame| app.render(frame))?;

        match rx.recv().await {
            None => return Ok(ExitCode::SUCCESS),
            Some(action) => {
                handle_effect(&tx, app.handle(&action), config.clone());
            }
        }

        if let Some(code) = app.exit() {
            return Ok(code);
        }
    }
}

/// Spawn a new task to run an effect, and report it back to the stream.
fn handle_effect(
    tx: &UnboundedSender<app::Action>,
    effect_opt: Option<app::Effect>,
    config: Arc<config::Config>,
) {
    if let Some(effect) = effect_opt {
        let init_result = tx.clone();

        tokio::spawn(async move {
            let next_action = effect.run(config).await;

            // TODO: what do we do if the channel is closed? It probably means
            // we're shutting down and it's OK to drop messages, but we still
            // get the error.
            let _ = init_result.send(next_action);
        });
    }
}
