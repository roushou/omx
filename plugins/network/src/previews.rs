use super::fixtures::Fixture;
use super::*;
use omega::testing::{State, SurfaceHarness, SystemTopic};
use omega_preview::Cases;

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::state())
        .surface::<Panel>("network", Fixture::state())
        .surface::<Panel>("vpn", Fixture::vpn())
        .surface_with::<Panel>("password", || {
            let mut panel = SurfaceHarness::new(&Fixture::state())?;
            panel.send(Message::Select("Guest/5G~".into()))?;
            Ok(panel)
        })
        .surface::<Panel>(
            "unavailable",
            State::new()
                .absent(SystemTopic::Network)
                .absent(SystemTopic::Wifi)
                .absent(SystemTopic::Throughput)
                .absent(SystemTopic::Vpn),
        )
        .run()
        .unwrap();
}
