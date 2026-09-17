use omega::{
    View,
    ui::{Column, Component, Row, Size, Spacer, Text},
};
use std::fmt::Display;

/// A label and trailing content separated by flexible space.
/// Either slot accepts a view, including controls. The gap defaults to 12 pixels.
///
/// ```
/// use desktop_ui::Detail;
/// use omega::ui::{Text, Toggle};
/// let row = Detail::new(Text::new("Enabled"), Toggle::new(true)).gap(8);
/// ```
pub struct Detail {
    label: View,
    value: View,
    gap: u32,
}

impl Detail {
    pub fn new(label: impl Into<View>, value: impl Into<View>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            gap: 12,
        }
    }

    pub fn gap(mut self, pixels: u32) -> Self {
        self.gap = pixels;
        self
    }

    /// A muted text label and a text value in the standard detail row.
    pub fn row(label: impl Display, value: impl Display) -> View {
        Self::new(Text::new(label).muted(), Text::new(value)).render()
    }

    /// A muted caption above a text value, separated by 2 pixels.
    pub fn tile(label: &str, value: impl Display) -> View {
        Column::new()
            .gap(2)
            .child(Text::new(label).size(Size::Caption).muted())
            .child(Text::new(value))
            .into()
    }
}

impl Component for Detail {
    fn render(&self) -> View {
        Row::new()
            .gap(self.gap)
            .child(self.label.clone())
            .child(Spacer::new())
            .child(self.value.clone())
            .into()
    }
}
