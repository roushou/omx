//! Connection state, Wi-Fi controls, VPN status and interface traffic.

mod panel;
use omega::{
    Percent, Surface, View,
    platform::network::{Network, Vpn},
    surface::{Events, Task},
    ui::{Glyph, Icon, Row, Text},
};
pub use panel::{Effects, Message, Model, Panel};
use std::convert::Infallible;

/// Network presentation preferences.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Show the primary connection's SSID beside the bar icon. Defaults to false.
    pub show_ssid: bool,
    /// Show per-interface traffic counters and rates in the panel. Defaults to true.
    pub show_traffic: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_ssid: true,
            show_traffic: true,
        }
    }
}

/// Primary connection icon with an active-VPN marker.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    network: Network,
    vpn: Vpn,
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
        let label = if !self.network.has_reading() {
            "Network unavailable".into()
        } else if !self.network.is_connected() {
            "Disconnected".into()
        } else {
            self.network.ssid().unwrap_or_else(|| "Ethernet".into())
        };

        let mut row = Row::new()
            .gap(6)
            .tooltip(if self.vpn.is_connected() {
                format!("{label} · VPN connected")
            } else {
                label.clone()
            })
            .child(Icon::named(NetworkIcon::of(&self.network)));

        if self.settings.show_ssid && self.network.is_connected() {
            row = row.child(Text::new(label));
        }

        if self.vpn.is_connected() {
            row = row.child(Icon::new(Glyph::Lock));
        }

        row.into()
    }
}

pub(crate) struct NetworkIcon;

impl NetworkIcon {
    pub(crate) fn strength(level: Percent) -> &'static str {
        ["󰤯", "󰤟", "󰤢", "󰤥", "󰤨"][usize::from(level.whole_percent().saturating_sub(1) / 20).min(4)]
    }

    pub(crate) fn of(network: &Network) -> &'static str {
        if !network.is_connected() {
            "󰤮"
        } else if network.ssid().is_some() {
            Self::strength(network.strength())
        } else {
            "󰈀"
        }
    }
}

/// Register the indicator and stateful connection panel.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).surface(Panel)
}

#[cfg(test)]
mod fixtures;

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lifecycle_tests;
