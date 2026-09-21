# Bluetooth

Connect and disconnect known Bluetooth devices. The panel lists connected devices
first and shows battery levels when the device reports them.

Pair a new device through your existing Bluetooth settings first. Discovery,
pairing, adapter power, trust, and forgetting devices aren't available here yet.

## Configuration

`show_count` defaults to `true`. For an icon-only indicator:

```rust
PluginWidget::new("bluetooth", bluetooth::Indicator)
    .settings(&bluetooth::Settings { show_count: false })
    .panel(bluetooth::Panel)
```

## Connections

Click **Connect** or **Disconnect** beside a device. Pending requests and failures
appear on the control; connection status updates when the Bluetooth service
reports the change. Blocked devices cannot be connected.

The `connect` and `disconnect` commands take Omega's adapter-qualified `DeviceId`.
This keeps devices on different adapters distinct, even when their addresses
match. Repeating a command for a device already in the requested state does
nothing. Devices removed since the panel rendered are rejected.

A missing battery percentage means the device hasn't reported one. A displayed
0% is an actual reading.

## Development

```sh
omega preview bluetooth --case devices
omega preview bluetooth --case off
cargo test -p bluetooth
```

Use `omega preview bluetooth --list` for all cases. See
[src/lib.rs](src/lib.rs) for device ordering,
[commands](../../commands/bluetooth/src/commands.rs) for connection controls, and
the `omega::ui` content components for shared layouts.
