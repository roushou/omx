use super::{Body, Editor, Message, Model, Scratch, Settings};
use omega::config::Fields;
use omega::storage::Revision;
use omega::surface::TextEdit;
use omega::testing::{Drawn, State, Stored, SurfaceHarness};

fn revision(epoch: &str, revision: u64) -> Revision {
    Revision {
        epoch: epoch.repeat(32),
        revision,
    }
}

#[tokio::test]
async fn the_scratchpad_draws_a_multiline_editor() {
    let drawn = Drawn::of::<Editor>(&State::new()).unwrap();

    assert!(drawn.kinds().contains(&"textarea"));
    assert!(drawn.text().contains("Notes"));
}

#[tokio::test]
async fn an_edit_updates_the_local_draft() {
    let mut panel = SurfaceHarness::<Editor>::new(&State::new()).unwrap();
    panel
        .send(Message::Edited(TextEdit {
            text: "hello".into(),
            revision: 1,
            reset: 0,
        }))
        .unwrap();

    let Model { text, dirty, .. } = panel.model();
    assert_eq!(text.text(), "hello");
    assert!(*dirty);
}

#[tokio::test]
async fn a_stored_note_loads_into_the_editor() {
    let mut panel = SurfaceHarness::<Editor>::new(&State::new()).unwrap();

    let read = panel.expect_storage_read::<Scratch>().await.unwrap();
    let snapshot = Stored::<Scratch>::new(revision("a", 1))
        .unwrap()
        .entry(
            "scratch".to_string(),
            Body {
                text: "hello".into(),
            },
            1,
        )
        .unwrap();
    read.reply(&snapshot).unwrap();
    panel.complete().await.unwrap();

    assert_eq!(
        panel.draw().prop("root.1", "value").as_deref(),
        Some("hello")
    );
}

#[tokio::test]
async fn an_edit_inserts_the_note_when_none_exists() {
    let settings = Settings {
        autosave_millis: 0,
        ..Settings::default()
    };
    let mut panel = SurfaceHarness::<Editor>::configured(&State::new(), &settings.write()).unwrap();

    let read = panel.expect_storage_read::<Scratch>().await.unwrap();
    let empty = Stored::<Scratch>::new(revision("a", 0)).unwrap();
    read.reply(&empty).unwrap();
    panel.complete().await.unwrap();

    panel
        .send(Message::Edited(TextEdit {
            text: "hi".into(),
            revision: 1,
            reset: 0,
        }))
        .unwrap();
    panel.complete().await.unwrap();

    let insert = panel.expect_storage_insert::<Scratch>().await.unwrap();
    assert_eq!(insert.key().as_str(), "scratch");
    assert_eq!(insert.value().text, "hi");
    insert.succeed(revision("a", 1)).unwrap();
    panel.complete().await.unwrap();

    assert_eq!(panel.model().revision.as_ref().unwrap().revision, 1);
}
