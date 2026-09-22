//! Browse a folder of images as a full-window viewer.
//!
//! The plugin lists the images under a configured path, decodes and orients
//! them in Rust, and writes bounded display and thumbnail derivatives to a
//! cache. The main picture fills the window; a strip of nearby thumbnails
//! re-centres around the selection.
//!
//! Open it from a compositor binding:
//!
//! ```sh
//! omega present image-viewer gallery --width 1200 --height 800 \
//!   --config '{"path":"/home/me/Pictures"}'
//! ```
//!
//! Arrow keys and `h`/`l` move between images, Home and End jump to the ends,
//! `r` refreshes the folder, and Escape closes the window.

mod cache;
mod catalog;
mod derive;
mod viewer;

pub use cache::Cache;
pub use catalog::{Catalog, ImageFile};
pub use derive::{Deriver, Prepared, Thumbnail};
pub use viewer::{Message, Model, Settings, Viewer};

/// Register the image viewer surface.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Viewer)
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
