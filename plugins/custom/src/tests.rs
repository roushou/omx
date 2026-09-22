//! The command module captures through the daemon and renders its output.

use super::*;
use omega::testing::{Drawn, State, SurfaceHarness, manifest_of};
use omega_proto::omega::{Capability, invoke};

#[tokio::test]
async fn the_widget_declares_spawn_and_starts_empty() {
    let manifest = manifest_of(&omega::Plugin::new("custom", "0.1.0").surface_default::<Custom>());
    assert!(manifest.granted().unwrap().contains(&Capability::Spawn));

    // No capture has completed yet, so there is nothing to draw.
    let drawn = Drawn::of::<Custom>(&State::new()).unwrap();
    assert_eq!(drawn.text(), "");
}

#[tokio::test]
async fn mounting_requests_a_capture_and_a_result_renders() {
    let mut harness = SurfaceHarness::<Custom>::new(&State::new()).unwrap();

    // Mounting asks the daemon to run the command.
    let effect = harness.take_effect().expect("a capture is requested");
    assert!(matches!(effect.operation(), invoke::Op::Act(_)));
    drop(effect);

    // A captured result is rendered.
    harness
        .send(Message::Refreshed(Ok("hello".into())))
        .unwrap();
    assert_eq!(harness.draw().text(), "hello");
}

#[tokio::test]
async fn a_failure_before_any_capture_renders_the_unavailable_mark() {
    let mut harness = SurfaceHarness::<Custom>::new(&State::new()).unwrap();
    harness
        .send(Message::Refreshed(Err(omega::Error::invalid("boom"))))
        .unwrap();
    assert!(harness.draw().text().contains('\u{2014}'));
}
