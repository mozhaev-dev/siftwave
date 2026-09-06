use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "siftwave", version, about = "AI-driven podcast generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Serve,
}
