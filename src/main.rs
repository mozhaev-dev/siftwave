mod cli;
mod mcp_service;

use crate::cli::{Cli, Commands};
use crate::mcp_service::McpService;
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            fs::create_dir_all(&path)?;
            fs::create_dir_all(path.join("data"))?;
            fs::create_dir_all(path.join("episodes"))?;
        }
        Commands::Serve { workspace } => {
            let workspace = workspace.canonicalize()?;

            if !workspace.is_dir() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("workspace path is not a directory: {}", workspace.display()),
                )
                .into());
            }
            let service = McpService::new(workspace);
            let running_service = service.serve(stdio()).await?;
            running_service.waiting().await?;
        }
    }

    Ok(())
}
