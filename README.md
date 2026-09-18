# omx

An [Omarchy](https://omarchy.org/) desktop configuration written in Rust with [Omega](https://github.com/roushou/omega).

Eleven plugins cover an app and command launcher, workspaces, a calendar, device controls,
media playback, system stats, and AI usage. Omarchy's menu and system tray stay
in the bar. Layout and plugin settings live in Rust; colors and fonts follow your
Omarchy theme.

[Requirements](#requirements) · [Plugins](#plugins) · [Configuration](#configuration) · [Troubleshooting](docs/troubleshooting.md) · [Contributing](CONTRIBUTING.md)

## Requirements

- Omarchy
- Rust 1.88+
- Omega 0.3.9+

See [Omega's installation instructions](https://github.com/roushou/omega) if you
haven't set it up yet.

From the repository directory, validate the configuration and preview a plugin:

```sh
export OMEGA_CONFIG_DIR="$PWD"
omega check
omega preview clock --case calendar
```

Run these from the repository root. Previews use sample data; opening the network
preview won't change your Wi-Fi connection.

To use the whole configuration, place the repository directory at `~/.config/omega`
or symlink that path to it. Move an existing configuration aside first. Then build and apply:

```sh
omega build --wait
omega status
```

This applies the bar layout to your running desktop. You can also select a
configuration directory with `OMEGA_CONFIG_DIR` when running Omega commands.

## Plugins

Click a bar indicator to open its panel. Workspace buttons switch directly.

| Plugin                                             | What it does                                               |
| -------------------------------------------------- | ---------------------------------------------------------- |
| [Workspaces](plugins/workspaces/README.md)         | Switch numbered workspaces and see which have windows      |
| [Launcher](plugins/launcher/README.md)             | Fuzzy app search and persistent favorites                  |
| [Clock](plugins/clock/README.md)                   | Local time and a keyboard-navigable calendar               |
| [Network](plugins/network/README.md)               | Connect to Wi-Fi, view VPN status and interface traffic    |
| [Audio](plugins/audio/README.md)                   | Adjust volume and mute the default output                  |
| [Display](plugins/display/README.md)               | Adjust the backlight and inspect connected monitors        |
| [Power](plugins/power/README.md)                   | Battery charge, time remaining, and power profiles         |
| [Bluetooth](plugins/bluetooth/README.md)           | Connect known devices and check their batteries            |
| [Media](plugins/media/README.md)                   | Control playback for individual MPRIS players              |
| [AI usage](plugins/ai-usage/README.md)             | Subscription allowances, token history, and prepaid credit |
| [System monitor](plugins/system-monitor/README.md) | CPU, memory, storage, temperatures, and fans               |

Each plugin's README lists its settings, supported controls, and preview commands.

### Standalone launcher

Open the launcher as an overlay:

```sh
omega present launcher panel --overlay --dismiss-on-outside --width 600 --height 700 \
  --config '{"width":552,"visible-rows":7}'
```

Type to search apps and desktop actions, use Up/Down to select, and press Enter to run one. Escape or a
click outside closes it. Favorites are shared with the bar panel
and persist across daemon restarts.

See the [launcher guide](plugins/launcher/README.md#replace-the-apps-shortcut)
to bind it to Super+Alt+Space. Keybindings are configured separately from the bar.

The launcher uses Omega's unreleased storage and command APIs. Development requires
linking the matching Omega checkout with `omega link /path/to/omega` and
running its daemon with protocol version 2. Rebuild plugins and reinstall the renderer
when updating Omega. Published Omega 0.3.9 cannot build this launcher yet.

## Configuration

Edit [system/src/main.rs](system/src/main.rs) to change the bar's order and
placement. Plugin options are Rust structs:

```rust
PluginWidget::new("network", network::Indicator)
    .settings(&network::Settings {
        show_ssid: true,
        ..Default::default()
    })
    .panel(network::Panel)
```

`PluginWidget` comes from `omega_omarchy::shell`. Add these declarations to a
bar section in `System::document`, then run `omega build --wait`. Placement
settings apply to both the indicator and its panel.

To reuse a plugin in another Omega configuration, copy its directory, add it to
your system crate's dependencies, and place its surfaces. Most plugins also use
[desktop-ui](crates/desktop-ui/README.md). AI usage needs the schedule shown
in [its README](plugins/ai-usage/README.md).

## Documentation

- [Troubleshooting](docs/troubleshooting.md): missing plugins, panel issues, stale data, and build failures.
- [Architecture](docs/architecture.md): workspace structure, state, actions, and rendering.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, tests, CI checks,
and pull request guidelines.

## License

[MIT](./LICENSE)
