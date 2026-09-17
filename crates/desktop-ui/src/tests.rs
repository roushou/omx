use super::*;
use omega::{
    Percent, Surface, View,
    surface::{Events, Task},
    testing::{Drawn, State, SurfaceHarness},
    ui::{Button, Column, Component, Slider, Text},
};

#[derive(omega::Surface)]
struct Controls;

enum Message {
    Add,
    Set(Percent),
}

impl Surface for Controls {
    type Model = u32;
    type Message = Message;
    type Effects = ();

    fn update(&self, model: &mut u32, message: Message, _: &()) -> Task<Message> {
        match message {
            Message::Add => *model += 1,
            Message::Set(value) => *model = u32::from(value.whole_percent()),
        }
        Task::none()
    }

    fn render(&self, model: &u32, events: &Events<Message>) -> View {
        Section::new()
            .heading(Text::new("Custom heading"))
            .gap(10)
            .children([
                PanelHeader::new(Column::new().child(Text::new("Custom title")))
                    .leading(
                        Button::new("Leading")
                            .key("action")
                            .on_press(events.on(|()| Message::Add)),
                    )
                    .subtitle(Text::new("Mixed case subtitle"))
                    .trailing(
                        Button::new("Trailing")
                            .key("trailing")
                            .on_press(events.on(|()| Message::Add)),
                    )
                    .gap(6)
                    .content_gap(3)
                    .key("header"),
                ItemRow::new(Text::new("Device"))
                    .leading(
                        Button::new("Inspect")
                            .key("inspect")
                            .on_press(events.on(|()| Message::Add)),
                    )
                    .subtitle(Text::new("Available"))
                    .trailing(
                        Button::new("Connect")
                            .key("connect")
                            .on_press(events.on(|()| Message::Add)),
                    )
                    .gap(9)
                    .content_gap(4)
                    .key("device"),
                Detail::new(
                    Text::new("Action"),
                    Button::new("Run")
                        .key("action")
                        .on_press(events.on(|()| Message::Add)),
                )
                .gap(4)
                .key("detail"),
                LabelledControl::new(
                    Text::new("Level"),
                    Text::new(model),
                    Slider::new(Percent::whole(*model as u8))
                        .key("control")
                        .on_change(events.on(Message::Set)),
                )
                .gap(5)
                .label_gap(3)
                .key("meter"),
            ])
            .render()
    }
}

#[test]
fn custom_slots_preserve_binding_identity_across_composition() {
    let mut surface = SurfaceHarness::<Controls>::new(&State::new()).unwrap();

    assert!(surface.draw().text().contains("Mixed case subtitle"));

    for key in [
        "header/action",
        "header/trailing",
        "detail/action",
        "device/inspect",
        "device/connect",
    ] {
        let drawn = surface.draw();
        surface.interact(&drawn, key, "press", ()).unwrap();
    }
    assert_eq!(*surface.model(), 5);
    let drawn = surface.draw();
    surface
        .interact(&drawn, "meter/control", "change", Percent::whole(42))
        .unwrap();

    assert_eq!(*surface.model(), 42);
}

#[test]
fn optional_slots_and_headings_can_be_omitted() {
    let heading = Drawn::of_view(PanelHeader::new(Text::new("Title")));

    assert_eq!(heading.text(), "Title");
    assert!(heading.first("icon").is_none());
    let section = Drawn::of_view(Section::new().child(Text::new("Body")));

    assert_eq!(section.text(), "Body");
    assert!(section.first("separator").is_none());
    assert_eq!(
        Drawn::of_view(PanelHeader::labelled("Title", "ready")).text(),
        "Title READY"
    );
}

#[test]
fn item_row_supports_a_title_without_optional_slots() {
    let row = Drawn::of_view(ItemRow::new(Text::new("Only a title")));
    assert_eq!(row.text(), "Only a title");
    assert!(row.first("button").is_none());
    assert!(row.first("icon").is_none());
}
