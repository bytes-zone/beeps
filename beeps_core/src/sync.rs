/// A client to interact with the API
pub mod client;
pub use client::Client;

/// Documents associated with the user's account
pub mod documents;

/// Things that can go wrong in the API
pub mod error;
pub use error::Error;

/// Log into the server
pub mod login;

/// Register with the server
pub mod register;

/// Check auth
pub mod whoami;
