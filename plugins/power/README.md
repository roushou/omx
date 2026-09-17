# Power

Battery charge, charging status, time remaining, and power-profile selection.
On desktops without a battery, a plug icon keeps the profile controls accessible.

## Configuration

| Setting           | Default | Description                                                                            |
| ----------------- | ------- | -------------------------------------------------------------------------------------- |
| `low`             | `20`    | Highlight charge below this percentage while unplugged and not charging; capped at 100 |
| `show_percentage` | `true`  | Show the charge percentage in the bar                                                  |

For an icon-only indicator with a 15% warning threshold:

```rust
PluginWidget::new("power", power::Indicator)
    .settings(&power::Settings { low: 15, show_percentage: false })
    .panel(power::Panel)
```

## Power profiles

Choose a profile in the panel. Available choices come from the system's
power-profile service, and any reported performance restrictions appear below
the controls. The selection updates when the service reports the change.

The `profile` command accepts Omega's `PowerProfile` type and rejects profiles
that are no longer available. This plugin doesn't switch profiles automatically
when AC power changes.

Battery information depends on what the system reports. Missing readings are
shown as unavailable. Charge limits, cycle count, power draw, and low-battery
notifications aren't implemented here.

## Development

```sh
omega preview power --case battery
omega preview power --case charging
omega preview power --case desktop
cargo test -p power
```

Use `omega preview power --list` for all cases. See
[src/lib.rs](src/lib.rs) for the battery indicator and profile controls.
