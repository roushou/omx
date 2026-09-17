use crate::Panel;
use omega::{
    surface::TextEdit,
    testing::{
        State, SurfaceHarness,
        topic::{Application, ApplicationsState},
    },
};

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn catalogue() -> ApplicationsState {
        ApplicationsState {
            applications: vec![
                Application {
                    id: "org.example.Files.desktop".into(),
                    name: "Files".into(),
                    generic_name: "File Manager".into(),
                    description: "Browse folders and documents".into(),
                    keywords: vec!["folder".into(), "directory".into()],
                    ..Default::default()
                },
                Application {
                    id: "org.example.Terminal.desktop".into(),
                    name: "Terminal".into(),
                    generic_name: "Terminal Emulator".into(),
                    description: "Use a command line".into(),
                    keywords: vec!["shell".into(), "console".into()],
                    ..Default::default()
                },
                Application {
                    id: "org.example.Browser.desktop".into(),
                    name: "Browser".into(),
                    generic_name: "Web Browser".into(),
                    description: "Explore websites".into(),
                    keywords: vec!["internet".into()],
                    ..Default::default()
                },
                Application {
                    id: "org.example.Editor.desktop".into(),
                    name: "Text Editor".into(),
                    generic_name: "Text Editor".into(),
                    description: "Edit plain text and source code".into(),
                    keywords: vec!["notes".into()],
                    ..Default::default()
                },
                Application {
                    id: "org.example.Music.desktop".into(),
                    name: "Music".into(),
                    description: "Play your music collection".into(),
                    keywords: vec!["audio".into()],
                    ..Default::default()
                },
                Application {
                    id: "org.example.Monitor.desktop".into(),
                    name: "System Monitor".into(),
                    description: "Inspect processes".into(),
                    terminal: true,
                    ..Default::default()
                },
                Application {
                    id: "org.example.Photos.desktop".into(),
                    name: "Photos".into(),
                    description: "View images".into(),
                    ..Default::default()
                },
                Application {
                    id: "org.example.Unicode.desktop".into(),
                    name: "Éditeur 日本語".into(),
                    description: "Unicode names".into(),
                    ..Default::default()
                },
            ],
        }
    }

    pub(crate) fn state() -> State {
        State::new().with(Self::catalogue())
    }

    pub(crate) fn edit(panel: &mut SurfaceHarness<Panel>, query: &str) {
        let edit = TextEdit {
            text: query.into(),
            revision: panel.model().query.revision() + 1,
            reset: panel.model().query.reset_revision(),
        };

        let drawn = panel.draw();
        let key = format!("query-{}", panel.model().epoch);
        panel.interact(&drawn, &key, "change", edit).unwrap();
    }
}
