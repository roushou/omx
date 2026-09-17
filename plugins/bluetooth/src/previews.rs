use super::*;
use omega::testing::{
    State, SystemTopic,
    topic::{BluetoothDevice, BluetoothState},
};
use omega_preview::Cases;

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn devices() -> BluetoothState {
        BluetoothState {
            available: true,
            powered: true,
            devices: vec![
                BluetoothDevice {
                    id: "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_01".into(),
                    address: "AA:BB:CC:DD:EE:01".into(),
                    name: "Headphones".into(),
                    connected: true,
                    paired: true,
                    icon: "audio-headphones".into(),
                    battery_percent: Some(76),
                    can_connect: true,
                },
                BluetoothDevice {
                    id: "/org/bluez/hci1/dev_AA_BB_CC_DD_EE_01".into(),
                    address: "AA:BB:CC:DD:EE:01".into(),
                    name: "Headphones · USB adapter".into(),
                    connected: false,
                    paired: true,
                    icon: "audio-headphones".into(),
                    battery_percent: None,
                    can_connect: true,
                },
                BluetoothDevice {
                    id: "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_02".into(),
                    address: "AA:BB:CC:DD:EE:02".into(),
                    name: "Keyboard".into(),
                    connected: true,
                    paired: true,
                    icon: "input-keyboard".into(),
                    battery_percent: Some(0),
                    can_connect: true,
                },
            ],
            ..Default::default()
        }
    }

    pub(crate) fn state() -> State {
        State::new().with(Self::devices())
    }
}

#[test]
fn preview() {
    Cases::new()
        .surface::<Indicator>("bar", Fixture::state())
        .surface::<Panel>("devices", Fixture::state())
        .surface::<Panel>(
            "off",
            State::new().with(BluetoothState {
                available: true,
                ..Default::default()
            }),
        )
        .surface::<Panel>(
            "empty",
            State::new().with(BluetoothState {
                available: true,
                powered: true,
                ..Default::default()
            }),
        )
        .surface::<Panel>("unavailable", State::new().absent(SystemTopic::Bluetooth))
        .run()
        .unwrap();
}
