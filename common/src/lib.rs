#![warn(
    clippy::pedantic,
    clippy::allow_attributes,
    clippy::absolute_paths,
    clippy::alloc_instead_of_core,
    clippy::decimal_literal_representation
)]
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
