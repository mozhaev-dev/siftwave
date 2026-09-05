use rmcp::{tool, tool_router};

#[derive(Debug, Clone)]
pub struct McpService;

#[tool_router(server_handler)]
impl McpService {
    #[tool(description = "Check whether the server is running")]
    async fn ping(&self) -> String {
        String::from("pong")
    }
}
