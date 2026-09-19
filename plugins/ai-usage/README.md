# AI usage

Check AI coding subscriptions from the bar: allowance usage, reset times, today's
tokens, daily activity, and usage by model. Providers with prepaid credit can
show a balance instead of subscription limits.

The plugin uses Omarchy's usage collectors. Sign into the coding tools you use,
then open the panel and click **Refresh**. Codex and Claude can supply limits and
token history; Fireworks can supply account-wide usage and a credit balance.
The available data depends on the installed collector and your account setup.

## Configuration

| Setting      | Default   | Description                                                                                    |
| ------------ | --------- | ---------------------------------------------------------------------------------------------- |
| `providers`  | `[]`      | Show providers with usage or available limits; a nonempty list selects provider IDs explicitly |
| `preferred`  | `"codex"` | Provider selected when opening a new panel, if available                                       |
| `show_label` | `true`    | Show `AI` beside the bar icon                                                                  |

For a Codex-only panel:

```rust
PluginWidget::new("ai-usage", ai_usage::Indicator)
    .settings(&ai_usage::Settings {
        providers: vec!["codex".into()],
        ..Default::default()
    })
    .panel(ai_usage::Panel)
```

Explicitly selected providers can show setup status even when they have no usage.
These settings filter the display; collection still runs for all installed
providers. Each panel remembers its own provider selection across refreshes.

## Refreshing data

The [system document](../../system/src/main.rs) includes this schedule:

```rust
.schedule(Schedules::every(
    "ai-usage-poll",
    Cadence::seconds(5),
    Actions::invoke(ai_usage_commands::Poll),
))
```

`Actions`, `Cadence`, and `Schedules` come from `omega_document`. Include this
schedule and `.command_host(ai_usage_commands::Host::declaration())?` if you
copy the plugin into another config. Copy `commands/ai-usage` alongside the UI.

The schedule publishes worker progress every five seconds. Collection runs at
startup, every 15 minutes, or when you click **Refresh**. It can also be requested
from the CLI:

```sh
omega run ai-usage.refresh
```

One worker serves all monitors and panels. Repeated refresh requests share the
active collection. A run times out after 90 seconds; failures retry after one
minute. Cached values stay visible after collection errors, and records older
than 30 minutes are marked stale.

## Data source

The worker runs `omarchy agent usage-update` and reads its schema-version-1 JSON
files from `$XDG_STATE_HOME/omarchy/agents/usage`, or
`~/.local/state/omarchy/agents/usage` by default. Authentication and provider
requests belong to Omarchy's collectors. The plugin doesn't read credentials or
transcripts itself.

If data is missing, run `omarchy agent usage-update` in a terminal to inspect
collector errors, then refresh the panel. A malformed record keeps its last good
value with an error; removing a provider's file removes it from the snapshot.

Missing statistics are labelled, and missing prompt/session counts are omitted.
Estimated balances are marked as estimated. History covers the period reported
by each collector, which can differ between providers. Cross-device aggregation
isn't implemented.

## Development

```sh
omega preview ai-usage --case usage
omega preview ai-usage --case credit
omega preview ai-usage --case stale
cargo test -p ai-usage
```

[data.rs](../../commands/ai-usage/src/data.rs) parses collector records,
[source.rs](../../commands/ai-usage/src/source.rs) owns
collection and caching, and [panel.rs](src/panel.rs) renders the shared snapshot.
Previews use synthetic data and never run collectors.
