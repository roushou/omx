//! Known Bluetooth devices and explicit connection controls.

use bluetooth_commands::{Connect, Disconnect};

use desktop_ui::{ItemRow, PanelHeader};
use omega::{
    Surface, View,
    platform::bluetooth::{Bluetooth, BluetoothDevice, BluetoothStatus},
    surface::{Events, Task},
    ui::{Button, Column, Component, Glyph, Icon, Row, Separator, Size, Text},
};
use std::convert::Infallible;

/// Bar display preferences.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Show the number of connected devices beside the icon. Defaults to false.
    pub show_count: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { show_count: true }
    }
}

/// Adapter status and connected-device count.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    bluetooth: Bluetooth,
    #[omega(config)]
    settings: Settings,
}

impl Surface for Indicator {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let devices = self.bluetooth.connected_devices();
        let status = Devices::status(self.bluetooth.status());
        let tooltip = if devices.is_empty() {
            format!("Bluetooth · {status}")
        } else {
            format!(
                "Bluetooth · {}",
                devices
                    .iter()
                    .map(|d| d.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let mut row = Row::new()
            .gap(6)
            .tooltip(tooltip)
            .child(if devices.is_empty() {
                Icon::new(Glyph::Bluetooth)
            } else {
                Icon::named("󰂱")
            });

        if self.settings.show_count {
            row = row.child(Text::new(devices.len()));
        }

        row.into()
    }
}

/// Known devices, reported batteries, and connect/disconnect controls.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    bluetooth: Bluetooth,
}

/// Command dependencies used by this surface's bindings.
#[derive(Debug, omega::Effects)]
pub struct CommandEffects {
    _connect: omega::command::Caller<Connect>,
    _disconnect: omega::command::Caller<Disconnect>,
}

impl Surface for Panel {
    type Model = ();
    type Message = Infallible;
    type Effects = CommandEffects;

    fn update(&self, _: &mut (), message: Infallible, _: &Self::Effects) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let mut panel = Column::new().width(352).gap(14).child(
            PanelHeader::labelled("Bluetooth", Devices::status(self.bluetooth.status()))
                .leading(Icon::new(Glyph::Bluetooth).size(Size::Display))
                .render(),
        );

        let mut devices = self.bluetooth.known_devices();
        devices.sort_by(|a, b| {
            b.is_connected()
                .cmp(&a.is_connected())
                .then(a.name().cmp(b.name()))
                .then(a.id().cmp(b.id()))
        });

        if devices.is_empty() {
            return panel
                .child(
                    Text::new(match self.bluetooth.status() {
                        BluetoothStatus::Unavailable => "Bluetooth unavailable",
                        BluetoothStatus::NoAdapter => "No Bluetooth adapter",
                        BluetoothStatus::Off => "Bluetooth is off",
                        BluetoothStatus::On => "No paired devices",
                    })
                    .muted(),
                )
                .into();
        }

        for device in devices {
            panel = panel
                .child(Separator::new())
                .child(DeviceCard(&device).render());
        }

        panel.into()
    }
}

struct DeviceCard<'a>(&'a BluetoothDevice);

impl Component for DeviceCard<'_> {
    fn render(&self) -> View {
        let device = self.0;
        let id = device.id().clone();
        let state = if device.is_connected() {
            "Connected"
        } else if device.can_connect() {
            "Paired"
        } else {
            "Unavailable"
        };

        let detail = match device.battery() {
            Some(level) => format!("{state} · Battery {level}"),
            None => state.into(),
        };

        let control = if device.is_connected() {
            Button::new("Disconnect")
                .key("connection")
                .secondary()
                .on_press(Disconnect.with(id.clone()))
        } else {
            Button::new("Connect")
                .key("connection")
                .disabled_if(!device.can_connect())
                .on_press(Connect.with(id.clone()))
        };

        Column::new()
            .key(id.as_str())
            .gap(8)
            .child(
                ItemRow::new(Text::new(device.name()).bold())
                    .leading(Devices::icon(device))
                    .subtitle(Text::new(detail).size(Size::Caption).muted())
                    .content_gap(3)
                    .render(),
            )
            .child(control.fill_width())
            .into()
    }
}

struct Devices;

impl Devices {
    fn status(status: BluetoothStatus) -> &'static str {
        match status {
            BluetoothStatus::Unavailable => "Unavailable",
            BluetoothStatus::NoAdapter => "No adapter",
            BluetoothStatus::Off => "Off",
            BluetoothStatus::On => "On",
        }
    }

    fn icon(device: &BluetoothDevice) -> Icon {
        match device.icon() {
            "audio-headphones" | "audio-headset" => Icon::new(Glyph::Headphones),
            "input-keyboard" => Icon::new(Glyph::Keyboard),
            "input-mouse" => Icon::named("󰍽"),
            "phone" => Icon::named("󰏲"),
            _ => Icon::new(Glyph::Bluetooth),
        }
    }
}

/// Register the Bluetooth surfaces.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).surface(Panel)
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
