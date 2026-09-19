use crate::{Snapshot, source};
use omega::{Command, record::Own};

/// Admit an immediate collection. Repeated requests join the active collection.
/// Completion acknowledges admission; the shared snapshot reports progress and errors.
#[derive(Debug, omega::Command)]
pub struct Refresh {
    snapshot: Own<Snapshot>,
}

impl Command for Refresh {
    const ID: &'static str = "ai-usage.refresh";

    type Input = ();
    type Output = ();

    async fn call(&self, _: ()) -> omega::Result<()> {
        self.snapshot
            .set(&source::Source::shared().poll(true))
            .await
    }
}

/// Publish worker progress and refresh automatically when due. Schedule every 5 seconds.
/// Collection runs every 15 minutes, or after 1 minute following a collection failure.
#[derive(Debug, omega::Command)]
pub struct Poll {
    snapshot: Own<Snapshot>,
}

impl Command for Poll {
    const ID: &'static str = "ai-usage.poll";

    type Input = ();
    type Output = ();

    async fn call(&self, _: ()) -> omega::Result<()> {
        let next = source::Source::shared().poll(false);

        if self.snapshot.get() != next {
            self.snapshot.set(&next).await?;
        }
        Ok(())
    }
}
