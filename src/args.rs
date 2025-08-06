use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List all windows with their properties.
    #[clap(alias = "t")]
    #[clap(alias = "wi")]
    #[clap(alias = "window")]
    Toplevels,

    /// List all monitors with their properties.
    #[clap(alias = "o")]
    #[clap(alias = "output")]
    Outputs,

    /// List all workspace groups
    #[clap(alias = "wg")]
    WorkspaceGroups,

    /// List all workspaces
    #[clap(alias = "w")]
    Workspaces {
        #[command(subcommand)]
        subcommand: Option<WorkspaceSubcommand>,
    },
}

#[derive(Subcommand, Debug, Default)]
pub enum WorkspaceSubcommand {
    /// Move the workspace to the n-th position within it's group
    #[command()]
    MoveToPos { workspace: String, position: u8 },

    /// List all workspaces
    #[command()]
    #[default]
    List,
}
