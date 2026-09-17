use super::{Message, Panel, fixtures::Fixture};
use omega::{
    effect::EffectError,
    surface::Lifecycle,
    testing::{SurfaceHarness, operation::Refusal},
};

#[tokio::test]
async fn dismissed_requests_release_pending_without_restoring_old_errors() {
    for connect in [false, true] {
        for reopen_before_completion in [false, true] {
            let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
            panel.lifecycle(Lifecycle::Presented).unwrap();
            panel.send(Message::Select("Cafe".into())).unwrap();
            panel
                .send(if connect {
                    Message::Connect
                } else {
                    Message::Disconnect
                })
                .unwrap();

            assert!(panel.model().requesting);

            panel.lifecycle(Lifecycle::Hidden).unwrap();

            if reopen_before_completion {
                panel.lifecycle(Lifecycle::Presented).unwrap();
            }

            panel.send(Message::Disconnect).unwrap();

            assert!(panel.model().requesting);
            panel
                .complete_effect(Err(EffectError::Refused(Refusal::unavailable(
                    "failure from dismissed request",
                ))))
                .await
                .unwrap();

            assert!(
                panel.take_effect().is_none(),
                "hiding must not admit duplicates"
            );

            panel.complete().await.unwrap();

            assert!(!panel.model().requesting);

            if !reopen_before_completion {
                panel.lifecycle(Lifecycle::Presented).unwrap();
            }
            assert!(
                !panel
                    .draw()
                    .text()
                    .contains("failure from dismissed request")
            );

            assert!(panel.model().selected.is_none());
            assert!(panel.model().password.text().is_empty());

            panel.send(Message::Disconnect).unwrap();
            panel
                .complete_effect(Err(EffectError::Refused(Refusal::unavailable(
                    "failure from current request",
                ))))
                .await
                .unwrap();
            panel.complete().await.unwrap();

            assert!(panel.draw().text().contains("failure from current request"));
        }
    }
}

#[tokio::test]
async fn closing_a_pending_request_allows_a_new_request_on_reopen() {
    for connect in [false, true] {
        let mut panel = SurfaceHarness::<Panel>::new(&Fixture::state()).unwrap();
        panel.send(Message::Select("Cafe".into())).unwrap();
        panel
            .send(if connect {
                Message::Connect
            } else {
                Message::Disconnect
            })
            .unwrap();
        // Resolve admission but leave its task completion queued when the instance closes.
        panel.complete_effect(Ok(None)).await.unwrap();
        panel.lifecycle(Lifecycle::Closed).unwrap();

        assert!(!panel.model().requesting);
        assert!(panel.model().selected.is_none());
        assert!(panel.model().password.text().is_empty());

        panel.lifecycle(Lifecycle::Presented).unwrap();
        panel.send(Message::Disconnect).unwrap();

        assert!(panel.model().requesting);
        panel.complete_effect(Ok(None)).await.unwrap();
        panel.complete().await.unwrap();

        assert!(!panel.model().requesting);
        assert!(panel.take_effect().is_none());
    }
}
