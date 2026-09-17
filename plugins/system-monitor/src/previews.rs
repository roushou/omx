use super::*;
use omega::testing::{
    State, SystemTopic,
    topic::{DiskState, Fan, Mount, Sensor, SystemState, ThermalsState},
};
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn system() -> SystemState {
        SystemState {
            cpu_percent: 24,
            core_percent: vec![12, 18, 28, 38],
            memory_total_bytes: 16 * 1024 * 1024 * 1024,
            memory_available_bytes: 10 * 1024 * 1024 * 1024,
            swap_total_bytes: 4 * 1024 * 1024 * 1024,
            swap_free_bytes: 4 * 1024 * 1024 * 1024,
            load_1: 0.72,
            load_5: 0.54,
            load_15: 0.38,
            uptime_seconds: 18342,
        }
    }

    pub(crate) fn disk() -> DiskState {
        DiskState {
            mounts: vec![
                Mount {
                    path: "/".into(),
                    device: "/dev/nvme0n1p2".into(),
                    filesystem: "btrfs".into(),
                    total_bytes: 512 * 1024 * 1024 * 1024,
                    available_bytes: 320 * 1024 * 1024 * 1024,
                },
                Mount {
                    path: "/home".into(),
                    device: "/dev/nvme0n1p2".into(),
                    filesystem: "btrfs".into(),
                    total_bytes: 512 * 1024 * 1024 * 1024,
                    available_bytes: 320 * 1024 * 1024 * 1024,
                },
            ],
        }
    }

    pub(crate) fn thermals() -> ThermalsState {
        ThermalsState {
            sensors: vec![
                Sensor {
                    chip: "coretemp".into(),
                    label: "Package".into(),
                    millicelsius: 48000,
                },
                Sensor {
                    chip: "nvme".into(),
                    label: "Composite".into(),
                    millicelsius: 39000,
                },
            ],
            fans: vec![Fan {
                chip: "thinkpad".into(),
                label: "Fan 1".into(),
                rpm: 2200,
            }],
        }
    }

    pub(crate) fn state() -> State {
        State::new()
            .with(Self::system())
            .with(Self::disk())
            .with(Self::thermals())
    }

    pub(crate) fn missing() -> State {
        State::new()
            .absent(SystemTopic::System)
            .absent(SystemTopic::Disk)
            .absent(SystemTopic::Thermals)
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::state())
        .surface::<Panel>("system", Fixture::state())
        .surface::<Panel>("unavailable", Fixture::missing())
        .surface::<Panel>(
            "no-sensors",
            Fixture::state().with(ThermalsState::default()),
        )
        .run()
        .unwrap();
}
