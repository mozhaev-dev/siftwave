use std::path::PathBuf;

use rmcp::{tool, tool_router};

#[derive(Debug, Clone)]
pub struct McpService {
    workspace: PathBuf,
}

#[tool_router(server_handler)]
impl McpService {
    #[tool(description = "Check whether the server is running")]
    async fn ping(&self) -> String {
        String::from("pong")
    }

    #[tool(description = "Return active workspace path")]
    async fn workspace_path(&self) -> String {
        self.workspace.display().to_string()
    }
}

impl McpService {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}
