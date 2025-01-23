//! Common code across all beeps clients (TUI, WASM in the browser)

/// The state that gets synced between nodes.
pub mod document;
pub use document::Document;

/// A grow-only map (G-Map.) Values must be mergeable.
pub mod gmap;
pub use gmap::GMap;

/// A grow-only set (G-Set.)
pub mod gset;
pub use gset::GSet;

/// A Hybrid Logical clock (HLC)
pub mod hlc;
pub use hlc::Hlc;

/// A Last-Write-Wins (LWW) register.
pub mod lww;
pub use lww::Lww;

/// The interface all CRDTs must implement to merge.
pub mod merge;

/// A node ID.
pub mod node_id;
pub use node_id::NodeId;

/// A replica (that is, state + node ID)
pub mod replica;
pub use replica::Replica;

/// Sync with the sync server.
pub mod sync;

/// Scheduling pings
pub mod scheduler;
pub use scheduler::Scheduler;

#[cfg(test)]
mod test;
