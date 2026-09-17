# Network

Wi-Fi connections, Ethernet status, active VPNs, and per-interface traffic.
The bar shows connection type and Wi-Fi strength, plus a marker when a VPN is active.

## Connect to Wi-Fi

Open the panel, select a network, enter a password if needed, and click **Connect**.
Leave the password empty to use saved credentials. Open networks don't need one.
The current network appears first, followed by networks sorted by signal strength
and name.

Connection controls stay disabled while a request is pending. Status changes when
NetworkManager reports the connection; an accepted request can still take time to
connect. Errors appear in the panel. **Disconnect Wi-Fi** leaves Ethernet and VPN
connections alone.

The password field clears when you change networks, submit, or dismiss the panel.
Passwords aren't stored by this plugin. Closing the panel doesn't cancel a
connection request already handed to NetworkManager.

## Configuration

| Setting        | Default | Description                                       |
| -------------- | ------- | ------------------------------------------------- |
| `show_ssid`    | `true`  | Show the primary connection name in the bar       |
| `show_traffic` | `true`  | Show interface rates and cumulative byte counters |

```rust
PluginWidget::new("network", network::Indicator)
    .settings(&network::Settings {
        show_ssid: true,
        ..Default::default()
    })
    .panel(network::Panel)
```

Traffic is listed by interface because Omega doesn't expose which interface
belongs to the primary connection. VPN status includes connections reported by
NetworkManager; the panel cannot start or stop them.

Radio toggles, manual scans, forgetting networks, enterprise authentication fields,
and IP/DNS configuration aren't available here. Use your network settings for
those operations.

## Development

```sh
omega preview network --case network
omega preview network --case password
omega preview network --case vpn
cargo test -p network
```

[panel.rs](src/panel.rs) handles network selection, password input, and connection
requests. [lib.rs](src/lib.rs) contains the indicator and settings. Previews capture
requests without changing real connections.
