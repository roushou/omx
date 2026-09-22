# Notifications

Notifications Omega has raised, shown in a standalone overlay, plus a battery
alert that raises one when the charge crosses a low or critical threshold.

## Summon the center

Bind a compositor shortcut to present the center:

```sh
omega present notifications center --overlay --dismiss-on-outside --width 360 --height 320
```

Add it to `~/.config/hypr/bindings.lua`:

```lua
o.bind("SUPER + N", "Omega notifications", [[omega present notifications center --overlay --dismiss-on-outside --width 360 --height 320]])
```

The center lists the notifications Omega has raised, newest last, and shows an
empty state when there are none. It is read-only: dismissal belongs to the
desktop's notification server, which removes the entry once it closes.

## Battery alerts

The plugin registers a reaction on `EVENT_BATTERY_LOW` and
`EVENT_BATTERY_CRITICAL`. Each crossing raises a notification whose body carries
the level that crossed. The reaction runs only on crossings, not every reading.

## Tests and previews

```sh
omega preview notifications --case raised
omega preview notifications --case empty
cargo test -p notifications
```
