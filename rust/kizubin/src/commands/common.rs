use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;

use crate::fsutil;

#[derive(Debug, Args)]
pub(crate) struct CommonArgs {
    /// Specify react native project path
    #[clap(long = "project", value_parser, value_name = "PROJECT_ROOT", value_hint = clap::ValueHint::DirPath)]
    pub(crate) project_root: Option<Utf8PathBuf>,

    /// Force running even if not in react native project
    #[clap(long = "force", value_parser, value_name = "FORCE")]
    pub(crate) force: bool,
}

impl CommonArgs {
    pub(crate) fn new(common: Option<Utf8PathBuf>, force: bool) -> Self {
        Self {
            project_root: common,
            force,
        }
    }

    pub(crate) fn resolve_project_root(&self) -> Utf8PathBuf {
        self.project_root.clone().unwrap_or(".".into())
    }

    pub(crate) fn setup(&self) -> Result<()> {
        let in_react_native_project = self
            .resolve_project_root()
            .join("node_modules")
            .join("@react-native")
            .try_exists()
            .context("failed to check if in react native project")?;

        if !in_react_native_project && !self.force {
            anyhow::bail!(
                "not in react native project. specify one with `--project` or run with `--force` to ignore this"
            );
        }

        if self.force {
            eprintln!("warning: not in react native project, ignoring");
        }

        fsutil::cd(self.resolve_project_root())?;
        Ok(())
    }
}

impl Default for CommonArgs {
    fn default() -> Self {
        Self::new(Some(Utf8PathBuf::from(".")), false)
    }
}
