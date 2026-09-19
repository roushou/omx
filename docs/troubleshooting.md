# Troubleshooting

Start with the runtime status and the affected plugin's log:

```sh
omega status --versions
omega logs network -n 100
```

Replace `network` with the plugin name. Status reports whether the build was
accepted, plugin processes are running, and renderer attachments match the CLI.
Plugin logs remain available after a process exits. An empty log doesn't establish
that the panel or its data source is working.

## My edits don't appear

Check which configuration you're building. From the repository directory:

```sh
export OMEGA_CONFIG_DIR="$PWD"
omega check
omega build --wait
omega status
```

`cargo build` only compiles Rust. `omega check` validates plugins and the system
document without activating them. `omega build --wait` publishes a build and waits
for the daemon to apply it. Check plugin health afterward: a build being accepted
doesn't mean every plugin started successfully.

Bar order and placements live in [system/src/main.rs](../system/src/main.rs).
Plugin behavior and setting defaults live under `plugins/`.

## Omega isn't running

```sh
omega daemon status
```

If the service is installed but stopped:

```sh
systemctl --user start omega.service
```

If it isn't installed, or the status reports an outdated executable path:

```sh
omega daemon install
```

Installation configures and starts the user service. If startup still fails,
inspect its journal:

```sh
journalctl --user -u omega.service -n 100 --no-pager
```

## A plugin is missing or keeps restarting

Look for its entry in `omega status`, then read `omega logs <plugin>`.
A plugin absent from status may not be in the active build. Confirm that its crate
is under `plugins/`, run `omega check`, and rebuild.

A running plugin also needs a placement in the system document to appear in the
bar. A dependency in `system/Cargo.toml` alone doesn't place it. The media indicator
intentionally hides when there are no players.

After fixing the reported problem, rebuild if you changed code. To retry the
currently built plugin without rebuilding:

```sh
omega restart network
```

Restarting ends that plugin's current panel instances and their local state.
If restarting doesn't help, keep the logs and report the failure rather than
repeatedly restarting it.

## A command fails while its panel is healthy

The UI plugin and its command host are separate processes. For audio, inspect:

```sh
omega status audio-commands
omega commands --json
```

`Idle` is normal before an on-demand host's first call. `Backing off` reports
when another startup is eligible; an on-demand host still needs a new call.
`Failed` includes the last startup error. Status also shows active and queued
calls, process phases, and recent command failures with their execution times.
Use `omega commands audio-commands` for readable command and host diagnostics,
or add `--json` for structured output. A ready UI does not establish that its
command host can complete an operation.

Check that the host is registered with `.command_host(...)` in
`system/src/main.rs`. Importing its library into a plugin is not sufficient.
For daemon-side failures, inspect `journalctl --user -u omega.service -n 100`.

## Panels don't open, clicks fail, or keyboard focus is wrong

Check that the widget has a `.panel(...)` placement. Workspaces intentionally has
no panel; its buttons switch workspaces directly.

Next, inspect the renderer:

```sh
omega status --versions
omega shell status
```

The installed renderer and its running attachments should match the CLI. If you
upgraded the CLI but still have an older daemon or renderer, update them:

```sh
omega daemon install
omega shell install
omega status --versions
```

`omega shell install` installs the CLI's embedded renderer and restarts the
Omarchy shell. Verify the running attachments afterward; matching files on disk
alone don't prove the shell loaded them. Use the Omega version
listed in the [requirements](../README.md#requirements).

For failures specific to an external monitor, include which output is affected,
its resolution and scale, and whether disconnecting it changes the behavior.

## The standalone launcher looks different or its shortcut does nothing

First open it directly with the command in the
[launcher guide](../plugins/launcher/README.md#keyboard-controls). If that works,
check the compositor binding. Building omx doesn't install keybindings; the
[shortcut setup](../plugins/launcher/README.md#replace-the-apps-shortcut) is separate.

The bar popup and standalone overlay use the same launcher surface. The overlay
gets colors, fonts, spacing, and borders from Omarchy's theme. If its appearance
is stale after upgrading, run `omega shell install`, reopen the launcher, and
check `omega status --versions` while it is open. Standalone renderer attachments
are checked as well as bar attachments.

If `OMARCHY_PATH` is set, it must point to an Omarchy installation containing
`shell/Commons`. An invalid override prevents standalone theme initialization.

## Launcher favorites are missing or unavailable

Favorites persist across plugin and daemon restarts and are shared by the bar
popup and standalone overlay. Search and selection belong to each instance.
A favorite is shown only while its application is present in the installed
catalogue; changing the desktop-entry ID makes it a different application.

Inspect the saved IDs and any storage errors:

```sh
omega storage list
omega storage show omx.launcher.favorites
omega status launcher
```

The panel shows **Loading favorites…** or **Favorites unavailable** until its
subscription is ready. Favorite controls are disabled during that time, but
search and launching still work. Up to 32 favorites can be stored. Failed writes
leave an error in the panel; they are not retried automatically.

## A reading is unavailable or a control is disabled

First check whether the corresponding device or service is available outside
`omx`. A preview uses sample data, so a working preview doesn't prove that the
live service is connected.

| Symptom                  | Check                                                                                     |
| ------------------------ | ----------------------------------------------------------------------------------------- |
| No audio controls        | A default output is available in the audio service                                        |
| No Wi-Fi networks        | NetworkManager sees a wireless adapter and nearby networks                                |
| Bluetooth device missing | The device is already known to the Bluetooth service; this panel doesn't pair new devices |
| No brightness slider     | The system exposes a supported backlight; external DDC/CI controls aren't supported       |
| No battery reading       | The machine reports a battery; desktops can still use power profiles                      |
| No media player          | The application exposes MPRIS playback controls                                           |
| Missing mount or sensor  | The exact configured mount or sensor is reported by Omega                                 |

See the [plugin READMEs](../README.md#plugins) for supported operations. Missing
values aren't treated as zero. Some controls also disable while a request is
pending or when the device doesn't support the operation.

## AI usage is empty or stale

Run the collector directly to see its diagnostics:

```sh
omarchy agent usage-update
omega run ai-usage.refresh
```

Collectors require the relevant provider's login or account configuration. They
write records under `$XDG_STATE_HOME/omarchy/agents/usage`, defaulting to
`~/.local/state/omarchy/agents/usage`.

Check that the `ai-usage-poll` schedule remains in the system document. It publishes
results every five seconds; collection runs every 15 minutes. Refresh returns
when collection is requested, so completion isn't immediate. A run can take up to
90 seconds. Errors retain cached data, and records older than 30 minutes are
marked stale.

A provider hidden by `ai_usage::Settings::providers` won't appear even if its
collector succeeds. See [AI usage](../plugins/ai-usage/README.md) for the settings
and data contract.

## The configuration won't compile

Run `omega check` from the intended configuration and start with the first
compiler error. Check the installed Rust version with `rustc --version`; CI uses
stable Rust.

If Cargo tries to use an unavailable local Omega repository, inspect
`.cargo/config.toml`. To remove overrides created by `omega link`:

```sh
omega link --published
omega check
```

CLI, daemon, renderer, and SDK versions are reported separately by
`omega status --versions`. Updating the CLI doesn't update the workspace's Cargo
dependencies. When changing dependencies, update and commit `Cargo.lock`; CI uses
`--locked`. For the full validation commands, see
[Contributing](../CONTRIBUTING.md#checks).

## A preview won't open

Run the plugin's tests and list its registered cases:

```sh
cargo test -p network
omega preview network --list
omega preview network --case network
```

The default preview entry point is the library test `previews::preview`.
Use the exact case name from `--list`. If compilation succeeds but the preview
window fails, inspect the CLI's error for missing renderer runtime dependencies.
Rust tests can run without an Omarchy session; interactive previews still need
the graphical preview runtime.

## Reporting a bug

Include the reproduction steps, expected behavior, `omega status --versions`,
and relevant plugin logs. A screenshot or a failing preview case helps for UI
problems. Remove credentials and personal data before posting. See the
[bug report guidance](../CONTRIBUTING.md#bug-reports).
