//! Visual cases for the viewer, backed by committed fixture images.

use super::{Message, Settings, Viewer};
use omega::config::Fields;
use omega::testing::{State, SurfaceHarness};
use omega_preview::Cases;

/// A deterministic cache directory shared by the cases.
fn cache() -> String {
    std::env::temp_dir()
        .join("omega-image-viewer-preview")
        .to_string_lossy()
        .into_owned()
}

/// Base settings pointed at the committed fixture folder.
fn gallery() -> Settings {
    Settings {
        path: format!("{}/fixtures", env!("CARGO_MANIFEST_DIR")),
        cache: cache(),
        width: 600,
        height: 560,
        thumbnail: 72,
        max_images: 500,
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface_with::<Viewer>("gallery", || {
            SurfaceHarness::configured(&State::new(), &gallery().write())
        })
        .surface_with::<Viewer>("single", || {
            SurfaceHarness::configured(
                &State::new(),
                &Settings {
                    path: format!("{}/fixtures/apple.png", env!("CARGO_MANIFEST_DIR")),
                    ..gallery()
                }
                .write(),
            )
        })
        .surface_with::<Viewer>("empty", || {
            SurfaceHarness::configured(
                &State::new(),
                &Settings {
                    path: String::new(),
                    ..gallery()
                }
                .write(),
            )
        })
        .surface_with::<Viewer>("zoom", || {
            let mut harness = SurfaceHarness::configured(&State::new(), &gallery().write())?;
            for _ in 0..4 {
                harness.send(Message::Zoom(1))?;
            }
            Ok(harness)
        })
        .run()
        .unwrap();
}
