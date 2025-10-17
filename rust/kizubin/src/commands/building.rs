use anyhow::Result;
use clap::{Args, Subcommand};

use super::{builders::AndroidBuildArgs, builders::IosBuildArgs};

#[derive(Args, Debug)]
pub(crate) struct BuildArgs {
    #[clap(subcommand)]
    cmd: BuildCmd,
}

impl BuildArgs {
    pub(crate) fn build(&self) -> Result<()> {
        self.cmd.build()
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum BuildCmd {
    /// Build for android
    Android(AndroidBuildArgs),
    /// Build for iOS
    Ios(IosBuildArgs),
}

impl BuildCmd {
    pub(crate) fn build(&self) -> Result<()> {
        match self {
            Self::Android(a) => a.build(),
            Self::Ios(i) => i.build(),
        }
    }
}
