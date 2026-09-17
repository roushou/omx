use super::{Indicator, Message, Panel, Settings, fixtures::Fixture};
use omega::{
    config::Fields,
    surface::Lifecycle,
    testing::{Drawn, State, SurfaceHarness, SystemTopic, manifest_of},
};

#[test]
fn clock_formats_midnight_noon_and_single_digit_hours() {
    let settings = Settings {
        twelve_hour: true,
        show_weekday: false,
        ..Settings::default()
    }
    .write();

    for (hour, expected) in [
        (0, "12:05 AM"),
        (9, "9:05 AM"),
        (12, "12:05 PM"),
        (23, "11:05 PM"),
    ] {
        assert_eq!(
            Drawn::configured::<Indicator>(&Fixture::at(2026, 9, 16, hour, 5), &settings)
                .unwrap()
                .text(),
            expected
        );
    }

    let drawn = Drawn::of::<Indicator>(&Fixture::at(2026, 9, 16, 0, 5)).unwrap();

    assert_eq!(drawn.text(), "Wednesday 00:05");
    assert!(
        drawn
            .prop("root", "tooltip")
            .unwrap()
            .contains("16 September 2026")
    );
}

#[test]
fn missing_or_invalid_readings_do_not_invent_a_date() {
    let absent = State::new().absent(SystemTopic::Time);

    assert_eq!(Drawn::of::<Indicator>(&absent).unwrap().text(), "—");
    assert_eq!(
        Drawn::of::<Panel>(&absent).unwrap().text(),
        "Clock unavailable"
    );

    let invalid = State::new().with(omega::testing::topic::TimeState {
        year: 2026,
        month: 2,
        day: 30,
        ..Default::default()
    });

    assert_eq!(
        Drawn::of::<Panel>(&invalid).unwrap().text(),
        "Clock unavailable"
    );
}

#[test]
fn calendar_navigation_is_local_and_live_date_keeps_updating() {
    let state = Fixture::at(2026, 9, 30, 23, 59);
    let mut first = SurfaceHarness::<Panel>::new(&state).unwrap();
    let mut second = SurfaceHarness::<Panel>::new(&state).unwrap();
    let drawn = first.draw();
    first
        .interact(&drawn, "previous-month", "press", ())
        .unwrap();

    assert_eq!(
        first.draw().prop("month", "text").as_deref(),
        Some("August 2026")
    );

    assert_eq!(
        second.draw().prop("month", "text").as_deref(),
        Some("September 2026")
    );

    let midnight = Fixture::at(2026, 10, 1, 0, 0);
    first.state(&midnight);
    second.state(&midnight);

    assert_eq!(
        first.draw().prop("month", "text").as_deref(),
        Some("August 2026")
    );

    assert_eq!(
        first.draw().prop("today-date", "text").as_deref(),
        Some("October 1")
    );

    assert_eq!(
        second.draw().prop("month", "text").as_deref(),
        Some("October 2026")
    );

    let drawn = first.draw();
    first.interact(&drawn, "today", "press", ()).unwrap();

    assert_eq!(
        first.draw().prop("month", "text").as_deref(),
        Some("October 2026")
    );

    first.send(Message::Shift(1)).unwrap();
    first.lifecycle(Lifecycle::Hidden).unwrap();
    first.lifecycle(Lifecycle::Presented).unwrap();

    assert_eq!(
        first.draw().prop("month", "text").as_deref(),
        Some("October 2026")
    );

    assert!(first.take_effect().is_none());
}

#[test]
fn shortcuts_reach_the_same_navigation_as_buttons() {
    let mut panel = SurfaceHarness::<Panel>::new(&Fixture::at(2026, 1, 15, 10, 0)).unwrap();
    let drawn = panel.draw();
    let root = drawn.node("calendar").unwrap();
    let left = root
        .shortcuts
        .iter()
        .find(|shortcut| shortcut.key == "key:ArrowLeft")
        .unwrap();
    panel.interact(&drawn, "calendar", &left.event, ()).unwrap();

    assert_eq!(
        panel.draw().prop("month", "text").as_deref(),
        Some("December 2025")
    );

    let drawn = panel.draw();
    let home = drawn
        .node("calendar")
        .unwrap()
        .shortcuts
        .iter()
        .find(|shortcut| shortcut.key == "key:Home")
        .unwrap();
    panel.interact(&drawn, "calendar", &home.event, ()).unwrap();

    assert_eq!(
        panel.draw().prop("month", "text").as_deref(),
        Some("January 2026")
    );
}

#[test]
fn calendar_preferences_control_layout_and_progress_is_pinned_to_today() {
    let state = Fixture::at(2024, 7, 2, 10, 0);
    let mut panel = SurfaceHarness::<Panel>::new(&state).unwrap();
    let before = panel.draw().prop("year-share", "text");
    panel.send(Message::Shift(6)).unwrap();

    assert_eq!(panel.draw().prop("year-share", "text"), before);
    assert_eq!(before.as_deref(), Some("50%"));

    let settings = Settings {
        monday_first: false,
        show_week_numbers: false,
        show_year_progress: false,
        ..Settings::default()
    }
    .write();
    let drawn = Drawn::configured::<Panel>(&state, &settings).unwrap();

    assert!(drawn.node("week-heading").is_none());
    assert!(drawn.node("year-progress").is_none());
    assert!(drawn.node("day-2024-06-30").is_some());

    let day_keys: Vec<_> = drawn
        .keys()
        .into_iter()
        .filter(|key| key.starts_with("day-") && !key.contains('.'))
        .collect();

    assert_eq!(day_keys.len(), 42);
}

#[test]
fn navigation_limits_disable_buttons_without_breaking_the_panel() {
    for (year, month, blocked) in [(1, 1, "previous-month"), (9999, 12, "next-month")] {
        let mut panel = SurfaceHarness::<Panel>::new(&Fixture::at(year, month, 1, 0, 0)).unwrap();

        assert_eq!(panel.draw().flag(blocked, "disabled"), Some(true));

        let title = panel.draw().prop("month", "text");
        panel
            .send(Message::Shift(if year == 1 { -1 } else { 1 }))
            .unwrap();

        assert_eq!(panel.draw().prop("month", "text"), title);
    }
}

#[test]
fn plugin_only_reads_time_and_uses_no_effect_capabilities() {
    let manifest = manifest_of(&super::plugin());

    assert_eq!(manifest.state_topics, ["time"]);
    assert_eq!(
        manifest.granted().unwrap(),
        [omega::internal::Capability::StateRead]
    );

    assert!(manifest.commands.is_empty());
    assert_eq!(
        manifest
            .surfaces
            .iter()
            .map(|surface| surface.id.as_str())
            .collect::<Vec<_>>(),
        ["indicator", "panel"]
    );
}
