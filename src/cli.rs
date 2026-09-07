use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "siftwave", version, about = "AI-driven podcast generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Serve {
        #[arg(long, value_name = "PATH")]
        workspace: PathBuf,
    },
    Init {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
}
