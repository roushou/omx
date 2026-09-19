# Display

Adjust the system backlight and inspect connected monitors. The panel lists each
monitor's name, resolution, refresh rate, and scale. The bar icon changes when
more than one display is connected.

## Configuration

`show_percentage` defaults to `true`. For an icon-only indicator:

```rust
PluginWidget::new("display", display::Indicator)
    .settings(&display::Settings { show_percentage: false })
    .panel(display::Panel)
```

## Brightness

Use the slider or run:

```sh
omega run display.brightness 60%
```

The command accepts 0–100% and controls the system backlight, usually the laptop
panel. It doesn't adjust every monitor in the list. On a machine without a
supported backlight, monitor details remain available but brightness controls
are unavailable.

External-monitor brightness over DDC/CI and changes to resolution, scale,
position, or mirroring aren't supported by this plugin.

## Development

```sh
omega preview display --case displays
omega preview display --case external-only
cargo test -p display
```

Use `omega preview display --list` for all cases. The
[source](src/lib.rs) combines Omega's `backlight` and `monitors` readings.
Disconnected outputs disappear on the next update.
