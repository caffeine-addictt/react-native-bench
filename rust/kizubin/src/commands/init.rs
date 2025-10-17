use anyhow::Result;
use clap::{Args, Subcommand};

use super::{builders::AndroidBuildArgs, builders::IosBuildArgs};

#[derive(Args, Debug)]
pub(crate) struct InitArgs {
    #[clap(subcommand)]
    cmd: InitCmd,
}

impl InitArgs {
    pub(crate) fn run(&self) -> Result<()> {
        self.cmd.run()
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum InitCmd {
    /// Build for android
    Android(AndroidBuildArgs),
    /// Build for iOS
    Ios(IosBuildArgs),
}

impl InitCmd {
    pub(crate) fn run(&self) -> Result<()> {
        match self {
            Self::Android(a) => a.build(),
            Self::Ios(i) => i.build(),
        }
    }
}
