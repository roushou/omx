# Media

Playback controls for MPRIS players, including compatible music players and
browsers. The bar shows the current track and playback state. It hides when no
players are available; paused players remain accessible.

Open the panel to select a player, view track details, and use **Previous**,
**Play/Pause**, or **Next**. Controls that a player doesn't support are disabled.

## Configuration

| Setting       | Default | Description                                         |
| ------------- | ------- | --------------------------------------------------- |
| `show_title`  | `true`  | Show the track title beside the bar icon            |
| `title_chars` | `28`    | Maximum title length in characters, clamped to 8–80 |

```rust
PluginWidget::new("media", media::Indicator)
    .settings(&media::Settings {
        title_chars: 40,
        ..Default::default()
    })
    .panel(media::Panel)
```

## Player selection

The bar follows Omega's automatically selected player. Each panel can select a
different player without changing that choice. If your selected player exits,
choose another; controls won't silently switch to a different application.

The `play`, `pause`, `previous`, and `next` commands take a typed `PlayerId`.
Commands check that the player still exists and supports the requested operation.
Failed requests appear on the controls.

The displayed duration is the track's length. Elapsed playback time, seeking,
and artwork aren't available yet.

## Development

```sh
omega preview media --case players
omega preview media --case paused
cargo test -p media
```

Use `omega preview media --list` for all cases. See
[src/lib.rs](src/lib.rs) for the bar, commands, and panel selection.
