use crate::tests::Fixture;
use crate::{Indicator, Message, Panel};
use omega::{config::Fields, testing::SurfaceHarness};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::state(Fixture::snapshot()))
        .surface::<Panel>("usage", Fixture::state(Fixture::snapshot()))
        .surface_with::<Panel>("claude", || {
            let mut panel = SurfaceHarness::new(&Fixture::state(Fixture::snapshot()))?;
            panel.send(Message::Select("claude".into()))?;
            Ok(panel)
        })
        .surface_with::<Panel>("credit", || {
            let mut panel = SurfaceHarness::new(&Fixture::state(Fixture::snapshot()))?;
            panel.send(Message::Select("fireworks".into()))?;
            Ok(panel)
        })
        .surface::<Panel>("stale", Fixture::state(Fixture::stale()))
        .surface::<Panel>(
            "loading",
            Fixture::state(crate::Snapshot {
                refreshing: true,
                ..Default::default()
            }),
        )
        .surface::<Panel>(
            "empty",
            Fixture::state(crate::Snapshot {
                loaded: true,
                ..Default::default()
            }),
        )
        .surface_with::<Panel>("single-provider", || {
            SurfaceHarness::configured(
                &Fixture::state(Fixture::snapshot()),
                &crate::Settings {
                    providers: vec!["codex".into()],
                    ..Default::default()
                }
                .write(),
            )
        })
        .run()
        .unwrap();
}
