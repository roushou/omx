use super::{fixtures::Fixture, *};
use omega::testing::{State, SystemTopic, topic::ApplicationsState};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface_with::<Panel>("standalone", || {
            use omega::config::Fields;
            Fixture::preview(
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
        .surface_with::<Panel>("applications", || {
            Fixture::preview(&Fixture::state(), &Default::default())
        })
        .surface::<Panel>("favorites-loading", Fixture::state())
        .surface_with::<Panel>("favorites", || {
            let mut panel = Fixture::preview(&Fixture::state(), &Default::default())?;
            Fixture::favorites(&mut panel, &["org.example.Terminal.desktop"])?;
            Ok(panel)
        })
        .surface_with::<Panel>("fuzzy", || {
            let mut panel = Fixture::preview(&Fixture::state(), &Default::default())?;
            Fixture::edit(&mut panel, "txted");
            Ok(panel)
        })
        .surface_with::<Panel>("search", || {
            let mut panel = Fixture::preview(&Fixture::state(), &Default::default())?;
            Fixture::edit(&mut panel, "file");
            Ok(panel)
        })
        .surface_with::<Panel>("no-matches", || {
            let mut panel = Fixture::preview(&Fixture::state(), &Default::default())?;
            Fixture::edit(&mut panel, "unmatched query");
            Ok(panel)
        })
        .surface_with::<Panel>("empty", || {
            Fixture::preview(
                &State::new().with(ApplicationsState::default()),
                &Default::default(),
            )
        })
        .surface_with::<Panel>("unavailable", || {
            Fixture::preview(
                &State::new().absent(SystemTopic::Applications),
                &Default::default(),
            )
        })
        .run()
        .unwrap();
}
