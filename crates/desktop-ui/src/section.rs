use omega::{
    View,
    ui::{Column, Component},
};

/// A group of arbitrary views with an optional heading and a 14-pixel default gap.
/// Adds no separator or fixed width. Set those on the surrounding composition.
///
/// ```
/// use desktop_ui::Section;
/// use omega::ui::{Header, Text};
/// let section = Section::new().heading(Header::new("Devices"))
///     .gap(8).children([Text::new("Keyboard"), Text::new("Mouse")]);
/// ```
pub struct Section {
    heading: View,
    children: Vec<View>,
    gap: u32,
}

impl Section {
    pub fn new() -> Self {
        Self {
            heading: View::empty(),
            children: Vec::new(),
            gap: 14,
        }
    }

    pub fn heading(mut self, view: impl Into<View>) -> Self {
        self.heading = view.into();
        self
    }

    pub fn gap(mut self, pixels: u32) -> Self {
        self.gap = pixels;
        self
    }

    pub fn child(mut self, view: impl Into<View>) -> Self {
        self.children.push(view.into());
        self
    }

    pub fn children<V: Into<View>>(mut self, views: impl IntoIterator<Item = V>) -> Self {
        self.children.extend(views.into_iter().map(Into::into));
        self
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Section {
    fn render(&self) -> View {
        Column::new()
            .gap(self.gap)
            .child(self.heading.clone())
            .children(self.children.iter().cloned())
            .into()
    }
}
