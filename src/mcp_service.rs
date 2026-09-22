use std::path::PathBuf;
use tokio_rusqlite::Connection;

use rmcp::{ErrorData, Json, handler::server::wrapper::Parameters, schemars, tool, tool_router};

use crate::storage;

#[derive(Debug, Clone)]
pub struct McpService {
    workspace: PathBuf,
    database: Connection,
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

    #[tool(description = "Create a podcast topic")]
    async fn create_topic(
        &self,
        Parameters(input): Parameters<CreateTopicInput>,
    ) -> Result<Json<CreateTopicOutput>, ErrorData> {
        let CreateTopicInput { name, description } = input;

        let id = storage::create_topic(&self.database, name.clone(), description.clone())
            .await
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;

        Ok(Json(CreateTopicOutput {
            id,
            name,
            description,
        }))
    }
}

impl McpService {
    pub fn new(workspace: PathBuf, connection: Connection) -> Self {
        Self {
            workspace,
            database: connection,
        }
    }
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
struct CreateTopicInput {
    name: String,
    description: String,
}

#[derive(Debug, serde::Serialize, rmcp::schemars::JsonSchema)]
struct CreateTopicOutput {
    id: i64,
    name: String,
    description: String,
}
