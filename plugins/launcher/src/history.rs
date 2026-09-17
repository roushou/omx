use omega::platform::applications::ApplicationId;

/// Shared launcher preferences and the 20 most recently admitted launches.
/// Omega retains this record for the lifetime of the daemon.
#[derive(Debug, Clone, Default, omega::PluginState)]
pub(crate) struct History {
    pub favorites: Vec<String>,
    pub recent: Vec<String>,
}

impl History {
    pub(crate) fn favorite(&self, id: &ApplicationId) -> bool {
        self.favorites.iter().any(|value| value == id.as_str())
    }

    pub(crate) fn toggle(&mut self, id: &ApplicationId) {
        if self.favorite(id) {
            self.favorites.retain(|value| value != id.as_str());
        } else {
            self.favorites.push(id.to_string());
        }
    }

    pub(crate) fn launched(&mut self, id: &ApplicationId) {
        self.recent.retain(|value| value != id.as_str());
        self.recent.insert(0, id.to_string());
        self.recent.truncate(20);
    }

    pub(crate) fn rank(&self, id: &ApplicationId) -> (u8, usize) {
        if let Some(index) = self.favorites.iter().position(|value| value == id.as_str()) {
            (0, index)
        } else if let Some(index) = self.recent.iter().position(|value| value == id.as_str()) {
            (1, index)
        } else {
            (2, 0)
        }
    }
}
