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
    pub(crate) fn panel(state: &State) -> omega::Result<SurfaceHarness<Panel>> {
        Self::configured(state, &Default::default())
    }

    pub(crate) fn configured(
        state: &State,
        settings: &omega::config::Values,
    ) -> omega::Result<SurfaceHarness<Panel>> {
        let mut panel = SurfaceHarness::configured(state, settings)?;
        panel
            .take_effect()
            .expect("initial subscription")
            .complete(Ok(None))?;
        Self::favorites(&mut panel, &[])?;
        Ok(panel)
    }

    pub(crate) fn favorites(panel: &mut SurfaceHarness<Panel>, ids: &[&str]) -> omega::Result<()> {
        let mut snapshot =
            omega::testing::Stored::<crate::favorites::Favorites>::new(omega::storage::Revision {
                epoch: "a".repeat(32),
                revision: 1,
            })?;
        for id in ids {
            snapshot = snapshot.entry(id.parse().unwrap(), (), 1)?;
        }
        panel.storage(&snapshot)
    }

    pub(crate) fn revision(revision: u64) -> omega::storage::Revision {
        omega::storage::Revision {
            epoch: "a".repeat(32),
            revision,
        }
    }

    pub(crate) fn snapshot(ids: &[&str]) -> omega::testing::Stored<crate::favorites::Favorites> {
        let mut snapshot = omega::testing::Stored::new(Self::revision(5)).unwrap();
        for id in ids {
            snapshot = snapshot.entry(id.parse().unwrap(), (), 4).unwrap();
        }
        snapshot
    }

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
