use crate::Settings;
use omega::platform::desktop::{Workspace, WorkspaceIndex};
use std::collections::BTreeMap;

pub(crate) struct Slot {
    pub(crate) index: WorkspaceIndex,
    pub(crate) windows: u32,
    pub(crate) focused: bool,
    name: String,
    monitor: String,
}

impl Slot {
    fn empty(index: WorkspaceIndex) -> Self {
        Self {
            index,
            windows: 0,
            focused: false,
            name: index.to_string(),
            monitor: String::new(),
        }
    }

    pub(crate) fn tooltip(&self) -> String {
        let mut text = format!("Workspace {}", self.index);

        if self.name != self.index.to_string() {
            text.push_str(&format!(" · {}", self.name));
        }

        if !self.monitor.is_empty() {
            text.push_str(&format!(" · {}", self.monitor));
        }

        text.push_str(&format!(
            " · {} {}",
            self.windows,
            if self.windows == 1 {
                "window"
            } else {
                "windows"
            }
        ));

        if self.focused {
            text.push_str(" · Focused");
        }
        text
    }
}

pub(crate) struct Slots;

impl Slots {
    pub(crate) fn of(workspaces: &[Workspace], settings: &Settings) -> Vec<Slot> {
        let maximum = u32::from(settings.maximum.clamp(1, 30));
        let persistent = u32::from(settings.persistent).min(maximum);
        let mut slots = BTreeMap::<WorkspaceIndex, Slot>::new();

        for index in 1..=persistent {
            let index = WorkspaceIndex::new(index).expect("bounded positive workspace index");
            slots.insert(index, Slot::empty(index));
        }

        for workspace in workspaces {
            if workspace.id() <= 0 || workspace.id() as u32 > maximum {
                continue;
            }

            let index = WorkspaceIndex::new(workspace.id() as u32)
                .expect("bounded positive workspace index");
            slots.insert(
                index,
                Slot {
                    index,
                    windows: workspace.windows(),
                    focused: workspace.is_active(),
                    name: workspace.name().to_string(),
                    monitor: workspace.monitor().to_string(),
                },
            );
        }

        slots.into_values().collect()
    }
}
