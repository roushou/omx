use super::previews::Fixture;
use super::*;
use omega::{
    config::IntoValue,
    testing::{Called, Drawn, State, SurfaceHarness, SystemTopic, topic::BacklightState},
};

#[test]
fn missing_backlight_keeps_monitor_information_without_a_slider() {
    let panel = Drawn::of::<Panel>(&Fixture::state().absent(SystemTopic::Backlight)).unwrap();

    assert!(panel.text().contains("No supported backlight"));
    assert!(panel.first("slider").is_none());
    assert!(panel.text().contains("DP-1"));
    assert!(panel.text().contains("150%"));
}

#[test]
fn hotplug_removes_old_outputs_and_changes_the_bar_icon() {
    let state = Fixture::state();
    let mut panel = SurfaceHarness::<Panel>::new(&state).unwrap();
    let mut indicator = SurfaceHarness::<Indicator>::new(&state).unwrap();

    assert!(panel.draw().node("DP-1").is_some());

    let before = indicator.draw();

    assert_eq!(
        before
            .prop(&before.first("icon").unwrap(), "name")
            .as_deref(),
        Some("󰍺")
    );

    let next = State::new()
        .with(Fixture::monitors(1))
        .with(BacklightState { percent: 60 });
    panel.state(&next);
    indicator.state(&next);

    assert!(panel.draw().node("DP-1").is_none());

    let after = indicator.draw();

    assert_eq!(
        after.prop(&after.first("icon").unwrap(), "name").as_deref(),
        Some("󰍹")
    );
}

#[tokio::test]
async fn brightness_requires_a_backlight_and_a_valid_percentage() {
    for value in [-0.1, 1.1, f64::NAN] {
        let called =
            Called::raw::<SetBrightness>(&Fixture::state(), vec![value.into_value()]).await;

        assert!(called.answer.is_err());
        assert!(called.effects.is_empty());
    }

    let missing = Called::of::<SetBrightness>(
        &Fixture::state().absent(SystemTopic::Backlight),
        Percent::whole(60),
    )
    .await;

    assert!(missing.answer.is_err());
    assert!(missing.effects.is_empty());

    for percent in [0, 100] {
        let valid = Called::of::<SetBrightness>(&Fixture::state(), Percent::whole(percent)).await;

        assert!(valid.answer.is_ok());
        assert_eq!(valid.effects.len(), 1);
    }
}
