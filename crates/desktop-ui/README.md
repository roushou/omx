# desktop-ui

Shared panel layouts for the plugins in this workspace. Use them for headings,
detail rows, sections, and labelled controls. They accept ordinary Omega views,
so a trailing slot can hold a value, a button, or another layout.

Add the library to a plugin's `Cargo.toml`:

```toml
[dependencies]
desktop-ui = { path = "../../crates/desktop-ui" }
```

## Example

```rust
use desktop_ui::{Detail, PanelHeader, Section};
use omega::ui::{Component, Glyph, Icon, Size, Text};

let header = PanelHeader::new(Text::new("Devices").bold())
    .leading(Icon::new(Glyph::Bluetooth).size(Size::Display))
    .subtitle(Text::new("Connected devices").muted());

let panel = Section::new()
    .heading(header)
    .gap(12)
    .child(Detail::row("Headphones", "80%"))
    .child(Detail::row("Keyboard", "Connected"))
    .width(360);
```

The caller sets panel width and padding, reads device state, and provides action
bindings. These components don't query services or keep application state.

## Components

| Component         | Use                                                       | Default spacing                                        |
| ----------------- | --------------------------------------------------------- | ------------------------------------------------------ |
| `ItemRow`         | Arbitrary title/subtitle, leading and trailing views      | 12 px between columns; 2 px between title and subtitle |
| `PanelHeader`     | Title with optional leading, subtitle, and trailing views | 14 px between columns; 2 px between title and subtitle |
| `Section`         | Optional heading followed by child views                  | 14 px                                                  |
| `Detail`          | Label and trailing content separated by flexible space    | 12 px minimum gap                                      |
| `LabelledControl` | Label/value row above a control or meter                  | 8 px vertical; 12 px between label and value           |
| `LevelControl`    | Percentage slider with a label and typed change binding   | Uses `LabelledControl`                                 |

`ItemRow` places optional leading and trailing content around a growing
title/subtitle column. All slots accept views; callers supply styles and bindings.
Use it for application and device rows. `PanelHeader` uses the same layout with
heading defaults. Neither component owns selection or click behavior.

The general layouts expose gap setters. Width, padding, and other component
modifiers work through Omega's `Component` trait.

Convenience constructors supply the styles used by the desktop:

- `PanelHeader::labelled(title, status)` adds a bold title and uppercase status
  caption. Use `new(title_view)` and `subtitle(view)` to supply your own content.
- `Detail::row(label, value)` creates a muted label and plain text value.
- `Detail::tile(label, value)` stacks a small caption above its value.
- `LevelControl` takes a label, `Percent`, and `Bind<Percent>`. Use
  `LabelledControl` when you need a different control or layout.

For example, a meter with custom spacing:

```rust
use desktop_ui::LabelledControl;
use omega::{Percent, ui::{Progress, Text}};

let used = Percent::whole(40);
let meter = LabelledControl::new(
    Text::new("Used").muted(),
    Text::new(used),
    Progress::new(used),
)
.gap(6);
```

Check for missing readings before constructing a meter. Showing zero would imply
that a measurement was available.

## Component keys

Passing a component directly to a container creates an Omega component scope.
For repeated controls, give each component a stable key. A `LevelControl` keyed
`output` exposes its internal slider as `output/slider`.

Calling `.render()` embeds the view without another scope. Use that form when
existing control keys or keyboard navigation references need to stay unchanged.
Action bindings work with either form.

## Development

```sh
cargo test -p desktop-ui
cargo doc -p desktop-ui --no-deps
```

See [src](src) for each component's implementation and API examples. The examples
in the Rust documentation run as doctests. If you copy a plugin that depends on
this crate, copy the library too, or replace its components with your own layouts.
