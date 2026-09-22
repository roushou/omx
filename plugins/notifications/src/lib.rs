//! Notifications Omega has raised, plus a battery alert that raises them.
//!
//! The center is a standalone overlay; summon it from a compositor binding:
//!
//! ```sh
//! omega present notifications center --overlay --dismiss-on-outside --width 360 --height 320
//! ```
//!
//! The battery alert reacts to low and critical crossings and raises a
//! notification carrying the battery level that crossed.

use omega::platform::notification::{Notification, Notifications, Notify};
use omega::surface::{Events, Task};
use omega::ui::{
    Column, Component, EmptyState, Glyph, Icon, ItemRow, PanelHeader, Section, Size, Text,
};
use omega::{Reaction, Surface, View};
use omega_proto::omega::{Event, EventKind};
use std::convert::Infallible;

/// The in-flight notifications Omega has raised, newest last.
#[derive(Debug, omega::Surface)]
pub struct Center {
    notifications: Notifications,
}

impl Surface for Center {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let raised = self.notifications.active();
        let mut panel = Column::new().width(360).gap(12).child(
            PanelHeader::labelled("Notifications", format!("{} active", raised.len()))
                .leading(Icon::new(Glyph::Bell).size(Size::Display))
                .render(),
        );

        if raised.is_empty() {
            panel = panel.child(EmptyState::new("Nothing raised").icon(Glyph::Bell));
        } else {
            let mut section = Section::new();
            for notification in raised {
                section = section.child(
                    ItemRow::new(Text::new(notification.summary).bold())
                        .subtitle(Text::new(notification.body).muted())
                        .leading(Icon::new(Glyph::Bell)),
                );
            }
            panel = panel.child(section);
        }

        panel.into()
    }
}

/// Raise a notification when the battery crosses a low or critical threshold.
/// The payload carries the level that crossed, matching the event detail.
#[derive(Debug, omega::Reaction)]
pub struct BatteryAlert {
    notify: Notify,
}

impl Reaction for BatteryAlert {
    fn fire(&self, event: &Event) {
        let Some(omega_proto::omega::event::Detail::Power(power)) = &event.detail else {
            return;
        };
        let percent = (power.battery_percent * 100.0).round() as u8;
        let (summary, body) = if event.kind == EventKind::EventBatteryCritical as i32 {
            (
                "Battery critical",
                format!("{percent}% remaining — plug in now"),
            )
        } else {
            ("Battery low", format!("{percent}% remaining"))
        };

        let notification = Notification::new(summary)
            .body(body)
            .icon("battery-caution");
        if let Ok(receipt) = self.notify.show(notification).receipt() {
            receipt.detach();
        }
    }
}

/// Register the center and the battery alert.
pub fn plugin() -> omega::Plugin {
    omega::plugin!()
        .surface(Center)
        .on::<BatteryAlert>(EventKind::EventBatteryLow)
        .on::<BatteryAlert>(EventKind::EventBatteryCritical)
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
