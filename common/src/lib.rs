#![warn(
    missing_docs,
    clippy::pedantic,
    clippy::allow_attributes,
    clippy::absolute_paths,
    clippy::alloc_instead_of_core,
    clippy::decimal_literal_representation,
    clippy::missing_docs_in_private_items
)]
#![allow(clippy::must_use_candidate)]

//! Common code across all beeps clients (TUI, WASM in the browser)

/// A grow-only map (G-Map.) Values must be mergeable.
pub mod gmap;

/// A grow-only set (G-Set.)
pub mod gset;

/// A Hybrid Logical clock (HLC)
pub mod hlc;

/// A Last-Write-Wins (LWW) register.
pub mod lww;

/// The interface all CRDTs must implement to merge.
pub mod merge;

/// A node ID.
pub mod node_id;

/// A replica (that is, state + node ID)
pub mod replica;

/// The state that gets synced between nodes.
pub mod state;

/// Scheduling pings
pub mod scheduler;

#[cfg(test)]
mod test;
