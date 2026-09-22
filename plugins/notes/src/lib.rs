//! A persistent scratchpad, presented as an overlay.
//!
//! Summon it from a compositor binding:
//!
//! ```sh
//! omega present notes editor --overlay --dismiss-on-outside --width 440 --height 360
//! ```
//!
//! Edits autosave to `omx.notes.scratch` after a short quiet period and when the
//! overlay is hidden. Escape or a click outside dismisses it.

use std::time::Duration;

use omega::storage::{Backend, Entry, Limits, Revision, Storage, StoragePolicy, Store};
use omega::surface::{Events, Lifecycle, Task, TextEdit, TextValue};
use omega::ui::{Button, Column, Row, Size, Text, TextArea};
use omega::{Surface, View};

/// The single scratchpad entry's key.
const SCRATCH: &str = "scratch";

/// One note body. JSON-compatible, so it persists as `json-v1`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub text: String,
}

/// The scratchpad store. One entry holds the current note.
#[derive(Debug)]
pub struct Scratch;

impl Storage for Scratch {
    type Key = String;
    type Value = Body;

    const ID: &'static str = "omx.notes.scratch";
    const POLICY: StoragePolicy = StoragePolicy::Persistent {
        backend: Backend::Json,
        schema_version: 1,
    };
    const LIMITS: Limits = Limits {
        max_entries: 8,
        max_value_bytes: 16 * 1024,
        max_total_bytes: 128 * 1024,
    };
}

/// Presentation settings.
#[derive(Debug, Clone, omega::Config)]
pub struct Settings {
    /// Visible editor rows before the note scrolls. Clamped to 2–40.
    pub rows: u32,
    /// Quiet period before an edit is written. Zero writes immediately.
    pub autosave_millis: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rows: 10,
            autosave_millis: 500,
        }
    }
}

/// Local editor state.
#[derive(Debug, Default)]
pub struct Model {
    text: TextValue,
    revision: Option<Revision>,
    loaded: bool,
    dirty: bool,
    saving: bool,
    generation: u64,
    error: String,
}

/// Typed events from this instance's editor.
#[derive(Debug)]
pub enum Message {
    Loaded(omega::Result<Option<Entry<String, Body>>>),
    Edited(TextEdit),
    Flush {
        generation: u64,
    },
    Saved {
        generation: u64,
        result: omega::Result<Revision>,
    },
    Clear,
    Failed(String),
}

/// Effects available to event handling, separate from rendering.
#[derive(Debug, omega::Effects)]
pub struct Effects {
    scratch: Store<Scratch>,
}

/// A multi-line scratchpad that autosaves to shared storage.
#[derive(Debug, omega::Surface)]
pub struct Editor {
    #[omega(config)]
    settings: Settings,
}

impl Editor {
    /// A replaceable debounce task. A newer edit aborts the pending one, so only
    /// the last quiet period schedules a flush.
    fn debounce(&self, generation: u64) -> Task<Message> {
        let millis = self.settings.autosave_millis;
        Task::replace(
            "autosave",
            async move {
                if millis > 0 {
                    tokio::time::sleep(Duration::from_millis(millis)).await;
                }
                Ok(generation)
            },
            |result| match result {
                Ok(generation) => Message::Flush { generation },
                Err(error) => Message::Failed(error.to_string()),
            },
        )
    }

    /// Write the current text. The write is not replaceable, so an edit during a
    /// save cannot abort it and lose the revision token.
    fn persist(model: &Model, effects: &Effects, generation: u64) -> Task<Message> {
        let store = effects.scratch.clone();
        let text = model.text.text().to_owned();
        let expected = model.revision.clone();
        Task::perform(
            async move {
                let body = Body { text };
                match expected {
                    Some(revision) => store.replace(SCRATCH.to_string(), revision, body).await,
                    None => store.insert(SCRATCH.to_string(), body).await,
                }
            },
            move |result| Message::Saved { generation, result },
        )
    }

    fn dirty(model: &mut Model) -> Task<Message> {
        model.dirty = true;
        model.generation = model.generation.wrapping_add(1);
        model.error.clear();
        Task::none()
    }

    fn status(model: &Model) -> View {
        if !model.error.is_empty() {
            Text::new(&model.error).warning().into()
        } else if !model.loaded {
            Text::new("Loading…").muted().into()
        } else if model.saving {
            Text::new("Saving…").muted().into()
        } else if model.dirty {
            Text::new("Unsaved").muted().into()
        } else {
            Text::new("Saved").muted().into()
        }
    }
}

impl Surface for Editor {
    type Model = Model;
    type Message = Message;
    type Effects = Effects;

    fn mounted(&self, _: &mut Model, effects: &Effects) -> Task<Message> {
        let store = effects.scratch.clone();
        Task::perform(
            async move { store.get(&SCRATCH.to_string()).await },
            Message::Loaded,
        )
    }

    fn update(&self, model: &mut Model, message: Message, effects: &Effects) -> Task<Message> {
        match message {
            Message::Loaded(result) => {
                model.loaded = true;
                match result {
                    Ok(Some(entry)) => {
                        model.revision = Some(entry.revision);
                        // A load that loses the race with typing must not clobber it.
                        if !model.dirty {
                            model.text.reset(entry.value.text);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => model.error = format!("Could not load: {error}"),
                }
                Task::none()
            }
            Message::Edited(edit) => {
                if model.text.apply(edit) {
                    let _ = Self::dirty(model);
                    return self.debounce(model.generation);
                }
                Task::none()
            }
            Message::Flush { generation } => {
                if generation != model.generation || !model.dirty || model.saving {
                    return Task::none();
                }
                model.saving = true;
                model.dirty = false;
                Self::persist(model, effects, generation)
            }
            Message::Saved { generation, result } => {
                model.saving = false;
                match result {
                    Ok(revision) => {
                        model.revision = Some(revision);
                        model.error.clear();
                        if model.dirty {
                            model.saving = true;
                            model.dirty = false;
                            return Self::persist(model, effects, generation);
                        }
                    }
                    Err(error) => {
                        model.error = format!("Could not save: {error}");
                        model.dirty = true;
                    }
                }
                Task::none()
            }
            Message::Clear => {
                if model.text.text().is_empty() {
                    return Task::none();
                }
                model.text.reset("");
                let _ = Self::dirty(model);
                self.debounce(model.generation)
            }
            Message::Failed(error) => {
                model.error = error;
                Task::none()
            }
        }
    }

    fn lifecycle(&self, model: &mut Model, event: Lifecycle, effects: &Effects) -> Task<Message> {
        // Closed surfaces cannot schedule work, so a pending edit is flushed while
        // hiding. A close before the debounce elapses keeps the last saved text.
        if event == Lifecycle::Hidden && model.dirty && !model.saving {
            model.saving = true;
            model.dirty = false;
            return Self::persist(model, effects, model.generation);
        }
        Task::none()
    }

    fn render(&self, model: &Model, events: &Events<Message>) -> View {
        Column::new()
            .fill_width()
            .gap(8)
            .child(
                Row::new()
                    .gap(8)
                    .child(Text::new("Notes").size(Size::Title).bold())
                    .child(Self::status(model)),
            )
            .child(
                TextArea::new("")
                    .rows(self.settings.rows.clamp(2, 40))
                    .fill_width()
                    .autofocus()
                    .placeholder("Write a note…")
                    .controlled(&model.text)
                    .on_change(events.on(Message::Edited)),
            )
            .child(
                Row::new().gap(8).child(
                    Button::new("Clear")
                        .secondary()
                        .disabled_if(model.text.text().is_empty())
                        .on_press(events.on(|()| Message::Clear)),
                ),
            )
            .into()
    }
}

/// Register the scratchpad surface.
pub fn plugin() -> omega::Plugin {
    omega::plugin!().surface(Editor)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod previews;
