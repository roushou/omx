# Audio

Volume and mute controls for the default audio output. The bar shows a speaker
icon and volume percentage; click it to open the slider and mute switch.

## Configuration

`show_percentage` defaults to `true`. For an icon-only indicator:

```rust
PluginWidget::new("audio", audio::Indicator)
    .settings(&audio::Settings { show_percentage: false })
    .panel(audio::Panel)
```

Add this placement in [system/src/main.rs](../../system/src/main.rs).

## Commands

```sh
omega run audio volume 40%
omega run audio audible false  # mute
omega run audio audible true   # unmute
```

Volume accepts 0–100%. Mute commands set the requested state, so running the same
command twice doesn't toggle the output back. The backend chooses the default
output when it handles the request. Controls are unavailable when no audio
output is reported, and failed requests appear in the panel.

This plugin controls output audio only. Microphone controls, output selection,
and per-application mixing aren't available yet.

## Development

```sh
omega preview audio --case output
omega preview audio --case muted
cargo test -p audio
```

Use `omega preview audio --list` for all cases. The
[source](src/lib.rs) uses Omega's audio reading and control APIs, with the slider
layout from [desktop-ui](../../crates/desktop-ui/README.md).
