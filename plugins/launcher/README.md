# Launcher

Search installed desktop applications and launch them from the bar. Click the
magnifying glass beside the Omarchy menu, type a name, and press Enter.

## Keyboard controls

| Key       | Action                          |
| --------- | ------------------------------- |
| Up / Down | Select a result                 |
| Enter     | Launch the selected application |
| Escape    | Close the panel                 |

You can also click a result. The search field receives focus when the panel opens,
and **Clear** resets the query. Dismissing the panel resets search and selection.

To open a standalone launcher, for example from a compositor keybinding:

```sh
omega present launcher panel --overlay --width 600 --height 700 --dismiss-on-outside \
  --config '{"width":552,"visible_rows":7}'
```

## Replace the apps shortcut

Add this to `~/.config/hypr/bindings.lua`. It replaces Omarchy's dedicated apps
menu; the main menu remains on Super+Space.

```lua
hl.unbind("SUPER + ALT + SPACE")
o.bind("SUPER + ALT + SPACE", "Omega applications", [[omega present launcher panel --overlay --dismiss-on-outside --width 600 --height 700 --config '{"width":552,"visible_rows":7}']])
```

Run `hyprctl reload` and `hyprctl configerrors` after editing. Repeated invocations
reuse the launcher instance; Escape or a click outside dismisses it.

## Favorites

Select an application and use **Add favorite** or **Remove favorite**. With an
empty query, favorites appear first, followed by the remaining applications.
Both groups are alphabetical. Typing searches the installed application catalogue
and orders matches by relevance; favorites keep their star. Applications that
are no longer installed are not displayed.

Up to 32 favorites are shared by the bar and standalone launcher through Omega's
persistent JSON storage. They survive plugin and daemon restarts. Search and
selection stay local to each view.

The store is `omx.launcher.favorites`. Each key is an application ID; its presence
marks a favorite, and its value is `null`. Adding or removing a favorite changes
only that application's entry. Removal checks the entry's revision, so a stale
write cannot remove a concurrently replaced favorite. Save errors stay visible
and are not retried automatically.

While favorites are loading or unavailable, the panel disables favorite controls
and shows a status message. Search and application launching remain available.
Launching an application does not write to storage.

```sh
omega storage show omx.launcher.favorites
omega storage export omx.launcher.favorites > launcher-favorites.json
```

This implementation currently requires the local storage-enabled Omega checkout;
published Omega 0.3.9 does not provide this API.

## Configuration

| Setting             | Default | Description                                         |
| ------------------- | ------- | --------------------------------------------------- |
| `max_results`       | `40`    | Maximum results, clamped to 1–100                   |
| `show_descriptions` | `true`  | Show category or description below each name        |
| `width`             | `416`   | Content width in logical pixels, clamped to 320–800 |
| `visible_rows`      | `7`     | Visible rows before scrolling, clamped to 3–10      |

```rust
PluginWidget::new("launcher", launcher::Indicator)
    .settings(&launcher::Settings {
        max_results: 20,
        show_descriptions: false,
        ..Default::default()
    })
    .panel(launcher::Panel)
```

## Search and launch behavior

Search ignores case. Every word must match the application's name, category,
keywords, description, or desktop-entry ID. A token can also match a sequence of
characters in the name: `txted` finds **Text Editor**. Exact names and prefixes
rank above other matches; literal metadata matches rank above fuzzy matches.
Fuzzy matching preserves character order and favors fewer skipped characters.
Personal ordering applies only when the query is empty, so a favorite does not
hide a better search match.

Omega supplies the application catalogue, including desktop-entry visibility and
localization. Installing or removing applications updates results automatically.
Icons come from the desktop theme.

The launcher opens desktop entries; it doesn't run typed shell commands. While a
launch request is pending, repeated activation is disabled. Once the desktop
service accepts the launch, the panel hides. Acceptance does not guarantee that
the application finished starting. Reported failures remain visible, and requests
are never retried automatically.

If the search field doesn't receive focus, check `omega status --versions` and
install the renderer matching your CLI with `omega shell install`.

## Development

```sh
omega preview launcher --case applications
omega preview launcher --case search
omega preview launcher --case fuzzy
omega preview launcher --case favorites
omega preview launcher --case favorites-loading
omega preview launcher --case no-matches
cargo test -p launcher
```

The [source](src/lib.rs) contains the stateful panel and activation handling.
Previews use synthetic application entries and capture launch requests, so they
won't open real applications. Use `omega preview launcher --list` for all cases.
