use super::Center;
use omega::testing::{State, SystemTopic, topic::NotificationsState};
use omega_proto::omega::ActiveNotification;
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn raised() -> State {
        State::new().with(NotificationsState {
            notifications: vec![
                ActiveNotification {
                    id: 1,
                    summary: "Battery low".into(),
                    body: "15% remaining".into(),
                    ..Default::default()
                },
                ActiveNotification {
                    id: 2,
                    summary: "Update ready".into(),
                    body: "Reboot to finish".into(),
                    ..Default::default()
                },
            ],
        })
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Center>("raised", Fixture::raised())
        .surface::<Center>("empty", State::new().absent(SystemTopic::Notifications))
        .run()
        .unwrap();
}
