use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::building::BuildArgs;

#[derive(Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub(crate) cmd: CliCmd,
}

impl CliArgs {
    pub fn run(&self) -> Result<()> {
        self.cmd.run()
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum CliCmd {
    /// Build for android & iOS
    Build(BuildArgs),
}

impl CliCmd {
    pub(crate) fn run(&self) -> Result<()> {
        match self {
            Self::Build(b) => b.build(),
        }
    }
}
