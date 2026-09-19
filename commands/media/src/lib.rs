//! Reusable media operations and their executable host.

mod commands;
mod host;

pub use commands::{Next, Pause, Play, Previous};
pub use host::Host;
