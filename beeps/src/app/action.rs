use beeps_core::sync::Client;
use crossterm::event::KeyEvent;

/// Things that can happen to this app
#[derive(Debug)]
pub enum Action {
    /// We successfully saved the replica
    SavedReplica,

    /// We successfully saved the sync client
    SavedSyncClientAuth,

    /// We logged in successfully and got a new JWT
    LoggedIn(Client),

    /// The user did something on the keyboard
    Key(KeyEvent),

    /// Something bad happened; display it to the user
    Problem(String),

    /// Some amount of time passed and we should do clock things
    TimePassed,
}
