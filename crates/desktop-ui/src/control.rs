use super::Detail;
use omega::{
    Percent, View,
    ui::{Bind, Column, Component, Size, Slider, Text},
};

/// A label/value row above an arbitrary control or meter.
/// Vertical spacing defaults to 8 pixels; label/value spacing defaults to 12.
/// Values, bindings, keys, and missing-reading behavior belong to the caller.
///
/// ```
/// use desktop_ui::LabelledControl;
/// use omega::{Percent, ui::{Progress, Text}};
/// let level = Percent::whole(40);
/// let meter = LabelledControl::new(
///     Text::new("Used").muted(), Text::new(level), Progress::new(level),
/// ).gap(6);
/// ```
pub struct LabelledControl {
    detail: Detail,
    control: View,
    gap: u32,
}

impl LabelledControl {
    pub fn new(label: impl Into<View>, value: impl Into<View>, control: impl Into<View>) -> Self {
        Self {
            detail: Detail::new(label, value),
            control: control.into(),
            gap: 8,
        }
    }

    pub fn gap(mut self, pixels: u32) -> Self {
        self.gap = pixels;
        self
    }

    pub fn label_gap(mut self, pixels: u32) -> Self {
        self.detail = self.detail.gap(pixels);
        self
    }
}

impl Component for LabelledControl {
    fn render(&self) -> View {
        Column::new()
            .gap(self.gap)
            .child(self.detail.render())
            .child(self.control.clone())
            .into()
    }
}

/// A labelled percentage slider with the desktop's caption styling.
/// The caller owns the typed change binding. The control's local key is `slider`.
/// Use `LabelledControl` for other labels, controls, or spacing.
pub struct LevelControl<'a> {
    pub label: &'a str,
    pub level: Percent,
    pub change: Bind<Percent>,
}

impl Component for LevelControl<'_> {
    fn render(&self) -> View {
        LabelledControl::new(
            Text::new(self.label.to_uppercase())
                .size(Size::Caption)
                .bold()
                .muted(),
            Text::new(self.level).size(Size::Caption).muted(),
            Slider::new(self.level)
                .key("slider")
                .on_change(self.change.clone()),
        )
        .label_gap(0)
        .render()
    }
}
