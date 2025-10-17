use clap::Parser;
use kizubin::{Result, cli};

fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    args.run()
}
