use omega::{
    Command,
    platform::power::{PowerProfile, PowerProfiles, SetProfile},
};

/// Select one of the profiles currently offered by the system.
#[derive(Debug, omega::Command)]
pub struct ChangeProfile {
    profiles: PowerProfiles,
    control: SetProfile,
}

impl Command for ChangeProfile {
    const ID: &'static str = "power.profile";

    type Input = PowerProfile;
    type Output = ();

    async fn call(&self, profile: PowerProfile) -> omega::Result<()> {
        if !self.profiles.available().contains(&profile) {
            return Err(omega::Error::invalid(
                "Power profile is no longer available",
            ));
        }

        self.control.set(profile).await
    }
}
