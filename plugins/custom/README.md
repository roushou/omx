# Custom command widget

Shows a shell command's output in the bar, refreshed on an interval. The daemon
runs the command (bounded output, ten-second timeout); the widget renders its
trimmed stdout. This is the configurable "command module": one plugin serves any
command via per-placement settings.

## Settings

| Setting            | Default        | Description                          |
| ------------------ | -------------- | ------------------------------------ |
| `command`          | `date +%H:%M`  | Shell command; stdout is the text    |
| `interval_seconds` | `60`           | Seconds between runs; zero means one |

## Adding it to the bar

Add a placement to `system/src/main.rs`, giving that placement its command:

```rust
PluginWidget::new("kernel", custom::Custom)
    .settings(&custom::Settings {
        command: "uname -r".into(),
        interval_seconds: 3600,
    })
```

Place several instances with different settings to show several commands. Each
placement is its own instance with its own model and timer.

## Behavior

The first capture runs when the instance mounts. Each result renders the trimmed
stdout; a failed or timed-out command renders an em dash until the next attempt.
The timer restarts after each result, so a slow command does not stack requests.

## Tests

```sh
cargo test -p custom
```
