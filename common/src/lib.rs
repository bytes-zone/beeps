#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

pub mod gmap;
pub mod hlc;
pub mod lww;
pub mod merge;
pub mod node_id;
pub mod replica;
pub mod scheduler;
pub mod state;

#[cfg(test)]
mod test;
