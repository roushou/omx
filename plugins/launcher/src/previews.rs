use super::{fixtures::Fixture, *};
use omega::testing::{State, SurfaceHarness, SystemTopic, topic::ApplicationsState};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface_with::<Panel>("standalone", || {
            use omega::config::Fields;
            SurfaceHarness::configured(
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
        .surface::<Panel>("applications", Fixture::state())
        .surface_with::<Panel>("favorites", || {
            use omega::config::Fields;
            use omega::record::PluginState;
            let history = History {
                favorites: vec!["org.example.Terminal.desktop".into()],
                recent: vec!["org.example.Files.desktop".into()],
            };
            SurfaceHarness::new(&Fixture::state().keyspace(&History::address(), history.write()))
        })
        .surface_with::<Panel>("fuzzy", || {
            let mut panel = SurfaceHarness::new(&Fixture::state())?;
            Fixture::edit(&mut panel, "txted");
            Ok(panel)
        })
        .surface_with::<Panel>("search", || {
            let mut panel = SurfaceHarness::new(&Fixture::state())?;
            Fixture::edit(&mut panel, "file");
            Ok(panel)
        })
        .surface_with::<Panel>("no-matches", || {
            let mut panel = SurfaceHarness::new(&Fixture::state())?;
            Fixture::edit(&mut panel, "unmatched query");
            Ok(panel)
        })
        .surface::<Panel>("empty", State::new().with(ApplicationsState::default()))
        .surface::<Panel>(
            "unavailable",
            State::new().absent(SystemTopic::Applications),
        )
        .run()
        .unwrap();
}
