use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use std::cmp::min;
use std::fmt;
use std::process::{Command, ExitStatus, Output, Stdio};

use crate::cliutil;

// Spawn a command
#[derive(Debug, Clone, Default)]
pub(crate) struct Cmd {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: Option<Utf8PathBuf>,
}

#[allow(dead_code)]
impl Cmd {
    pub(crate) fn new<S: Into<String>>(program: S) -> Self {
        Self {
            program: program.into(),
            args: vec![],
            env: vec![],
            cwd: None,
        }
    }

    pub(crate) fn env<K: Into<String>, V: Into<String>>(mut self, k: K, v: V) -> Self {
        self.env.push((k.into(), v.into()));
        self
    }

    /// Add a single argument
    pub(crate) fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments
    pub(crate) fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set the current working directory
    pub(crate) fn cwd<P: Into<Utf8PathBuf>>(mut self, dir: P) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    pub(crate) fn get_cwd(&self) -> String {
        self.cwd.clone().map_or(".".to_string(), |c| c.into())
    }

    pub(crate) fn build_cmd(&self) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (k, v) in &self.env {
            cmd.env(k, v);
        }

        if let Some(ref dir) = self.cwd {
            cmd.current_dir(dir);
        }

        cmd
    }

    pub(crate) fn run(self) -> Result<CmdOutput> {
        let mut cmd = self.build_cmd();
        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("failed to execute `{}`: {}", self, e))?;

        if !output.status.success() {
            anyhow::bail!(
                "command `{}` failed (exit {})\nstdout: {}\nstderr: {}",
                self,
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        }

        Ok(output.into())
    }

    pub(crate) fn run_live<'a, S: Into<&'a str>>(self, name: S) -> Result<CmdOutput> {
        let mut cmd = self.build_cmd();
        let mut progress = cliutil::MultiStep::new(name.into(), 10);
        progress.show();

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().context("failed to take stdout")?;
        let stderr = child.stderr.take().context("failed to take stderr")?;
        progress.register_reader(stdout);
        progress.register_reader(stderr);

        let output = child.wait_with_output()?;
        if !output.status.success() {
            anyhow::bail!(
                "command `{}` failed (exit {})\nout: {}\nenv: {:?}",
                self,
                output.status,
                &progress.output(),
                cmd.get_envs()
            );
        }

        Ok(CmdOutput {
            inner: output,
            output_override: Some(progress.output()),
        })
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = if !self.args.is_empty() {
            " ".to_string() + self.args.join(" ").as_str()
        } else {
            "".to_string()
        };
        write!(f, "{}{}", self.program, args)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct CmdOutput {
    pub(crate) inner: Output,
    output_override: Option<String>,
}

#[allow(dead_code)]
impl CmdOutput {
    pub(crate) fn status(&self) -> ExitStatus {
        self.inner.status
    }

    pub(crate) fn stdout(&self) -> String {
        self.output_override
            .to_owned()
            .unwrap_or_else(|| String::from_utf8_lossy(&self.inner.stdout).into())
    }
    pub(crate) fn stderr(&self) -> String {
        self.output_override
            .to_owned()
            .unwrap_or_else(|| String::from_utf8_lossy(&self.inner.stderr).into())
    }
}

impl From<Output> for CmdOutput {
    fn from(o: Output) -> Self {
        Self {
            inner: o,
            output_override: None,
        }
    }
}

#[allow(dead_code)]
pub(crate) trait Cmds {
    fn run(&self) -> Result<Vec<CmdOutput>>;
    fn run_live(&self, name: Vec<&str>) -> Result<Vec<CmdOutput>>;
    fn cwd<P: Into<Utf8PathBuf>>(self, dir: P) -> Self;
    fn env<K: AsRef<str>, V: AsRef<str>>(self, k: K, v: V) -> Self;
}

impl Cmds for Vec<Cmd> {
    fn run(&self) -> Result<Vec<CmdOutput>> {
        let mut outputs = Vec::with_capacity(self.len());
        for cmd in self {
            outputs.push(cmd.clone().run()?);
        }
        Ok(outputs)
    }

    fn run_live(&self, names: Vec<&str>) -> Result<Vec<CmdOutput>> {
        let mut outputs = Vec::with_capacity(self.len());
        for (i, cmd) in self.iter().enumerate() {
            outputs.push(cmd.clone().run_live(names[min(i, names.len() - 1)])?);
        }
        Ok(outputs)
    }

    fn cwd<P: Into<Utf8PathBuf>>(mut self, dir: P) -> Self {
        let dir = dir.into();
        for c in &mut self {
            c.cwd = Some(dir.clone());
        }
        self
    }

    fn env<K: AsRef<str>, V: AsRef<str>>(self, k: K, v: V) -> Self {
        let k = k.as_ref();
        let v = v.as_ref();
        self.into_iter().map(|cmd| cmd.env(k, v)).collect()
    }
}

#[allow(dead_code)]
pub(crate) trait CmdOutputs {
    fn stdout(&self) -> String;
    fn stderr(&self) -> String;
}

impl CmdOutputs for Vec<CmdOutput> {
    fn stdout(&self) -> String {
        self.iter().map(|o| o.stdout()).collect()
    }

    fn stderr(&self) -> String {
        self.iter().map(|o| o.stderr()).collect()
    }
}
