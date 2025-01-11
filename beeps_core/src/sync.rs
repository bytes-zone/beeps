/// A client to interact with the API
pub mod client;
pub use client::Client;

/// Things that can go wrong in the API
pub mod error;
pub use error::Error;

/// Register with the server
pub mod register;
