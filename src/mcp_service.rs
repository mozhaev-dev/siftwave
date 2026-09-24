use std::path::PathBuf;
use tokio_rusqlite::Connection;

use rmcp::{ErrorData, Json, handler::server::wrapper::Parameters, schemars, tool, tool_router};

use crate::{storage, topic::Topic};

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
    ) -> Result<Json<TopicOutput>, ErrorData> {
        let CreateTopicInput { name, description } = input;

        let topic = storage::create_topic(&self.database, name, description)
            .await
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;

        Ok(Json(topic.into()))
    }

    #[tool(description = "Return a podcast topic by id")]
    async fn get_topic(
        &self,
        Parameters(input): Parameters<GetTopicInput>,
    ) -> Result<Json<TopicOutput>, ErrorData> {
        let topic = storage::get_topic_by_id(&self.database, input.id)
            .await
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;

        Ok(Json(TopicOutput {
            id: topic.id,
            name: topic.name,
            description: topic.description,
        }))
    }

    #[tool(description = "List podcast topics")]
    async fn list_topics(&self) -> Result<Json<Vec<TopicOutput>>, ErrorData> {
        let topics = storage::list_topics(&self.database)
            .await
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;

        let output = topics
            .into_iter()
            .map(TopicOutput::from)
            .collect::<Vec<_>>();

        Ok(Json(output))
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

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
struct GetTopicInput {
    id: i64,
}

#[derive(Debug, serde::Serialize, rmcp::schemars::JsonSchema)]
struct TopicOutput {
    id: i64,
    name: String,
    description: String,
}

impl From<Topic> for TopicOutput {
    fn from(value: Topic) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
        }
    }
}
