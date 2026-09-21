use super::{
    Settings,
    calendar::{Month, Year},
    reading::Reading,
};
use chrono::Datelike;
use omega::ui::{Labelled, PanelHeader};
use omega::{
    Percent, Surface, View,
    keyboard::{Chord, Key, Keymap},
    platform::time::Clock,
    surface::{Events, Lifecycle, Task},
    ui::{
        Bind, Button, Column, Component, Glyph, Grid, Icon, Progress, Role, Row, Separator, Size,
        Spacer, Text,
    },
};

/// A month calendar with local navigation. Opening the panel returns to today.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    clock: Clock,
    #[omega(config)]
    settings: Settings,
}

/// Calendar browsing state. None follows the daemon's current month, including midnight.
#[derive(Debug, Default)]
pub struct Model {
    month: Option<Month>,
}

/// Local calendar actions. Month shifts do not change the system clock.
#[derive(Debug, Clone)]
pub enum Message {
    Shift(i32),
    Today,
}

impl Surface for Panel {
    type Model = Model;
    type Message = Message;
    type Effects = ();

    fn update(&self, model: &mut Model, message: Message, _: &()) -> Task<Message> {
        match message {
            Message::Today => model.month = None,
            Message::Shift(delta) => {
                if let Some(today) = Reading::of(&self.clock) {
                    let current = Month::of(today.date);

                    if let Some(month) = model.month.unwrap_or(current).shift(delta) {
                        model.month = (month != current).then_some(month);
                    }
                }
            }
        }

        Task::none()
    }

    fn lifecycle(&self, model: &mut Model, event: Lifecycle, _: &()) -> Task<Message> {
        if matches!(event, Lifecycle::Presented | Lifecycle::Closed) {
            model.month = None;
        }

        Task::none()
    }

    fn render(&self, model: &Model, events: &Events<Message>) -> View {
        let Some(today) = Reading::of(&self.clock) else {
            return Text::new("Clock unavailable").muted().into();
        };

        let month = model.month.unwrap_or_else(|| Month::of(today.date));
        let mut panel = Column::new()
            .width(420)
            .gap(14)
            .key("calendar")
            .shortcuts(Navigation::keys(events))
            .child(Self::header(&today, &self.settings, events))
            .child(Separator::new())
            .child(Navigation::view(month, events));

        panel = panel.child(
            MonthGrid {
                month,
                today: today.date,
                settings: &self.settings,
                width: 420,
            }
            .render(),
        );

        if self.settings.show_year_progress {
            panel = panel
                .child(Separator::new())
                .child(Self::year_progress(today.date));
        }

        panel
            .child(
                Text::new("← → month · ↑ ↓ year · Home today")
                    .muted()
                    .size(Size::Caption),
            )
            .into()
    }
}

impl Panel {
    fn header(today: &Reading, settings: &Settings, events: &Events<Message>) -> View {
        PanelHeader::new(
            Text::new(today.date.format("%B %-d"))
                .size(Size::Title)
                .bold()
                .key("today-date"),
        )
        .leading(Icon::new(Glyph::Calendar).size(Size::Display))
        .subtitle(
            Text::new(format!(
                "{} · {}",
                today.date.format("%A"),
                today.time(settings),
            ))
            .muted()
            .size(Size::Caption),
        )
        .trailing(
            Button::new("Today")
                .secondary()
                .key("today")
                .on_press(events.send(Message::Today)),
        )
        .render()
    }

    fn year_progress(date: chrono::NaiveDate) -> View {
        let elapsed = Percent::of(Year::progress(date));
        Labelled::new(
            Text::new(date.year()).muted(),
            Text::new(elapsed).muted().key("year-share"),
            Progress::new(elapsed)
                .fill_width()
                .key("year-progress")
                .tooltip("Completed days in the current year"),
        )
        .gap(14)
        .label_gap(0)
        .render()
    }
}

struct MonthGrid<'a> {
    month: Month,
    today: chrono::NaiveDate,
    settings: &'a Settings,
    width: u32,
}

impl Component for MonthGrid<'_> {
    fn render(&self) -> View {
        let columns = if self.settings.show_week_numbers {
            8
        } else {
            7
        };

        let cell_width = (self.width - (columns - 1) * 2) / columns;
        let mut grid = Grid::new(columns).gap(2);

        if self.settings.show_week_numbers {
            grid = grid.child(Cell::view(
                Text::new("W")
                    .muted()
                    .size(Size::Caption)
                    .tooltip("ISO week number"),
                "week-heading",
                cell_width,
            ));
        }

        let names = if self.settings.monday_first {
            ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"]
        } else {
            ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"]
        };

        for name in names {
            grid = grid.child(Cell::view(
                Text::new(name).muted().size(Size::Caption),
                format!("weekday-{name}"),
                cell_width,
            ));
        }

        for (index, week) in self
            .month
            .weeks(self.settings.monday_first)
            .into_iter()
            .enumerate()
        {
            if self.settings.show_week_numbers {
                grid = grid.child(Cell::view(
                    Text::new(format!("{:02}", week.number))
                        .muted()
                        .size(Size::Caption),
                    format!("week-{index}"),
                    cell_width,
                ));
            }

            for date in week.days {
                let text = Text::new(date.day()).tooltip(date.format("%A, %-d %B %Y").to_string());
                let text = if date == self.today {
                    text.bold().color(Role::Accent)
                } else if !self.month.contains(date) {
                    text.muted()
                } else {
                    text
                };
                grid = grid.child(Cell::view(text, format!("day-{date}"), cell_width));
            }
        }

        grid.into()
    }
}

struct Cell;

impl Cell {
    fn view(text: Text, key: impl Into<String>, width: u32) -> View {
        Row::new()
            .width(width)
            .height(32)
            .key(key)
            .child(Spacer::new())
            .child(text)
            .child(Spacer::new())
            .into()
    }
}

struct Navigation;

impl Navigation {
    fn view(month: Month, events: &Events<Message>) -> View {
        Row::new()
            .gap(6)
            .child(Navigation::button(
                "«",
                "previous-year",
                "Previous year · ↑ / k",
                month,
                -12,
                events,
            ))
            .child(Navigation::button(
                "‹",
                "previous-month",
                "Previous month · ← / h",
                month,
                -1,
                events,
            ))
            .child(Spacer::new())
            .child(Text::new(month.title()).bold().key("month"))
            .child(Spacer::new())
            .child(Navigation::button(
                "›",
                "next-month",
                "Next month · → / l",
                month,
                1,
                events,
            ))
            .child(Navigation::button(
                "»",
                "next-year",
                "Next year · ↓ / j",
                month,
                12,
                events,
            ))
            .into()
    }

    fn button(
        label: &str,
        key: &str,
        tooltip: &str,
        month: Month,
        delta: i32,
        events: &Events<Message>,
    ) -> Button {
        Button::new(label)
            .width(32)
            .secondary()
            .key(key)
            .tooltip(tooltip)
            .disabled_if(month.shift(delta).is_none())
            .on_press(events.send(Message::Shift(delta)))
    }

    fn keys(events: &Events<Message>) -> Keymap<Bind<()>> {
        let mut keys = Keymap::single(Chord::new(Key::Home), events.send(Message::Today));

        for (key, delta) in [
            (Key::ArrowLeft, -1),
            (Key::ArrowRight, 1),
            (Key::ArrowUp, -12),
            (Key::ArrowDown, 12),
            (Key::PageUp, -1),
            (Key::PageDown, 1),
            (Key::Character('h'), -1),
            (Key::Character('l'), 1),
            (Key::Character('k'), -12),
            (Key::Character('j'), 12),
        ] {
            keys = keys
                .bind(Chord::new(key).repeat(), events.send(Message::Shift(delta)))
                .expect("calendar shortcuts are unique");
        }

        keys.bind(Chord::new(Key::Character('t')), events.send(Message::Today))
            .expect("today shortcut is unique")
    }
}
