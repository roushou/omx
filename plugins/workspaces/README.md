# Workspaces

Numbered workspace buttons for the bar. Click a number to switch workspaces.
A circle marks the focused workspace; occupied workspaces use the normal text
color and empty ones are dimmed.

By default, slots 1–5 stay visible even when empty. Existing workspaces through
10 are added as needed, with workspace 10 labelled `0`. Every monitor shows the
same list and global focus. Hover a button for the workspace name, monitor, and
window count.

## Configuration

| Setting      | Default | Description                                                            |
| ------------ | ------- | ---------------------------------------------------------------------- |
| `persistent` | `5`     | Always show slots 1 through this number; `0` disables persistent slots |
| `maximum`    | `10`    | Highest displayed number, clamped to 1–30                              |
| `vertical`   | `false` | Stack buttons vertically                                               |

To keep all ten workspaces visible:

```rust
PluginWidget::new("workspaces", workspaces::Indicator)
    .settings(&workspaces::Settings {
        persistent: 10,
        ..Default::default()
    })
```

`persistent` is capped at `maximum`. This plugin has no popup panel.

## Switching workspaces

You can also switch through Omega's CLI:

```sh
omega run workspaces.select 3
```

The `select` command takes a `WorkspaceIndex`. Unopened numbered workspaces are
valid targets; the compositor decides which monitor to use and whether to create
the workspace. Its back-and-forth behavior also applies.

The focus marker follows compositor state, including changes made through global
shortcuts. Named and special workspaces with negative IDs aren't shown in this
numbered list. Failed switch requests appear through Omega's control feedback.

## Development

```sh
omega preview workspaces --case bar
omega preview workspaces --case external-focus
cargo test -p workspaces
```

Use `omega preview workspaces --list` for all cases. The
[source](src/lib.rs) uses Omega's typed workspace API. Fixtures include multiple
monitors, empty slots, renamed workspaces, and focus changes.
