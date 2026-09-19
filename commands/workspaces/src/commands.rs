use omega::{
    Command,
    platform::desktop::{WorkspaceControl, WorkspaceIndex},
};

/// Focus a numbered workspace, creating it if the compositor supports it.
/// Does not require a current reading or optimistically change the highlight.
#[derive(Debug, omega::Command)]
pub struct Select {
    control: WorkspaceControl,
}

impl Command for Select {
    const ID: &'static str = "workspaces.select";

    type Input = WorkspaceIndex;
    type Output = ();

    async fn call(&self, index: WorkspaceIndex) -> omega::Result<()> {
        self.control.switch_to(index).await
    }
}
