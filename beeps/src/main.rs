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
    time,
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

    // We expect side-effectful behaviors (that is, things like FS or network
    // access) to take place via async tasks. Once those tasks are done, we read
    // their results off of a channel. We keep track of outstanding effects so
    // we can exit cleanly.
    let (effect_tx, mut effect_rx) = unbounded_channel();
    let mut outstanding_effects = Vec::with_capacity(1);

    // Initialize the app, spawn a task to handle side effects, and render the
    // first frame. We could render before spawning for a slightly faster draw,
    // but defer it so that anything taken care of in `app.init` will reflect in
    // the first draw.
    outstanding_effects.push(spawn_effect_task(
        effect_tx.clone(),
        Arc::clone(&config),
        app.init(),
    ));
    terminal.draw(|frame| app.render(frame))?;

    let mut event_stream = EventStream::new();

    let mut ticks = time::interval(time::Duration::from_secs(10));

    // Start our event loop!
    loop {
        // First thing we do is wait for an event. This can be either external
        // input or the async result of a effect. This is an `Option<_>` because
        // we don't necessarily need to pay attention to every single piece of
        // external input.
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

            _ = ticks.tick() => {
                Some(app::Action::TimePassed)
            },

            effect_opt = effect_rx.recv() => {
                effect_opt
            }
        };

        // Once we have an action, we send it to `app.handle` to get any next
        // effects. (n.b. it feels slightly strange to use `and_then` for this
        // since it's mutating `app`, but it's way more compact!)
        if let Some(action) = next_action_opt {
            for effect in app.handle(action) {
                // If we have an effect, we handle it the same way we handled
                // init. As before, we keep track of any effects we get this way
                // in a list.
                outstanding_effects.push(spawn_effect_task(
                    effect_tx.clone(),
                    Arc::clone(&config),
                    effect,
                ));
            }
        }

        // Now that we handle the event, we re-render to display any changes the
        // app cares about.
        terminal.draw(|frame| app.render(frame))?;

        // If the message we just handled was from an outstanding effect, we
        // need to remove the completed `JoinHandle` from the list. This list
        // should never be too long (since we do this on every pass through the
        // event loop) so a full scan is fine.
        outstanding_effects.retain(|handle| !handle.is_finished());

        // Finally, if the app indicates that it should exit, we wait for all
        // outstanding effects to finish (e.g. so we can persist final state to
        // disk) before exiting the loop with the exit code from the app.
        if let Some(code) = app.should_exit() {
            for effect in outstanding_effects.drain(..) {
                // we should do something with the results here. No need to
                // ignore failures just because we're exiting.
                let _ = effect.await;
            }

            return Ok(code);
        }
    }
}

/// Spawn a task to run an effect and send the next action to the app.
fn spawn_effect_task(
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
