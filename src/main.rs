mod cli;
mod mcp_service;

use crate::cli::{Cli, Commands};
use crate::mcp_service::McpService;
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { workspace } => {
            let workspace = workspace.canonicalize()?;

            if !workspace.is_dir() {
                return Result::Err(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("workspace path is not a directory: {}", workspace.display()),
                    )
                    .into(),
                );
            }
            let service = McpService;
            let running_service = service.serve(stdio()).await?;
            running_service.waiting().await?;
        }
    }

    Ok(())
}
