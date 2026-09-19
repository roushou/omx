//! Search applications and typed desktop actions, with optional semantic ranking.

mod candidates;
mod favorites;
mod search;
use candidates::{Candidate, CandidateId, Sources};
use desktop_ui::{ItemRow, PanelHeader};
use favorites::{AllFavorites, Favorites};
use omega::command::{Available, Caller, Commands};
use omega::platform::{audio::Media, bluetooth::Bluetooth};
use omega::storage::{Snapshot, Store, Subscribed};
use omega::surface::Optional;
use omega::{
    Surface, View,
    keyboard::{Chord, Key, Keymap},
    platform::applications::{ApplicationId, Applications, Launcher},
    surface::{Events, Lifecycle, Presentation, Task, TextEdit, TextValue},
    ui::{Button, Column, Component, Field, Glyph, Icon, Image, List, Row, Size, Text},
};
use search::Search;
use std::{collections::BTreeSet, convert::Infallible, future::Future};

/// Search presentation settings. Search state belongs to each panel instance.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Enable the explicit semantic-search button. Requests send query and candidate metadata to TypeSafe.
    pub semantic_search: bool,
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
            semantic_search: false,
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
    selected: Option<CandidateId>,
    catalogue: Vec<Available>,
    semantic: Option<typesafe::Ranking>,
    semantic_pending: bool,
    jev: Option<typesafe::Client>,
    pending: bool,
    saving: bool,
    dismissing: bool,
    epoch: u64,
    error: String,
}

impl Model {
    fn clear(&mut self) {
        self.query.reset("");
        self.semantic = None;
        self.semantic_pending = false;
        self.selected = None;
        self.error.clear();
    }

    fn busy(&self) -> bool {
        self.pending || self.saving || self.dismissing
    }
}

/// Typed events from this instance's search field and application list.
#[derive(Debug)]
pub enum Message {
    Edited(TextEdit),
    Selected(CandidateId),
    Activate(CandidateId),
    Semantic,
    Ranked {
        epoch: u64,
        query: String,
        result: omega::Result<typesafe::Ranking>,
    },
    CommandsLoaded {
        epoch: u64,
        result: omega::Result<Vec<Available>>,
    },
    Cancelled,
    Clear,
    ToggleFavorite,
    FavoriteSaved {
        epoch: u64,
        result: omega::Result<()>,
    },
    Dismiss,
    Launched {
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
    commands: Commands,
    audible: Caller<audio_commands::SetAudible>,
    volume: Caller<audio_commands::SetVolume>,
    brightness: Caller<display_commands::SetBrightness>,
    connect: Caller<bluetooth_commands::Connect>,
    disconnect: Caller<bluetooth_commands::Disconnect>,
    play: Caller<media_commands::Play>,
    pause: Caller<media_commands::Pause>,
    next: Caller<media_commands::Next>,
    launcher: Launcher,
    favorites: Store<Favorites>,
    presentation: Presentation,
}

impl Effects {
    fn add_favorite(
        &self,
        id: ApplicationId,
    ) -> impl Future<Output = omega::Result<()>> + Send + 'static {
        let favorites = self.favorites.clone();
        async move {
            favorites.insert(id, ()).await?;
            Ok(())
        }
    }

    fn remove_favorite(
        &self,
        id: ApplicationId,
    ) -> impl Future<Output = omega::Result<()>> + Send + 'static {
        let favorites = self.favorites.clone();
        async move {
            if let Some(entry) = favorites.get(&id).await? {
                favorites.remove(id, entry.revision).await?;
            }
            Ok(())
        }
    }
}

/// Search, keyboard navigation, and application activation.
#[derive(Debug, omega::Surface)]
pub struct Panel {
    media: Optional<Media>,
    bluetooth: Optional<Bluetooth>,
    applications: Applications,
    favorites: Subscribed<AllFavorites>,
    #[omega(config)]
    settings: Settings,
}

impl Panel {
    fn entries(&self, model: &Model) -> Vec<Candidate> {
        Sources::collect(
            self.applications.entries().unwrap_or_default(),
            &model.catalogue,
            &self.media,
            &self.bluetooth,
        )
    }
    fn matches<'a>(
        &self,
        entries: &'a [Candidate],
        model: &Model,
        favorites: &BTreeSet<ApplicationId>,
    ) -> search::Matches<'a> {
        match &model.semantic {
            Some(ranking) => Search::semantic(entries, &ranking.scores, self.settings.max_results),
            None => Search::find(
                entries,
                model.query.text(),
                self.settings.max_results,
                favorites,
            ),
        }
    }
    fn refresh(model: &Model, effects: &Effects) -> Task<Message> {
        let epoch = model.epoch;
        Task::replace("commands", effects.commands.list(), move |result| {
            Message::CommandsLoaded { epoch, result }
        })
    }

    fn favorites(&self) -> BTreeSet<ApplicationId> {
        match self.favorites.snapshot() {
            Snapshot::Ready(page) => page
                .entries()
                .iter()
                .map(|entry| entry.key.clone())
                .collect(),
            Snapshot::Loading | Snapshot::Failed(_) => BTreeSet::new(),
        }
    }

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

    fn row(app: &Candidate, descriptions: bool, favorites: &BTreeSet<ApplicationId>) -> View {
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

        if app.is_favorite(favorites) {
            row = row.trailing(Text::new("★").tooltip("Favorite"));
        } else if app.is_terminal() {
            row = row.trailing(Icon::new(Glyph::Terminal).tooltip("Runs in a terminal"));
        }

        row.render().key(app.id().to_string()).tooltip(format!(
            "{}\n{}",
            app.description(),
            app.id()
        ))
    }
}

impl Surface for Panel {
    type Model = Model;
    type Message = Message;
    type Effects = Effects;

    fn initialize(&mut self, _: &mut Model) -> omega::Result<()> {
        self.favorites.start(AllFavorites)
    }

    fn mounted(&self, model: &mut Model, effects: &Effects) -> Task<Message> {
        Self::refresh(model, effects)
    }

    fn update(&self, model: &mut Model, message: Message, effects: &Effects) -> Task<Message> {
        match message {
            Message::Cancelled => {}
            Message::CommandsLoaded { epoch, result } => {
                if epoch == model.epoch {
                    match result {
                        Ok(entries) => model.catalogue = entries,
                        Err(error) => model.error = format!("Commands unavailable: {error}"),
                    }
                }
            }
            Message::Semantic => {
                if model.busy()
                    || !self.settings.semantic_search
                    || model.query.text().trim().is_empty()
                {
                    return Task::none();
                }
                let client = match model
                    .jev
                    .clone()
                    .map(Ok)
                    .unwrap_or_else(typesafe::Client::from_env)
                {
                    Ok(client) => client,
                    Err(error) => {
                        model.error = error.to_string();
                        return Task::none();
                    }
                };
                model.jev = Some(client.clone());
                let candidates: Vec<_> = self
                    .entries(model)
                    .iter()
                    .map(Candidate::semantic)
                    .collect();
                let query = model.query.text().to_owned();
                let requested = query.clone();
                let epoch = model.epoch;
                model.semantic_pending = true;
                model.error.clear();
                return Task::replace(
                    "semantic",
                    async move {
                        client
                            .rank(&requested, &candidates)
                            .await
                            .map_err(|e| omega::Error::invalid(e.to_string()))
                    },
                    move |result| Message::Ranked {
                        epoch,
                        query: query.clone(),
                        result,
                    },
                );
            }
            Message::Ranked {
                epoch,
                query,
                result,
            } => {
                if epoch == model.epoch && query == model.query.text() {
                    model.semantic_pending = false;
                    match result {
                        Ok(ranking) => model.semantic = Some(ranking),
                        Err(error) => model.error = error.to_string(),
                    }
                }
            }
            Message::Edited(edit) => {
                if !model.busy() && model.query.apply(edit) {
                    let cancel = model.semantic_pending;
                    model.selected = None;
                    model.semantic = None;
                    model.semantic_pending = false;
                    model.error.clear();
                    if cancel {
                        return Task::replace("semantic", async { Ok(()) }, |_| Message::Cancelled);
                    }
                }
            }

            Message::Selected(id) => {
                if !model.busy() {
                    let entries = self.entries(model);

                    if self
                        .matches(&entries, model, &self.favorites())
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

                let entries = self.entries(model);

                if !self
                    .matches(&entries, model, &self.favorites())
                    .contains(&id)
                {
                    model.error = "This application is no longer in the current results".into();
                    return Task::none();
                }

                model.pending = true;
                model.error.clear();
                model.selected = Some(id.clone());
                let epoch = model.epoch;
                let candidates = self.matches(&entries, model, &self.favorites());
                let Some(candidate) = candidates.items.iter().find(|entry| entry.id() == &id)
                else {
                    return Task::none();
                };
                return Task::perform(candidate.execute(effects), move |result| {
                    Message::Launched { epoch, result }
                });
            }

            Message::Clear => {
                if !model.busy() {
                    model.clear();
                }
            }

            Message::Dismiss => return Self::dismiss(model, effects),
            Message::Launched { epoch, result } => {
                model.pending = false;
                if epoch != model.epoch {
                    return Task::none();
                }

                match result {
                    Ok(()) => return Self::dismiss(model, effects),
                    Err(error) => model.error = error.to_string(),
                }
            }

            Message::ToggleFavorite => {
                if model.busy() || !matches!(self.favorites.snapshot(), Snapshot::Ready(_)) {
                    return Task::none();
                }
                let entries = self.entries(model);
                let matches = self.matches(&entries, model, &self.favorites());
                if let Some(app) = matches.selected(model.selected.as_ref())
                    && let Some(app_id) = app.application()
                {
                    model.selected = Some(app.id().clone());
                    model.saving = true;
                    model.error.clear();
                    let epoch = model.epoch;
                    let completed = move |result| Message::FavoriteSaved { epoch, result };
                    return if self.favorites().contains(app_id) {
                        Task::perform(effects.remove_favorite(app_id.clone()), completed)
                    } else {
                        Task::perform(effects.add_favorite(app_id.clone()), completed)
                    };
                }
            }

            Message::FavoriteSaved { epoch, result } => {
                model.saving = false;
                if epoch == model.epoch
                    && let Err(error) = result
                {
                    model.error = format!("Could not save favorite: {error}");
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

    fn lifecycle(&self, model: &mut Model, event: Lifecycle, effects: &Effects) -> Task<Message> {
        model.epoch += 1;

        match event {
            Lifecycle::Presented => return Self::refresh(model, effects),
            Lifecycle::Hidden => {
                model.clear();
                model.dismissing = false;
            }

            Lifecycle::Closed => {
                model.clear();
                model.dismissing = false;
                model.pending = false;
                model.saving = false;
            }
        }

        Task::none()
    }

    fn render(&self, model: &Model, events: &Events<Message>) -> View {
        let applications = self.applications.entries();
        let entries = self.entries(model);
        let favorites = self.favorites();
        let matches = self.matches(&entries, model, &favorites);

        let selected = matches
            .selected(model.selected.as_ref())
            .map(|app| app.id().to_string())
            .unwrap_or_default();
        let status = if model.pending {
            "Running…".into()
        } else if model.semantic_pending {
            "Finding relevant results…".into()
        } else if model.saving {
            "Saving favorite…".into()
        } else if model.dismissing {
            "Closing…".into()
        } else {
            match &applications {
                Some(apps) => format!(
                    "{} apps · {} actions",
                    apps.len(),
                    entries.len().saturating_sub(apps.len())
                ),
                None => "Catalogue unavailable".into(),
            }
        };

        let mut panel =
            Column::new()
                .width(self.settings.width.clamp(320, 800))
                .gap(12)
                .shortcuts(Keymap::single(
                    Chord::new(Key::Escape),
                    events.on(|()| Message::Dismiss),
                ))
                .child(
                    PanelHeader::new(Text::new("Launcher").size(Size::Heading).bold())
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
                                        .placeholder("Applications, actions or what you want to do")
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
                        .children(matches.items.iter().map(|app| {
                            Self::row(app, self.settings.show_descriptions, &favorites)
                        })),
                );

        let selected_app = matches
            .selected(model.selected.as_ref())
            .and_then(Candidate::application);

        let summary = if matches.items.is_empty() {
            match &applications {
                None => "Application catalogue unavailable".into(),
                Some(entries) if entries.is_empty() => "No installed applications".into(),
                Some(_) => "No matching results".into(),
            }
        } else if matches.total > matches.items.len() {
            format!(
                "Showing {} of {} matches · refine your search",
                matches.items.len(),
                matches.total
            )
        } else if model.query.text().trim().is_empty() {
            "Favorites · Apps and actions".into()
        } else {
            format!("{} matches", matches.total)
        };
        let favorites_state = self.favorites.snapshot();
        let favorites_ready = matches!(favorites_state, Snapshot::Ready(_));
        let status_text = if !model.error.is_empty() {
            Text::new(&model.error).tooltip(&model.error).warning()
        } else {
            match &favorites_state {
                Snapshot::Loading => Text::new("Loading favorites…").muted(),
                Snapshot::Failed(error) => {
                    Text::new("Favorites unavailable").tooltip(error).warning()
                }
                Snapshot::Ready(_) => Text::new(&summary).muted(),
            }
        };
        if self.settings.semantic_search {
            panel = panel.child(
                Button::new("Search by meaning")
                    .key("semantic-search")
                    .secondary()
                    .disabled_if(
                        model.busy()
                            || model.semantic_pending
                            || model.query.text().trim().is_empty(),
                    )
                    .on_press(events.on(|()| Message::Semantic)),
            );
        }
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
                    Button::new(if selected_app.is_some_and(|id| favorites.contains(id)) {
                        "Remove favorite"
                    } else {
                        "Add favorite"
                    })
                    .key("favorite")
                    .width(144)
                    .secondary()
                    .disabled_if(model.busy() || !favorites_ready || selected_app.is_none())
                    .on_press(events.on(|()| Message::ToggleFavorite)),
                ),
        );

        panel
            .child(
                Text::new("↑↓ Select    Enter Run    Esc Close")
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
