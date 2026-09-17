# System monitor

CPU and memory usage in the bar, with load averages, swap, uptime, storage,
temperatures, and fan speeds in the panel.

## Configuration

| Setting       | Default          | Description                                   |
| ------------- | ---------------- | --------------------------------------------- |
| `show_memory` | `true`           | Include RAM usage beside CPU usage in the bar |
| `show_cores`  | `false`          | Show individual CPU cores in the panel        |
| `mounts`      | `["/", "/home"]` | Exact mount paths to display                  |

```rust
PluginWidget::new("system-monitor", system_monitor::Indicator)
    .settings(&system_monitor::Settings {
        show_cores: true,
        mounts: vec!["/".into()],
        ..Default::default()
    })
    .panel(system_monitor::Panel)
```

## Reading the numbers

- **RAM used** is total memory minus available memory.
- **Load averages** cover 1, 5, and 15 minutes. They aren't CPU percentages.
- **Storage available** excludes blocks reserved by the filesystem. If `/home`
  isn't a separate reported mount, it's marked as unreported; configure `/` instead.
- **Temperatures** retain the sensor and chip names. No temperature threshold is
  assumed to be safe or critical for every machine.
- **Fans** show RPM. A reported 0 RPM means the fan is stopped.

Repeated mount paths appear once, and capacities from different mount entries
aren't added together. Missing services and sensors are labelled. A machine with
no configured swap is distinguished from one with unused swap.

The panel displays system readings only. Process lists, history charts, and
process termination aren't included.

## Development

```sh
omega preview system-monitor --case system
omega preview system-monitor --case no-sensors
cargo test -p system-monitor
```

Use `omega preview system-monitor --list` for all cases. The
[source](src/lib.rs) reads Omega's `system`, `disk`, and `thermals` topics;
Omega handles sampling.
