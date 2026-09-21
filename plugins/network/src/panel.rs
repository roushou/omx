mod wireless;

use super::{NetworkIcon, Settings};
use omega::ui::{Detail, PanelHeader, Section};
use omega::{
    Surface, View,
    platform::network::{Network, Throughput, Vpn, Wifi, WifiControl, WifiPhase},
    surface::{Events, Lifecycle, Task, TextEdit, TextValue},
    ui::{Column, Component, Header, Icon, Row, Separator, Size, Text},
};
use wireless::Wireless;

/// A network panel whose selection, password and request state belong to one instance.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    network: Network,
    wifi: Wifi,
    traffic: Throughput,
    vpn: Vpn,
    #[omega(config)]
    settings: Settings,
}

/// Instance-local connection form. Credentials are never stored in a record.
#[derive(Default)]
pub struct Model {
    pub(crate) selected: Option<String>,
    pub(crate) password: TextValue,
    pub(crate) requesting: bool,
    error: String,
    epoch: u64,
}

/// Local connection interactions and their completion.
pub enum Message {
    Select(String),
    Password(TextEdit),
    Connect,
    Disconnect,
    Completed {
        epoch: u64,
        result: omega::Result<()>,
    },
}

/// Wi-Fi controls used only by event behavior.
#[derive(omega::Effects)]
pub struct Effects {
    wifi: WifiControl,
}

impl Surface for Panel {
    type Model = Model;
    type Message = Message;
    type Effects = Effects;

    fn update(&self, model: &mut Model, message: Message, effects: &Effects) -> Task<Message> {
        match message {
            Message::Select(ssid) => {
                if model.requesting || self.wifi.phase() == WifiPhase::Connecting {
                    return Task::none();
                }

                if model.selected.as_ref() != Some(&ssid) {
                    model.password.reset("");
                    model.error.clear();
                    model.selected = Some(ssid);
                }
            }

            Message::Password(edit) => {
                if !model.requesting && self.wifi.phase() != WifiPhase::Connecting {
                    model.password.apply(edit);
                }
            }

            Message::Connect => {
                if model.requesting || self.wifi.phase() == WifiPhase::Connecting {
                    return Task::none();
                }

                let point = self
                    .wifi
                    .networks()
                    .into_iter()
                    .find(|point| Some(point.ssid()) == model.selected.as_deref());

                let Some(point) = point else {
                    model.password.reset("");
                    model.error = "This network is no longer available".into();
                    return Task::none();
                };

                if point.is_active() {
                    model.password.reset("");
                    return Task::none();
                }

                let password = if point.is_secured() {
                    model.password.text().to_owned()
                } else {
                    String::new()
                };

                model.password.reset("");
                model.error.clear();
                model.requesting = true;
                let epoch = model.epoch;
                return Task::perform(
                    effects.wifi.connect(point.ssid(), password),
                    move |result| Message::Completed { epoch, result },
                );
            }

            Message::Disconnect => {
                if model.requesting || self.wifi.phase() == WifiPhase::Connecting {
                    return Task::none();
                }

                if self.wifi.phase() != WifiPhase::Connected {
                    model.error = "Wi-Fi is no longer connected".into();
                    return Task::none();
                }

                model.password.reset("");
                model.error.clear();
                model.requesting = true;
                let epoch = model.epoch;
                return Task::perform(effects.wifi.disconnect(), move |result| {
                    Message::Completed { epoch, result }
                });
            }

            Message::Completed { epoch, result } => {
                model.requesting = false;

                if epoch == model.epoch
                    && let Err(error) = result
                {
                    model.error = error.to_string();
                }
            }
        }

        Task::none()
    }

    fn lifecycle(&self, model: &mut Model, event: Lifecycle, _: &Effects) -> Task<Message> {
        if matches!(event, Lifecycle::Hidden | Lifecycle::Closed) {
            model.epoch += 1;
            model.password.reset("");
            model.selected = None;
            model.error.clear();
        }

        if event == Lifecycle::Closed {
            // Close cancels completion delivery; hiding retains the pending request.
            model.requesting = false;
        }

        Task::none()
    }

    fn render(&self, model: &Model, events: &Events<Message>) -> View {
        let wireless = Wireless::new(&self.wifi, model);
        let phase = wireless.phase;
        let mut panel = Column::new()
            .width(352)
            .gap(14)
            .child(self.header(phase, model.requesting))
            .children(self.vpn_section())
            .children(self.traffic_sections())
            .child(Separator::new())
            .child(Header::new("WI-FI"))
            .child(wireless.list(events))
            .children(wireless.controls(model, events));

        let error = if model.error.is_empty() && phase == WifiPhase::Failed {
            self.wifi.failure()
        } else {
            model.error.clone()
        };

        if !error.is_empty() {
            panel = panel.child(Text::new(error).key("error").warning());
        }

        panel.into()
    }
}

impl Panel {
    fn header(&self, phase: WifiPhase, requesting: bool) -> View {
        let status = if requesting {
            "Requesting…"
        } else {
            match phase {
                WifiPhase::Connecting => "Connecting…",
                WifiPhase::Failed => "Connection failed",
                _ if self.network.is_connected() => "Connected",
                WifiPhase::Connected => "Wi-Fi connected",
                WifiPhase::Disconnected => "Disconnected",
                WifiPhase::Unspecified => "Wi-Fi unavailable",
            }
        };

        let title = if phase == WifiPhase::Connecting && !self.wifi.ssid().is_empty() {
            self.wifi.ssid()
        } else {
            self.network
                .ssid()
                .filter(|_| self.network.is_connected())
                .unwrap_or_else(|| "Network".into())
        };

        PanelHeader::labelled(&title, status)
            .leading(Icon::named(NetworkIcon::of(&self.network)).size(Size::Display))
            .render()
    }

    fn vpn_section(&self) -> Vec<View> {
        let mut views = Vec::new();
        let tunnels = self.vpn.tunnels();

        if !tunnels.is_empty() {
            let section = Section::new().heading(Header::new("VPN")).children(
                tunnels
                    .iter()
                    .map(|tunnel| Detail::row(tunnel.name(), tunnel.interface())),
            );
            views.push(Separator::new().into());
            views.push(section.render());
        }

        views
    }

    fn traffic_sections(&self) -> Vec<View> {
        let mut views = Vec::new();
        if self.settings.show_traffic {
            for link in self
                .traffic
                .links()
                .into_iter()
                .filter(|link| !link.is_loopback())
            {
                let section = Section::new().heading(Header::new(link.interface())).child(
                    Row::new()
                        .gap(24)
                        .child(
                            Column::new()
                                .gap(8)
                                .fill_width()
                                .child(Detail::tile("Receiving", link.down()))
                                .child(Detail::tile("Downloaded", link.received())),
                        )
                        .child(
                            Column::new()
                                .gap(8)
                                .fill_width()
                                .child(Detail::tile("Sending", link.up()))
                                .child(Detail::tile("Uploaded", link.sent())),
                        ),
                );
                views.push(Separator::new().into());
                views.push(section.render());
            }
        }

        views
    }
}
