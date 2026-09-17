//! Search installed desktop applications and launch a selected entry.

mod history;
mod search;
use desktop_ui::{ItemRow, PanelHeader};
use history::History;
use omega::record::{Own, Watch};
use omega::{
    Surface, View,
    keyboard::{Chord, Key, Keymap},
    platform::applications::{Application, ApplicationId, Applications, Launcher},
    surface::{Events, Lifecycle, Presentation, Task, TextEdit, TextValue},
    ui::{Button, Column, Component, Field, Glyph, Icon, Image, List, Row, Size, Text},
};
use search::Search;
use std::convert::Infallible;

/// Search presentation settings. Search state belongs to each panel instance.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Maximum returned rows, clamped to 1–100. Defaults to 40.
    pub max_results: u8,
    /// Include an application's category or description in each row. Defaults to true.
    pub show_descriptions: bool,
    /// Content width, clamped to 320–800 logical pixels. Defaults to 416.
    pub width: u32,
    /// Visible result rows, clamped to 3–10. Defaults to 7.
    pub visible_rows: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_results: 40,
            show_descriptions: true,
            width: 416,
            visible_rows: 7,
        }
    }
}

/// A launcher button that remains available while the catalogue initializes.
#[derive(Debug, omega::Surface)]
pub struct Indicator {}
impl Surface for Indicator {
    type Model = ();
    type Message = Infallible;
    type Effects = ();

    fn update(&self, _: &mut (), message: Infallible, _: &()) -> Task<Infallible> {
        match message {}
    }

    fn render(&self, _: &(), _: &Events<Infallible>) -> View {
        Icon::new(Glyph::Search).tooltip("Applications").into()
    }
}

/// Local query, selection, and launch admission state.
#[derive(Debug, Default)]
pub struct Model {
    query: TextValue,
    selected: Option<ApplicationId>,
    pending: bool,
    dismissing: bool,
    epoch: u64,
    error: String,
}

impl Model {
    fn clear(&mut self) {
        self.query.reset("");
        self.selected = None;
        self.error.clear();
    }

    fn busy(&self) -> bool {
        self.pending || self.dismissing
    }
}

/// Typed events from this instance's search field and application list.
#[derive(Debug)]
pub enum Message {
    Edited(TextEdit),
    Selected(ApplicationId),
    Activate(ApplicationId),
    Clear,
    ToggleFavorite,
    Recorded {
        epoch: u64,
        dismiss: bool,
        result: omega::Result<()>,
    },
    Dismiss,
    Launched {
        id: ApplicationId,
        epoch: u64,
        result: omega::Result<()>,
    },
    Dismissed {
        epoch: u64,
        result: omega::Result<()>,
    },
}

/// Effects available to event handling, separate from rendering.
#[derive(Debug, omega::Effects)]
pub struct Effects {
    launcher: Launcher,
    history: Own<History>,
    presentation: Presentation,
}

/// Search, keyboard navigation, and application activation.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    applications: Applications,
    history: Watch<History>,
    #[omega(config)]
    settings: Settings,
}

impl Panel {
    fn dismiss(model: &mut Model, effects: &Effects) -> Task<Message> {
        if model.dismissing {
            return Task::none();
        }

        model.dismissing = true;
        let epoch = model.epoch;
        Task::perform(effects.presentation.hide(), move |result| {
            Message::Dismissed { epoch, result }
        })
    }

    fn row(app: &Application, descriptions: bool, history: &History) -> View {
        let icon: View = if app.icon().is_empty() {
            Icon::new(Glyph::Search).size(Size::Title).into()
        } else {
            Image::icon(app.icon()).width(28).height(28).into()
        };

        let mut row = ItemRow::new(Text::new(app.name()).size(Size::Heading).bold()).leading(icon);

        let description = if app.generic_name().is_empty() {
            app.description()
        } else {
            app.generic_name()
        };

        if descriptions && !description.is_empty() {
            row = row.subtitle(Text::new(description).size(Size::Body).muted());
        }

        if history.favorite(app.id()) {
            row = row.trailing(Text::new("★").tooltip("Favorite"));
        } else if history.recent.iter().any(|id| id == app.id().as_str()) {
            row = row.trailing(Text::new("Recent").size(Size::Body).muted());
        } else if app.is_terminal() {
            row = row.trailing(Icon::new(Glyph::Terminal).tooltip("Runs in a terminal"));
        }

        row.render()
            .key(app.id().as_str())
            .tooltip(format!("{}\n{}", app.description(), app.id()))
    }
}

impl Surface for Panel {
    type Model = Model;
    type Message = Message;
    type Effects = Effects;

    fn update(&self, model: &mut Model, message: Message, effects: &Effects) -> Task<Message> {
        match message {
            Message::Edited(edit) => {
                if !model.busy() && model.query.apply(edit) {
                    model.selected = None;
                    model.error.clear();
                }
            }

            Message::Selected(id) => {
                if !model.busy() {
                    let entries = self.applications.entries().unwrap_or_default();

                    if Search::find(
                        &entries,
                        model.query.text(),
                        self.settings.max_results,
                        &self.history.get(),
                    )
                    .contains(&id)
                    {
                        model.selected = Some(id);
                    }
                }
            }

            Message::Activate(id) => {
                if model.busy() {
                    return Task::none();
                }

                let entries = self.applications.entries().unwrap_or_default();

                if !Search::find(
                    &entries,
                    model.query.text(),
                    self.settings.max_results,
                    &self.history.get(),
                )
                .contains(&id)
                {
                    model.error = "This application is no longer in the current results".into();
                    return Task::none();
                }

                model.pending = true;
                model.error.clear();
                model.selected = Some(id.clone());
                let epoch = model.epoch;
                return Task::perform(effects.launcher.launch(&id), move |result| {
                    Message::Launched {
                        id: id.clone(),
                        epoch,
                        result,
                    }
                });
            }

            Message::Clear => {
                if !model.busy() {
                    model.clear();
                }
            }

            Message::Dismiss => return Self::dismiss(model, effects),
            Message::Launched { id, epoch, result } => match result {
                Ok(()) => {
                    let publication = effects.history.update(|history| history.launched(&id));
                    return Task::perform(publication, move |result| Message::Recorded {
                        epoch,
                        dismiss: true,
                        result,
                    });
                }
                Err(error) => {
                    model.pending = false;
                    if epoch == model.epoch {
                        model.error = error.to_string();
                    }
                }
            },

            Message::ToggleFavorite => {
                if model.busy() {
                    return Task::none();
                }
                let entries = self.applications.entries().unwrap_or_default();
                let matches = Search::find(
                    &entries,
                    model.query.text(),
                    self.settings.max_results,
                    &self.history.get(),
                );
                if let Some(app) = matches.selected(model.selected.as_ref()) {
                    model.selected = Some(app.id().clone());
                    let publication = effects.history.update(|history| history.toggle(app.id()));
                    let epoch = model.epoch;
                    return Task::perform(publication, move |result| Message::Recorded {
                        epoch,
                        dismiss: false,
                        result,
                    });
                }
            }

            Message::Recorded {
                epoch,
                dismiss,
                result,
            } => {
                if dismiss {
                    model.pending = false;
                }
                if epoch != model.epoch {
                    return Task::none();
                }
                match result {
                    Ok(()) if dismiss => return Self::dismiss(model, effects),
                    Ok(()) => {}
                    Err(error) => {
                        model.error = format!("Could not retain launcher history: {error}")
                    }
                }
            }

            Message::Dismissed { epoch, result } => {
                if epoch == model.epoch {
                    model.dismissing = false;

                    if let Err(error) = result {
                        model.error = format!("Could not dismiss the launcher: {error}");
                    }
                }
            }
        }

        Task::none()
    }

    fn lifecycle(&self, model: &mut Model, event: Lifecycle, _: &Effects) -> Task<Message> {
        model.epoch += 1;

        match event {
            Lifecycle::Presented => {}
            Lifecycle::Hidden => {
                model.clear();
                model.dismissing = false;
            }

            Lifecycle::Closed => {
                model.clear();
                model.dismissing = false;
                model.pending = false;
            }
        }

        Task::none()
    }

    fn render(&self, model: &Model, events: &Events<Message>) -> View {
        let entries = self.applications.entries();
        let history = self.history.get();
        let matches = Search::find(
            entries.as_deref().unwrap_or_default(),
            model.query.text(),
            self.settings.max_results,
            &history,
        );

        let selected = matches
            .selected(model.selected.as_ref())
            .map(|app| app.id().to_string())
            .unwrap_or_default();
        let status = if model.pending {
            "Opening application…".into()
        } else if model.dismissing {
            "Closing…".into()
        } else {
            match &entries {
                Some(entries) => format!("{} applications", entries.len()),
                None => "Catalogue unavailable".into(),
            }
        };

        let mut panel = Column::new()
            .width(self.settings.width.clamp(320, 800))
            .gap(12)
            .shortcuts(Keymap::single(
                Chord::new(Key::Escape),
                events.on(|()| Message::Dismiss),
            ))
            .child(
                PanelHeader::new(Text::new("Applications").size(Size::Heading).bold())
                    .subtitle(Text::new(&status).size(Size::Body).muted())
                    .leading(Icon::new(Glyph::Search).size(Size::Display))
                    .render(),
            )
            .child(
                Column::new()
                    .gap(6)
                    .child(Text::new("Search").size(Size::Title))
                    .child(
                        Row::new()
                            .gap(8)
                            .child(
                                // A fresh field on each opening restores search focus from any prior control.
                                Field::new("")
                                    .size(Size::Title)
                                    .key(format!("query-{}", model.epoch))
                                    .fill_width()
                                    .autofocus()
                                    .placeholder("Name, category or keyword")
                                    .controlled(&model.query)
                                    .on_change(events.on(Message::Edited))
                                    .navigate("results")
                                    .disabled_if(model.busy()),
                            )
                            .child(
                                Button::new("Clear")
                                    .key("clear")
                                    .secondary()
                                    .on_press(events.on(|()| Message::Clear))
                                    .disabled_if(model.busy() || model.query.text().is_empty()),
                            ),
                    ),
            )
            .child(
                List::new()
                    .id("results")
                    .key("results")
                    .gap(2)
                    .selected(selected)
                    .height(
                        u32::from(self.settings.visible_rows.clamp(3, 10))
                            * if self.settings.show_descriptions {
                                54
                            } else {
                                38
                            },
                    )
                    .disabled_if(model.busy())
                    .on_select(events.on(Message::Selected))
                    .on_activate(events.on(Message::Activate))
                    .children(
                        matches
                            .items
                            .iter()
                            .map(|app| Self::row(app, self.settings.show_descriptions, &history)),
                    ),
            );

        let selected_app = matches.selected(model.selected.as_ref());

        let summary = if matches.items.is_empty() {
            match &entries {
                None => "Application catalogue unavailable".into(),
                Some(entries) if entries.is_empty() => "No installed applications".into(),
                Some(_) => "No matching applications".into(),
            }
        } else if matches.total > matches.items.len() {
            format!(
                "Showing {} of {} matches · refine your search",
                matches.items.len(),
                matches.total
            )
        } else if model.query.text().trim().is_empty() {
            "Favorites · Recent · All applications".into()
        } else {
            format!("{} matches", matches.total)
        };
        let status_text = if model.error.is_empty() {
            Text::new(&summary).muted()
        } else {
            Text::new(&model.error).tooltip(&model.error).warning()
        };
        panel = panel.child(
            Row::new()
                .key("footer")
                .gap(12)
                .height(40)
                .child(
                    status_text
                        .key("empty")
                        .height(32)
                        .fill_width()
                        .size(Size::Body),
                )
                .child(
                    Button::new(
                        if selected_app.is_some_and(|app| history.favorite(app.id())) {
                            "Remove favorite"
                        } else {
                            "Add favorite"
                        },
                    )
                    .key("favorite")
                    .width(144)
                    .secondary()
                    .disabled_if(model.busy() || selected_app.is_none())
                    .on_press(events.on(|()| Message::ToggleFavorite)),
                ),
        );

        panel
            .child(
                Text::new("↑↓ Select    Enter Open    Esc Close")
                    .height(20)
                    .size(Size::Body)
                    .muted(),
            )
            .into()
    }
}

/// Register the bar button and application search panel.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Indicator).surface(Panel)
}

#[cfg(test)]
mod fixtures;

#[cfg(test)]
mod previews;

#[cfg(test)]
mod tests;
