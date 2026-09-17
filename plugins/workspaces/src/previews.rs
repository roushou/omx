use super::{Indicator, Settings, fixtures::Fixture};
use omega::{
    config::Fields,
    testing::{State, SurfaceHarness, SystemTopic},
};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::at(1))
        .surface::<Indicator>("external-focus", Fixture::at(3))
        .surface::<Indicator>("empty", Fixture::empty())
        .surface::<Indicator>("unavailable", State::new().absent(SystemTopic::Workspaces))
        .surface_with::<Indicator>("vertical", || {
            SurfaceHarness::configured(
                &Fixture::at(3),
                &Settings {
                    vertical: true,
                    ..Settings::default()
                }
                .write(),
            )
        })
        .run()
        .unwrap();
}
