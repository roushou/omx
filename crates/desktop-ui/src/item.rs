use omega::{
    View,
    ui::{Column, Component, Row},
};

/// A leading view, a growing title/subtitle column, and a trailing view.
/// Slots accept arbitrary views and retain their bindings. The row adds no
/// selection or click behavior. Horizontal spacing defaults to 12 pixels;
/// title/subtitle spacing defaults to 2 pixels.
///
/// ```
/// use desktop_ui::ItemRow;
/// use omega::ui::{Component, Glyph, Icon, Text};
/// let row = ItemRow::new(Text::new("Keyboard").bold())
///     .leading(Icon::new(Glyph::Bluetooth))
///     .subtitle(Text::new("Connected").muted())
///     .trailing(Text::new("76%"))
///     .key("keyboard");
/// ```
pub struct ItemRow {
    title: View,
    subtitle: View,
    leading: View,
    trailing: View,
    gap: u32,
    content_gap: u32,
}

impl ItemRow {
    pub fn new(title: impl Into<View>) -> Self {
        Self {
            title: title.into(),
            subtitle: View::empty(),
            leading: View::empty(),
            trailing: View::empty(),
            gap: 12,
            content_gap: 2,
        }
    }

    pub fn subtitle(mut self, view: impl Into<View>) -> Self {
        self.subtitle = view.into();
        self
    }

    pub fn leading(mut self, view: impl Into<View>) -> Self {
        self.leading = view.into();
        self
    }

    pub fn trailing(mut self, view: impl Into<View>) -> Self {
        self.trailing = view.into();
        self
    }

    pub fn gap(mut self, pixels: u32) -> Self {
        self.gap = pixels;
        self
    }

    pub fn content_gap(mut self, pixels: u32) -> Self {
        self.content_gap = pixels;
        self
    }
}

impl Component for ItemRow {
    fn render(&self) -> View {
        Row::new()
            .gap(self.gap)
            .child(self.leading.clone())
            .child(
                Column::new()
                    .fill_width()
                    .gap(self.content_gap)
                    .child(self.title.clone())
                    .child(self.subtitle.clone()),
            )
            .child(self.trailing.clone())
            .into()
    }
}
