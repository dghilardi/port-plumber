use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// Cli interface to port-plumber
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct PluCtlArgs {
    #[arg(short, long, default_value = "/run/port-plumber/cmd.sock")]
    pub path: PathBuf,
    #[clap(subcommand)]
    pub subcommand: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List current mappings
    List,
    Resolve { name: String }
}