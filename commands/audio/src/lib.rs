//! Reusable audio operations and their executable host.

mod commands;
mod host;

pub use commands::{SetAudible, SetVolume};
pub use host::Host;
