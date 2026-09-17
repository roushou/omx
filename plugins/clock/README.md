# Clock

Local time in the bar and a calendar in the panel. The default display uses a
24-hour clock and the full weekday name. Click it to browse months and years.

## Calendar controls

| Key                                      | Action                      |
| ---------------------------------------- | --------------------------- |
| Left / Right, h / l, Page Up / Page Down | Previous / next month       |
| Up / Down, k / j                         | Previous / next year        |
| Home, t                                  | Return to today             |
| Tab / Shift+Tab                          | Move between buttons        |
| Enter / Space                            | Activate the focused button |
| Escape                                   | Close the panel             |

Opening the panel returns to the current month. It follows midnight rollover
while you're viewing that month; browsing elsewhere keeps your chosen month.

## Configuration

| Setting              | Default | Description                                |
| -------------------- | ------- | ------------------------------------------ |
| `twelve_hour`        | `false` | Use a 12-hour clock with AM/PM             |
| `show_weekday`       | `true`  | Show the weekday in the bar                |
| `monday_first`       | `true`  | Start weeks on Monday; `false` uses Sunday |
| `show_week_numbers`  | `true`  | Show ISO week numbers                      |
| `show_year_progress` | `true`  | Show the fraction of the year completed    |

For a 12-hour clock and Sunday-first calendar:

```rust
PluginWidget::new("clock", clock::Indicator)
    .settings(&clock::Settings {
        twelve_hour: true,
        monday_first: false,
        ..Default::default()
    })
    .panel(clock::Panel)
```

Calendar labels are English. ISO week numbers use the Thursday in each row,
including with a Sunday-first calendar. Navigation covers years 1–9999.
Time updates once per minute. Timezone selection, seconds, alarms, and calendar
accounts aren't supported.

## Development

```sh
omega preview clock --case calendar
omega preview clock --case sunday-start
omega preview clock --case leap-year
cargo test -p clock
```

[calendar.rs](src/calendar.rs) contains date calculations;
[panel.rs](src/panel.rs) handles browsing and keyboard input. The plugin reads
local time from Omega. Preview dates are fixed, including leap-day and ISO
week-boundary cases.
