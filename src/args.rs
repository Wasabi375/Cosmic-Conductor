use std::fmt::Display;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, default_value_t)]
    pub format: OutputFormat,
}

#[derive(ValueEnum, Debug, Default, Clone, Copy)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
    JsonPretty,
}
impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OutputFormat::Human => "human",
            OutputFormat::Json => "json",
            OutputFormat::JsonPretty => "json-pretty",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List all windows with their properties.
    #[clap(alias = "t")]
    #[clap(alias = "wi")]
    #[clap(alias = "window")]
    Toplevels {
        #[command(subcommand)]
        subcommand: Option<ToplevelSubcommand>,
    },

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
pub enum ToplevelSubcommand {
    /// List all toplevels
    #[clap(alias = "l")]
    List {
        /// limit toplevels to workspace
        ///
        /// must be used with display if the name of the workspace is not unique
        #[arg(short, long)]
        workspace: Option<String>,

        /// limit toplevels to display
        #[arg(short, long)]
        display: Option<String>,

        /// show the geometry of each toplevel
        #[arg(short, long)]
        geometry: bool,
    },
    /// maximize the toplevel
    Max {
        /// the unique id of the toplevel
        ///
        /// It is enough to provide the first characters as long as they
        /// are unique.
        id: String,

        /// undo maximize toplevel instead
        ///
        /// can't be used with toggle
        #[arg(short, long)]
        unset: bool,

        /// toggle toplevel maximize state
        ///
        /// can't be used with unset
        #[arg(short, long)]
        toggle: bool,
    },
    /// minimize the toplevel
    Min {
        /// the unique id of the toplevel
        ///
        /// It is enough to provide the first characters as long as they
        /// are unique.
        id: String,

        /// undo minimize
        ///
        /// can't be used with toggle
        #[arg(short, long)]
        unset: bool,

        /// toggle toplevel minimiz state
        ///
        /// can't be used with unset
        #[arg(short, long)]
        toggle: bool,
    },
    /// fullscreen the toplevel
    #[clap(alias = "full")]
    Fullscreen {
        /// the unique id of the toplevel
        ///
        /// It is enough to provide the first characters as long as they
        /// are unique.
        id: String,

        /// fullscreen toplevel instead
        ///
        /// can't be used with toggle
        #[arg(short, long)]
        minimize: bool,

        /// toggle toplevel fullscreen state
        ///
        /// can't be used with unset
        #[arg(short, long)]
        toggle: bool,
    },
    /// mark the toplevel as sticky
    Sticky {
        /// the unique id of the toplevel
        ///
        /// It is enough to provide the first characters as long as they
        /// are unique.
        id: String,

        /// unset sticky for toplevel instead
        ///
        /// can't be used with toggle
        #[arg(short, long)]
        minimize: bool,

        /// toggle toplevel sticky state
        ///
        /// can't be used with unset
        #[arg(short, long)]
        toggle: bool,
    },
}

impl Default for ToplevelSubcommand {
    fn default() -> Self {
        ToplevelSubcommand::List {
            display: None,
            workspace: None,
            geometry: false,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceSubcommand {
    /// Move the workspace to the n-th position within it's group
    #[clap(alias = "mp")]
    MoveToPos {
        #[command(flatten)]
        workspace: WorkspaceIdent,
        /// The position to move to
        position: u8,
    },

    /// Move the workspace to the specified display
    #[clap(alias = "md")]
    MoveToDisplay {
        #[command(flatten)]
        workspace: WorkspaceIdent,

        /// the target display
        target_display: String,
        /// position on the display
        ///
        /// Moved to the last position if left empty
        position: Option<u8>,
    },
    #[clap(alias = "p")]
    Pin {
        #[command(flatten)]
        workspace: WorkspaceIdent,
    },
    #[clap(alias = "u")]
    Unpin {
        #[command(flatten)]
        workspace: WorkspaceIdent,
    },
    #[clap(alias = "a")]
    Activate {
        #[command(flatten)]
        workspace: WorkspaceIdent,
    },

    /// List all workspaces
    #[clap(alias = "l")]
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
