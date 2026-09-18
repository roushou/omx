use super::{fixtures::Fixture, *};
use omega::testing::{State, SystemTopic, topic::ApplicationsState};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface_with::<Panel>("standalone", || {
            use omega::config::Fields;
            Fixture::configured(
                &Fixture::state(),
                &Settings {
                    width: 552,
                    visible_rows: 7,
                    ..Default::default()
                }
                .write(),
            )
        })
        .surface::<Indicator>("bar", Fixture::state())
        .surface_with::<Panel>("applications", || Fixture::panel(&Fixture::state()))
        .surface::<Panel>("favorites-loading", Fixture::state())
        .surface_with::<Panel>("favorites", || {
            let mut panel = Fixture::panel(&Fixture::state())?;
            Fixture::favorites(&mut panel, &["org.example.Terminal.desktop"])?;
            Ok(panel)
        })
        .surface_with::<Panel>("fuzzy", || {
            let mut panel = Fixture::panel(&Fixture::state())?;
            Fixture::edit(&mut panel, "txted");
            Ok(panel)
        })
        .surface_with::<Panel>("search", || {
            let mut panel = Fixture::panel(&Fixture::state())?;
            Fixture::edit(&mut panel, "file");
            Ok(panel)
        })
        .surface_with::<Panel>("no-matches", || {
            let mut panel = Fixture::panel(&Fixture::state())?;
            Fixture::edit(&mut panel, "unmatched query");
            Ok(panel)
        })
        .surface_with::<Panel>("empty", || {
            Fixture::panel(&State::new().with(ApplicationsState::default()))
        })
        .surface_with::<Panel>("unavailable", || {
            Fixture::panel(&State::new().absent(SystemTopic::Applications))
        })
        .run()
        .unwrap();
}
