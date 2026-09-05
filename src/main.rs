mod mcp_service;

use crate::mcp_service::McpService;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = McpService;
    let running_service = service.serve(stdio()).await?;
    running_service.waiting().await?;
    Ok(())
}
