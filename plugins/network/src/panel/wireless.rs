use super::{Message, Model, NetworkIcon};
use omega::{
    View,
    platform::network::{AccessPoint, Wifi, WifiPhase},
    surface::Events,
    ui::{Button, Field, Glyph, Icon, List, Row, Size, Spacer, Text},
};

pub(super) struct Wireless {
    points: Vec<AccessPoint>,
    selected: Option<String>,
    busy: bool,
    pub phase: WifiPhase,
    available: bool,
}

impl Wireless {
    pub fn new(wifi: &Wifi, model: &Model) -> Self {
        let phase = wifi.phase();
        let mut points = wifi.networks();
        points.sort_by(|left, right| {
            right
                .is_active()
                .cmp(&left.is_active())
                .then(
                    right
                        .strength()
                        .whole_percent()
                        .cmp(&left.strength().whole_percent()),
                )
                .then(left.ssid().cmp(right.ssid()))
        });
        let selected = model.selected.as_deref().or_else(|| {
            points
                .iter()
                .find(|point| point.is_active())
                .map(|point| point.ssid())
        });
        let selected = selected.map(str::to_owned);
        Self {
            points,
            selected,
            busy: model.requesting || phase == WifiPhase::Connecting,
            phase,
            available: wifi.has_reading(),
        }
    }

    pub fn list(&self, events: &Events<Message>) -> View {
        if self.points.is_empty() {
            return Text::new(if !self.available || self.phase == WifiPhase::Unspecified {
                "Wi-Fi unavailable"
            } else {
                "No networks found"
            })
            .muted()
            .into();
        }

        List::new()
            .key("networks")
            .height((self.points.len().min(6) * 38) as u32)
            .gap(2)
            .selected(self.selected.as_deref().unwrap_or_default())
            .disabled_if(self.busy)
            .on_select(events.on(Message::Select))
            .on_activate(events.on(Message::Select))
            .children(self.points.iter().map(Self::row))
            .into()
    }

    fn row(point: &AccessPoint) -> View {
        let mut row = Row::new()
            .gap(8)
            .key(point.ssid())
            .child(Icon::named(NetworkIcon::strength(point.strength())))
            .child(Text::new(point.ssid()).fill_width())
            .child(Spacer::new());

        if point.is_active() {
            row = row.child(Text::new("Connected").size(Size::Caption).muted());
        }

        if point.is_secured() {
            row = row.child(Icon::new(Glyph::Lock).muted());
        }
        row.into()
    }

    pub fn controls(&self, model: &Model, events: &Events<Message>) -> Vec<View> {
        let mut views = Vec::new();
        if let Some(point) = self
            .points
            .iter()
            .find(|point| Some(point.ssid()) == self.selected.as_deref())
            && !point.is_active()
        {
            if point.is_secured() {
                views.push(
                    Field::new("Password")
                        .key("password")
                        .secret()
                        .placeholder("Leave blank to use a saved password")
                        .controlled(&model.password)
                        .disabled_if(self.busy)
                        .on_change(events.on(Message::Password))
                        .into(),
                );
            }

            views.push(
                Button::new(if self.busy {
                    "Connecting…"
                } else {
                    "Connect"
                })
                .key("connect")
                .fill_width()
                .disabled_if(self.busy)
                .on_press(events.on(|()| Message::Connect))
                .into(),
            );
        }

        if self.phase == WifiPhase::Connected {
            views.push(
                Button::new("Disconnect Wi-Fi")
                    .key("disconnect")
                    .secondary()
                    .fill_width()
                    .disabled_if(self.busy)
                    .on_press(events.on(|()| Message::Disconnect))
                    .into(),
            );
        }

        views
    }
}
