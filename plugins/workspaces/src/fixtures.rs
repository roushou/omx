use omega::testing::{State, topic::WorkspacesState};
use omega_proto::omega::WorkspaceInfo;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn at(active: i32) -> State {
        State::new().with(WorkspacesState {
            workspaces: vec![
                Self::workspace(10, "10", "DP-1", 1, active),
                Self::workspace(3, "Code", "DP-1", 2, active),
                Self::workspace(1, "1", "eDP-1", 1, active),
                Self::workspace(-99, "special:scratch", "eDP-1", 1, active),
                Self::workspace(11, "11", "DP-1", 1, active),
            ],
        })
    }

    fn workspace(id: i32, name: &str, monitor: &str, windows: u32, active: i32) -> WorkspaceInfo {
        WorkspaceInfo {
            id,
            name: name.into(),
            monitor_id: monitor.into(),
            windows,
            active: active == id,
        }
    }

    pub(crate) fn empty() -> State {
        State::new().with(WorkspacesState::default())
    }
}
