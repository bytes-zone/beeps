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
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    task::JoinHandle,
};

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

    let (effect_tx, mut effect_rx) = unbounded_channel();
    let mut outstanding_effects = Vec::with_capacity(1);

    // Initialize the app
    outstanding_effects.push(handle_effect(
        effect_tx.clone(),
        Arc::clone(&config),
        app.init(),
    ));

    let mut event_stream = EventStream::new();

    loop {
        terminal.draw(|frame| app.render(frame))?;

        let next_action_opt = tokio::select! {
            event_opt = event_stream.next() => {
                match event_opt {
                    Some(Ok(Event::Key(key_event))) => {
                        Some(app::Action::Key(key_event))
                    }
                    Some(Err(err)) => {
                        Some(app::Action::Problem(err.to_string()))
                    }
                    _ => None,
                }
            },

            effect_opt = effect_rx.recv() => {
                effect_opt
            }
        };

        if let Some(effect) = next_action_opt.and_then(|action| app.handle(action)) {
            outstanding_effects.push(handle_effect(
                effect_tx.clone(),
                Arc::clone(&config),
                effect,
            ));
        }

        // clear out any finished effects
        outstanding_effects.retain(|handle| !handle.is_finished());

        if let Some(code) = app.exit() {
            for effect in outstanding_effects.drain(..) {
                // we should do something with the results here. No need to
                // ignore failures just because we're exiting.
                let _ = effect.await;
            }

            return Ok(code);
        }
    }
}

fn handle_effect(
    effect_tx: UnboundedSender<app::Action>,
    config: Arc<config::Config>,
    effect: app::Effect,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let next_action = effect.run(config).await;

        // TODO: what do we do if the channel is closed? It probably means
        // we're shutting down and it's OK to drop messages, but we still
        // get the error.
        let _ = effect_tx.send(next_action);
    })
}
