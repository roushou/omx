use omega::{
    Command,
    platform::bluetooth::{Bluetooth, BluetoothControl, BluetoothDevice, DeviceId},
};

/// Connect a known, connectable endpoint. Already-connected devices cause no action.
#[derive(Debug, omega::Command)]
pub struct Connect {
    bluetooth: Bluetooth,
    control: BluetoothControl,
}

impl Command for Connect {
    const ID: &'static str = "bluetooth.connect";

    type Input = DeviceId;
    type Output = ();

    const DESCRIPTION: &'static str = "Connect a known Bluetooth device";

    async fn call(&self, id: DeviceId) -> omega::Result<()> {
        let device = Devices::find(&self.bluetooth, &id)?;

        if device.is_connected() {
            return Ok(());
        }

        if !device.can_connect() {
            return Err(omega::Error::invalid(
                "This device cannot connect while its adapter is off or the device is unavailable",
            ));
        }

        self.control.connect(&id).await
    }
}

/// Disconnect one known endpoint. Already-disconnected devices cause no action.
#[derive(Debug, omega::Command)]
pub struct Disconnect {
    bluetooth: Bluetooth,
    control: BluetoothControl,
}

impl Command for Disconnect {
    const ID: &'static str = "bluetooth.disconnect";

    type Input = DeviceId;
    type Output = ();

    const DESCRIPTION: &'static str = "Disconnect a Bluetooth device";

    async fn call(&self, id: DeviceId) -> omega::Result<()> {
        if !Devices::find(&self.bluetooth, &id)?.is_connected() {
            return Ok(());
        }

        self.control.disconnect(&id).await
    }
}

struct Devices;

impl Devices {
    fn find(bluetooth: &Bluetooth, id: &DeviceId) -> omega::Result<BluetoothDevice> {
        bluetooth
            .known_devices()
            .into_iter()
            .find(|device| device.id() == id)
            .ok_or_else(|| omega::Error::invalid("This Bluetooth device is no longer available"))
    }
}
