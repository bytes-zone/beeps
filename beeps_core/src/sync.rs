/// Things that can go wrong in the API
pub mod error;
pub use error::Error;

/// Register with the server
pub mod register;
pub use register::register;
