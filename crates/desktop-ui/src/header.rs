use super::ItemRow;
use omega::{
    View,
    ui::{Component, Size, Text},
};
use std::fmt::Display;

/// A heading with optional leading, subtitle, and trailing views.
/// Defaults to a 14-pixel horizontal gap and a 2-pixel title/subtitle gap.
/// The caller owns content styling and the surrounding width.
///
/// ```
/// use desktop_ui::PanelHeader;
/// use omega::ui::{Glyph, Icon, Size, Text};
/// let heading = PanelHeader::new(Text::new("Applications").bold())
///     .leading(Icon::new(Glyph::Search).size(Size::Display))
///     .subtitle(Text::new("Choose an application").muted())
///     .gap(12);
/// ```
pub struct PanelHeader {
    title: View,
    leading: View,
    subtitle: View,
    trailing: View,
    gap: u32,
    content_gap: u32,
}

impl PanelHeader {
    pub fn new(title: impl Into<View>) -> Self {
        Self {
            title: title.into(),
            leading: View::empty(),
            subtitle: View::empty(),
            trailing: View::empty(),
            gap: 14,
            content_gap: 2,
        }
    }

    /// Use the desktop's title style and an uppercase, muted status caption.
    /// Use `new` and `subtitle` for other content or typography.
    pub fn labelled(title: impl Display, status: impl Display) -> Self {
        Self::new(Text::new(title).size(Size::Title).bold()).subtitle(
            Text::new(status.to_string().to_uppercase())
                .size(Size::Caption)
                .bold()
                .muted(),
        )
    }

    pub fn leading(mut self, view: impl Into<View>) -> Self {
        self.leading = view.into();
        self
    }

    pub fn subtitle(mut self, view: impl Into<View>) -> Self {
        self.subtitle = view.into();
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

impl Component for PanelHeader {
    fn render(&self) -> View {
        ItemRow::new(self.title.clone())
            .leading(self.leading.clone())
            .subtitle(self.subtitle.clone())
            .trailing(self.trailing.clone())
            .gap(self.gap)
            .content_gap(self.content_gap)
            .render()
    }
}
