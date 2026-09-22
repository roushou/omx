# Notes

A persistent scratchpad, shown as an anchored overlay. Write a note, then
dismiss it; the text autosaves.

## Summon and dismiss

Bind a compositor shortcut to present the editor:

```sh
omega present notes editor --overlay --dismiss-on-outside --width 440 --height 360 \
  --config '{"rows":10}'
```

Add it to `~/.config/hypr/bindings.lua`, replacing any existing binding for the
chosen chord:

```lua
o.bind("SUPER + ALT + N", "Omega notes", [[omega present notes editor --overlay --dismiss-on-outside --width 440 --height 360]])
```

Repeated invocations reuse the same instance. Escape or a click outside dismisses
it. See the [launcher guide](../launcher/README.md#replace-the-apps-shortcut) for
the same binding pattern.

## Behavior

Edits autosave to the `omx.notes.scratch` store after a short quiet period, and
when the overlay is hidden. The editor keeps your caret and scroll position
across redraws. A failed save stays visible and is retried on the next edit.

The note is a single scratchpad entry. Inspect or back it up with:

```sh
omega storage show omx.notes.scratch
omega storage export omx.notes.scratch > notes.json
```

## Configuration

| Setting           | Default | Description                                     |
| ----------------- | ------- | ----------------------------------------------- |
| `rows`            | `10`    | Visible editor rows, clamped to 2–40            |
| `autosave_millis` | `500`   | Quiet period before a write; `0` writes at once |

```sh
omega preview notes --case scratch
omega preview notes --case compact
cargo test -p notes
```
