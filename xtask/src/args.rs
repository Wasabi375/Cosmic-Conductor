use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use clap_complete::{Generator, shells};
use clap_complete_nushell::Nushell;

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
pub struct Arguments {
    // Xtask to run
    #[command(subcommand)]
    pub cmd: Cmd,

    /// out_dir for generated artefacts, defaults to "target/assets"
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Cmd {
    Man,
    Completion { shell: Option<ShellType> },
    // clears just the assets
    Clear,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ShellType {
    Bash,
    Elviszh,
    Fish,
    Zsh,
    Nu,
}

impl Generator for ShellType {
    fn file_name(&self, name: &str) -> String {
        match self {
            ShellType::Bash => shells::Bash.file_name(name),
            ShellType::Elviszh => shells::Elvish.file_name(name),
            ShellType::Fish => shells::Fish.file_name(name),
            ShellType::Zsh => shells::Zsh.file_name(name),
            ShellType::Nu => Nushell.file_name(name),
        }
    }

    fn generate(&self, cmd: &clap::Command, buf: &mut dyn std::io::Write) {
        match self {
            ShellType::Bash => shells::Bash.generate(cmd, buf),
            ShellType::Elviszh => shells::Elvish.generate(cmd, buf),
            ShellType::Fish => shells::Fish.generate(cmd, buf),
            ShellType::Zsh => shells::Zsh.generate(cmd, buf),
            ShellType::Nu => Nushell.generate(cmd, buf),
        }
    }
}
