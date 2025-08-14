use clap::{Args, Parser, Subcommand};

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

#[derive(Subcommand, Debug)]
pub enum WorkspaceSubcommand {
    /// Move the workspace to the n-th position within it's group
    #[command()]
    MoveToPos {
        #[command(flatten)]
        workspace: WorkspaceIdent,
        /// The position to move to
        position: u8,
    },

    #[command()]
    Rename {
        #[command(flatten)]
        workspace: WorkspaceIdent,
        /// the new name for the workspace
        new_name: String,
    },

    /// List all workspaces
    #[command()]
    List {
        /// print capabilities
        #[arg(short, long)]
        capabilities: bool,
    },
}

impl Default for WorkspaceSubcommand {
    fn default() -> Self {
        Self::List {
            capabilities: false,
        }
    }
}

#[derive(Args, Debug)]
pub struct WorkspaceIdent {
    /// the name of the workspace
    pub name: String,
    /// the display of the workspace
    ///
    /// this can be empty if the name is unique
    #[arg(short, long)]
    pub display: Option<String>,
}
