use anyhow::Result;
use clap::Args;

use crate::commands::common::CommonArgs;

#[derive(Args, Debug)]
pub(crate) struct IosBuildArgs {
    #[clap(flatten)]
    config: CommonArgs,
}

impl IosBuildArgs {
    pub(crate) fn build(&self) -> Result<()> {
        self.config.setup()?;
        eprintln!("not implemented yet");
        Ok(())
    }
}
