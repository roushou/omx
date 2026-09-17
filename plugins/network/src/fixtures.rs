use omega::{
    platform::network::WifiPhase,
    testing::{
        State,
        topic::{ThroughputState, VpnState, WifiState},
    },
};

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn wifi() -> WifiState {
        let mut wifi = WifiState {
            phase: WifiPhase::Connected as i32,
            ssid: "Home".into(),
            ..Default::default()
        };

        wifi.access_points.resize_with(3, Default::default);

        for (point, (ssid, strength, secured, active)) in wifi.access_points.iter_mut().zip([
            ("Home", 86, true, true),
            ("Guest/5G~", 55, true, false),
            ("Cafe", 32, false, false),
        ]) {
            point.ssid = ssid.into();
            point.signal_percent = strength;
            point.secured = secured;
            point.active = active;
        }

        wifi
    }

    pub(crate) fn state() -> State {
        let mut traffic = ThroughputState::default();
        traffic.links.push(Default::default());
        let link = &mut traffic.links[0];
        link.interface = "wlan0".into();
        link.rx_bytes_per_sec = 245000;
        link.tx_bytes_per_sec = 28000;
        link.rx_bytes_total = 458000000;
        link.tx_bytes_total = 32000000;
        State::new()
            .with(Self::wifi())
            .network("Home", 86)
            .with(traffic)
            .with(VpnState::default())
    }

    pub(crate) fn vpn() -> State {
        let mut vpn = VpnState::default();
        vpn.tunnels.push(Default::default());
        vpn.tunnels[0].name = "Work".into();
        vpn.tunnels[0].interface = "wg0".into();
        vpn.tunnels[0].kind = "wireguard".into();
        Self::state().with(vpn)
    }
}
