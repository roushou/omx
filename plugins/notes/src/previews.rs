use super::{Editor, Settings};
use omega::config::Fields;
use omega::testing::{State, SurfaceHarness};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface::<Editor>("scratch", State::new())
        .surface_with::<Editor>("compact", || {
            SurfaceHarness::configured(
                &State::new(),
                &Settings {
                    rows: 4,
                    autosave_millis: 0,
                }
                .write(),
            )
        })
        .run()
        .unwrap();
}
