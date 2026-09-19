//! Battery state and supported power profiles.

use power_commands::ChangeProfile;

use desktop_ui::{Detail, PanelHeader, Section};
use omega::{
    Percent, Surface, View,
    platform::power::{Battery, Power, PowerProfile, PowerProfiles, ProfileLabel, Status},
    surface::{Events, Task},
    ui::{Choice, Column, Component, Glyph, Header, Icon, Progress, Row, Separator, Size, Text},
};
use std::convert::Infallible;

/// Battery presentation preferences.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Charge below this percentage is highlighted while unplugged. Defaults to 20.
    pub low: u8,
    /// Show the charge beside the bar icon. Defaults to false.
    pub show_percentage: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            low: 20,
            show_percentage: true,
        }
    }
}

/// Charge/charging indicator; a plug keeps power profiles accessible without a battery.
#[derive(Debug, omega::Surface)]
pub struct Indicator {
    battery: Battery,
    power: Power,
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
        if !self.battery.has_reading() {
            return Icon::new(Glyph::Plug)
                .tooltip("Power profiles · Battery unavailable")
                .into();
        }

        let charge = self.battery.charge();
        let mut row = Row::new()
            .gap(6)
            .tooltip(format!(
                "Battery · {charge} · {}",
                self.power.status().label()
            ))
            .child(Icon::named(BatteryIcon::of(
                charge,
                self.battery.is_charging(),
                self.power.status(),
            )));

        if self.settings.show_percentage {
            row = row.child(Text::new(charge));
        }

        if self.battery.charge() < Percent::whole(self.settings.low)
            && !self.battery.is_charging()
            && !self.power.on_mains()
        {
            row = row.warning();
        }

        row.into()
    }
}

/// Battery charge, time remaining and available power profiles.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    battery: Battery,
    power: Power,
    profiles: PowerProfiles,
    #[omega(config)]
    settings: Settings,
}

/// Command dependencies used by this surface's bindings.
#[derive(Debug, omega::Effects)]
pub struct CommandEffects {
    _change_profile: omega::command::Caller<ChangeProfile>,
}

impl Surface for Panel {
    type Model = ();
    type Message = Infallible;
    type Effects = CommandEffects;

    fn update(&self, _: &mut (), message: Infallible, _: &Self::Effects) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        let mut panel = Column::new().width(352).gap(14);

        if self.battery.has_reading() {
            let charge = self.battery.charge();
            let status = self.power.status();
            let low = charge < Percent::whole(self.settings.low)
                && !self.battery.is_charging()
                && !self.power.on_mains();
            let header = PanelHeader::labelled("Battery", status.label())
                .leading(
                    Icon::named(BatteryIcon::of(charge, self.battery.is_charging(), status))
                        .size(Size::Display),
                )
                .trailing(Text::new(charge).size(Size::Display).bold())
                .render();
            let progress = Progress::new(charge).fill_width().key("charge");
            panel = panel
                .child(if low { header.warning() } else { header })
                .child(if low { progress.warning() } else { progress });

            if self.battery.is_charging() {
                if let Some(time) = self.battery.until_full() {
                    panel = panel.child(Detail::row("Time to full", time));
                }
            } else if let Some(time) = self.battery.until_empty() {
                panel = panel.child(Detail::row("Time left", time));
            }
        } else {
            panel = panel.child(
                PanelHeader::labelled("Power", "Battery unavailable")
                    .leading(Icon::new(Glyph::Plug).size(Size::Display))
                    .render(),
            );
        }

        let mut section = Section::new().heading(Header::new("POWER PROFILE"));
        let profiles = self.profiles.available();

        if profiles.is_empty() {
            section = section.child(Text::new("Power profiles unavailable").muted());
        } else {
            section = section.child(
                Choice::new()
                    .key("profiles")
                    .options(profiles.into_iter().map(|profile| {
                        (
                            profile,
                            Row::new()
                                .gap(6)
                                .child(Icon::named(match profile {
                                    PowerProfile::Saver => "󰌪",
                                    PowerProfile::Balanced => "󰊚",
                                    PowerProfile::Performance => "󰓅",
                                    PowerProfile::Unspecified => "?",
                                }))
                                .child(Text::new(profile.label())),
                        )
                    }))
                    .selected(self.profiles.active())
                    .on_select(ChangeProfile),
            );
        }

        if let Some(reason) = self.profiles.degraded() {
            section = section.child(Text::new(format!("Performance limited: {reason}")).warning());
        }

        panel = panel.child(Separator::new()).child(section.render());

        panel.into()
    }
}

struct BatteryIcon;

impl BatteryIcon {
    fn of(charge: Percent, charging: bool, status: Status) -> &'static str {
        if status == Status::FullyCharged {
            return "󰂅";
        }

        let ramp = if charging {
            ["󰢜", "󰂆", "󰂇", "󰂈", "󰢝", "󰂉", "󰢞", "󰂊", "󰂋", "󰂅"]
        } else {
            ["󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"]
        };
        ramp[usize::from(charge.whole_percent() / 10).min(9)]
    }
}

/// Register the surfaces and typed profile control.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).surface(Panel)
}

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
