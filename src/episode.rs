#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStep {
    FindSources,
    RankSources,
    ResearchAndSummarize,
    WriteScript,
    CreateDiscussionContext,
    GenerateAudio,
    Completed,
}

impl WorkflowStep {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FindSources => "find_sources",
            Self::RankSources => "rank_sources",
            Self::ResearchAndSummarize => "research_and_summarize",
            Self::WriteScript => "write_script",
            Self::CreateDiscussionContext => "create_discussion_context",
            Self::GenerateAudio => "generate_audio",
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Episode {
    pub id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub topic_description: String,
    pub current_step: WorkflowStep,
    pub version: i64,
    pub created_at: String,
}
