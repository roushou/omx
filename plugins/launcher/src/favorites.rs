use omega::{
    platform::applications::ApplicationId,
    storage::{Backend, Limits, Query, Storage, StoragePolicy, Subscription},
};

/// Each entry marks one application as a favorite; values carry no extra data.
#[derive(Debug)]
pub(crate) struct Favorites;

impl Storage for Favorites {
    type Key = ApplicationId;
    type Value = ();

    const ID: &'static str = "omx.launcher.favorites";

    const POLICY: StoragePolicy = StoragePolicy::Persistent {
        backend: Backend::Json,
        schema_version: 1,
    };

    const LIMITS: Limits = Limits {
        max_entries: 32,
        max_value_bytes: 4,
        max_total_bytes: 16 * 1024,
    };
}

#[derive(Debug)]
pub(crate) struct AllFavorites;

impl Subscription for AllFavorites {
    type Storage = Favorites;

    fn query(&self) -> Query<Favorites> {
        Query::new().limit(Favorites::LIMITS.max_entries)
    }
}
