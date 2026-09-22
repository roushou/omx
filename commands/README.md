# Command hosts

Each domain has a library of reusable commands and a binary that hosts them.
UI plugins import command types and declare `Caller<T>` effects. The launcher
uses the same libraries; it never imports another UI plugin.

```text
commands/audio/src/
  lib.rs       Public exports
  commands.rs  SetVolume and SetAudible
  host.rs      Host registration
  main.rs      Process entry point
```

To create another host, with `OMEGA_CONFIG_DIR` pointing to this repository:

```sh
omega new example-commands --command-host
```

This generates the files above, adds the Cargo metadata and system dependency,
and prints its registration. Replace the starter `Echo` command with the domain's
operations.

Register the host in `system/src/main.rs`:

```rust
Document::new()
    .command_host(audio_commands::Host::declaration())?
```

Passing the declaration directly uses Omega's default deployment. Set overrides
in the system document by calling `.deployment()`:

```rust
Document::new()
    .command_host(
        audio_commands::Host::declaration()
            .deployment()
            .persistent(omega::host::StartPolicy::OnDemand)
            .execution(omega::host::ExecutionPolicy::serial()),
    )?
```

The host binary calls `Host::declaration().run()`. Deployment settings belong to
the system document and are applied by the daemon.

With this configuration, Omega starts each host on its first call and keeps that
process running. Calls
from panels, the launcher, schedules, and the CLI reach the same host through
the daemon. Hosts use Omega's shared platform services; importing a library
creates neither a host process nor another native audio or Bluetooth service.
The default execution policy serializes calls with a bounded queue.

Command IDs stay independent of the crate and host names:

```sh
omega run audio.volume 40%
omega run audio.audible false
omega commands
```

The eight hosts cover audio, Bluetooth, display brightness, media playback,
power profiles, workspaces, AI usage, and screen capture (screenshot, OCR, and
recording). The AI host also owns the collector
and publishes `Snapshot`; its UI only subscribes and renders. A five-second
system schedule calls `ai-usage.poll` to publish collection progress.

Library imports alone do not register a host. Keep each host declaration in
`host.rs` and use it from both the binary and the system document so their
command registrations stay aligned. Cargo marks executable hosts with
`[package.metadata.omega] kind = "command-host"`.

Command structs use `#[derive(omega::Command)]` to wire their fields. The derive
supplies construction, so their `Command` implementations only declare `ID`,
input/output types, descriptions, and behavior. Commands written without the
derive implement `omega::command::Construct` separately. Each invocation gets a
fresh handler; persistent host processes retain shared service connections and
explicit shared state.
