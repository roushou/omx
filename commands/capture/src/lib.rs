//! Reusable screen-capture operations and their executable host.

mod commands;
mod host;

pub use commands::{OcrText, Record, Screenshot, ScreenshotFullscreen, StopRecord};
pub use host::Host;
