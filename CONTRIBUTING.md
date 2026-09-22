# Contributing to omx

Bug fixes, new plugins, and improvements to the existing panels are welcome.
You can work on plugin logic and tests without running an Omarchy desktop.
A desktop session is needed to check real devices and the installed bar.

## Set up

Fork the repository, clone your fork, and create a branch for your changes.
From the repository directory:

```sh
rustup toolchain install stable --component rustfmt --component clippy
export OMEGA_CONFIG_DIR="$PWD"
cargo test --workspace --locked
```

CI uses stable Rust. For Omega commands and visual previews, use the Omega version
listed in the [README](README.md#requirements). Setting
`OMEGA_CONFIG_DIR` makes those commands use this repository's configuration.

The extra CI tools are:

```sh
cargo install dprint --version 0.56.1 --locked
cargo install cargo-deny --version 0.20.2 --locked
cargo install cargo-shear --version 1.13.4 --locked
```

## Find the code

Read the [architecture guide](docs/architecture.md) for how configuration, plugin
processes, surface state, and shared components fit together.

| Directory             | Contents                                               |
| --------------------- | ------------------------------------------------------ |
| `plugins/<name>/src/` | Plugin surfaces, tests, and preview cases              |
| `commands/<domain>/`  | Command implementations and host declarations          |
| `crates/typesafe/`    | Semantic-search client                                 |
| `system/`             | Bar layout, plugin placements, settings, and schedules |

Start with the plugin's README. It describes the controls, defaults, and current
limitations. [Network](plugins/network/README.md) is an example of a panel with
input and asynchronous requests; [clock](plugins/clock/README.md) shows local
panel state and keyboard navigation.

## Change a plugin

Run its tests and open a preview while you work:

```sh
cargo test -p network
omega preview network --list
omega preview network --case network
```

Preview cases use fixed data and captured effects. They must not launch real
applications, change device settings, or contact provider accounts. Add a case
when a new state would help reviewers understand a visual change.

Keep rendering separate from actions. Render from Omega readings and local panel
state; send changes through commands or surface effects. Use reported device state
to confirm changes, and show request failures where the user can act on them.
Missing readings should appear as unavailable rather than zero.

Keep behavior specific to a plugin in that plugin. Move a layout into `omega::ui::content`
when several plugins need it, and let callers provide its content and bindings.
New system APIs or renderer behavior belong in Omega.

For bug fixes, add a regression test that exercises the failing behavior. Test
interaction handling with `SurfaceHarness`, including unavailable devices or
pending requests when relevant. Documentation and spacing changes generally
need formatting checks and visual review rather than new tests.

Before trying a change on your desktop, validate the configuration:

```sh
omega check
```

To apply it to the running bar:

```sh
omega build --wait
omega status
```

`omega check` doesn't activate a build. `omega build --wait` does change the live
configuration. Use previews first if you only need to inspect layout.

## Add a plugin, command host, or library

With `OMEGA_CONFIG_DIR` pointing to this repository:

```sh
omega new example
```

The CLI creates `plugins/example` and adds the dependency to `system/Cargo.toml`.
Workspace member globs pick up the new crate. Add its placement to
[system/src/main.rs](system/src/main.rs); the CLI doesn't edit the bar layout.

For a command host:

```sh
omega new example-commands --command-host
```

The CLI creates a reusable library and executable in `commands/example-commands/`
and links it to `system`. Add the printed `.command_host(...)` call to the
system document. The starter `example-commands.echo` command accepts text.

For a shared library:

```sh
omega new example-ui --lib --into plugins/example
```

Use the workspace's edition and set `publish.workspace = true` in new package
manifests. The inherited value is `false`: these crates are parts of a desktop
configuration, not separate registry releases.

Include a README with usage, setting defaults, and any required service or
schedule. Register preview cases in the `previews::preview` library test, following
an existing plugin, and add the plugin to the root README's table.

## Dependencies and Omega changes

Use registry dependencies for Omega and path dependencies for libraries within
this workspace. Commit `Cargo.lock` when dependency resolution changes. CI uses
`--locked` and checks dependencies with cargo-deny and cargo-shear.
The policy in [deny.toml](deny.toml) checks advisories, licenses, and sources;
duplicate transitive versions produce warnings.

For work that also needs changes in a local Omega repository:

```sh
omega link ~/dev/omega
```

Overrides go in the gitignored `.cargo/config.toml`. Return to published packages
with `omega link --published` before the final validation. If a change requires
an unreleased Omega API, mention that dependency in the pull request.

## Checks

Run these from the repository root before submitting code changes:

```sh
cargo fmt --all --check
dprint check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --all-targets --locked
cargo test --workspace --locked
cargo deny --locked check
cargo shear
```

These are the checks in [CI](.github/workflows/ci.yml), which runs on pull requests
and pushes to `main`. Use `cargo fmt --all` to
format Rust and `dprint fmt` to format Markdown, JSON, and TOML. For changes to
plugin registration or the system document, run `omega check` too.

## Pull requests

Target `main`. Describe the problem, what changes for the user, and how you tested
it. For visual changes, include a screenshot or the preview case used to review
it. Mention hardware or services you couldn't test.

Use a Conventional Commit title, for example:

```text
fix(network): clear the password when switching networks
feat(clock): add an option to hide week numbers
docs: clarify plugin setup
```

Update the affected README when controls, defaults, dependencies, or setup steps
change. Keep examples runnable and use the terms shown in the UI.

## Bug reports

Check [Troubleshooting](docs/troubleshooting.md) for diagnostic commands and
common setup problems.

Include steps to reproduce, expected and actual behavior, and the affected plugin.
For runtime problems, include `omega status --versions` and relevant output from
`omega logs <plugin>`. For display issues, include monitor resolution and scale.
Remove credentials and personal data from logs or screenshots before posting.

Contributions are covered by the repository's [MIT license](LICENSE).
