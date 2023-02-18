use std::path::PathBuf;
use clap::Parser;

/// Allows to bind ports and execute initialization commands
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct PortPlumberArgs {
    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}