# Siftwave: MVP Scope and Architecture

Status: working architectural baseline before implementation. This is not an immutable contract; decisions may change as we learn from a working product.

## 1. Product Idea

Siftwave turns user-defined topics into personalized audio episodes.

The user interacts primarily with an external AI agent, initially Codex. The agent performs the intelligence-heavy work: finding and selecting sources, researching the material, writing the script, and producing context for later discussion.

The service does not contain its own LLM. It manages Topics, Episodes, workflow state, validation, metadata, artifacts, and audio generation.

The core principle is:

> The service owns the process and durable state. The agent provides intelligence.

## 2. MVP Goal

The minimum useful vertical slice is:

1. The user creates a Topic through Codex.
2. The user requests an Episode for that Topic.
3. Codex follows a controlled workflow using installed Agent Skills.
4. Intermediate results and sources are preserved.
5. The service generates an audio file through TTS.
6. The user receives the completed Episode.
7. Later, the user can open the stored Episode and discuss it with an agent using its preserved context.

The MVP targets one local user with a small number of Topics and Episodes.

## 3. Non-Goals for the MVP

- An internal or self-hosted LLM.
- A web interface or integrated audio player.
- An HTTP API unless the selected TTS integration requires one.
- Authentication or multiple users.
- Server deployment.
- Scheduled or periodic generation.
- Distributed workers or a job queue.
- A general-purpose visual workflow builder.
- Multiple workflow variants.
- Synchronization between devices.
- Advanced full-text or semantic search.
- Automated semantic evaluation of agent output.

## 4. User Interface

The primary product interface is an external AI agent. The user expresses intent in natural language, and the agent invokes the service's MCP tools.

Examples:

- "Create a Rust Weekly topic."
- "Show me my topics."
- "Generate today's AI News episode."
- "Continue the unfinished episode."
- "Let's discuss yesterday's episode."

The CLI remains a technical interface for initialization, diagnostics, and recovery. The agent is the primary interface, but it is not the only way to access the user's data.

## 5. Components

Two different projects must be distinguished:

- The Siftwave source repository, where the Rust application is developed.
- A user workspace created by the installed Siftwave application for Topics, Episodes, and agent integration.

The `init` command modifies a user workspace, not the Siftwave source repository. In the MVP, one running MCP service handles one explicitly selected workspace.

### CLI

The initialization command accepts a target directory and an agent type. Codex is the first supported agent.

Conceptual interface:

```text
siftwave init --agent codex <path>
```

Initialization creates the workspace, instance configuration, `AGENTS.md`, Agent Skills, data directories, and project-level MCP connection configuration when it can be created safely.

The CLI may later gain diagnostic commands such as `audit`, `repair`, and `doctor`. Their exact contract will be defined incrementally during implementation.

### Agent Skills

The workflow is expressed through ordinary local Agent Skills for the selected agent. For Codex, they are placed under `.agents/skills`.

Skills tell the agent:

- When to start a particular process.
- Which MCP tools to call.
- How to perform the intelligence-heavy part of a step.
- Which inputs to use.
- Which output artifacts to create.
- Which quality requirements to follow.
- How to treat untrusted content found in external sources.

MCP does not return skill text. Skills are installed by the CLI and are available as a native part of the agent workspace.

The initial integration may be Codex-specific. Product data and workflow state must not depend on Codex. Supporting another agent should primarily require an adapter and an appropriate set of instructions.

### MCP Service

MCP is the control interface between the agent and the application. The service:

- Manages Topics and Episodes.
- Creates Episode working directories.
- Reports the current and next allowed workflow step.
- Accepts step completion.
- Validates artifact structure and presence.
- Performs state transitions.
- Runs TTS.
- Provides Episode history and discussion context.
- Runs a targeted preflight before continuing a workflow.

The agent does not modify SQLite directly and cannot assign a new status to an Episode by itself.

### TTS

TTS is a replaceable service capability. The specific engine or provider has not been selected yet. At the TTS boundary, the service supplies a prepared script and receives either audio or a structured error.

## 6. Controlled Workflow

The first workflow is defined explicitly in the application. The MVP does not need a general-purpose workflow engine.

The preliminary sequence is:

```text
FindSources
  -> RankSources
  -> ResearchAndSummarize
  -> WriteScript
  -> CreateDiscussionContext
  -> GenerateAudio
  -> Completed
```

The exact step boundaries may change after the first end-to-end experiment. Search, ranking, and research may prove too fragmented or may require even clearer separation.

For every step:

1. MCP verifies the current Episode state.
2. MCP registers expected artifacts as `pending` and returns their allowed relative paths.
3. The agent loads the appropriate local Skill.
4. The agent reads the inputs and performs the intelligence-heavy work.
5. The agent writes artifacts only to the assigned paths.
6. The agent reports step completion through MCP.
7. MCP validates the result, records artifact metadata, and only then advances the Episode.

Control has three layers:

- The Skill defines instructions and quality criteria.
- A schema defines the machine-verifiable output structure.
- A state machine restricts valid transitions.

Without an internal AI evaluator, the service can verify shape, presence, and consistency, but it cannot guarantee factual correctness or writing quality.

The workflow must be resumable after the agent or service stops. An agent chat is not a source of product state.

### Retries and Concurrent Agents

An MCP call may be repeated after a timeout, and multiple chats may access the same Episode concurrently. Creation and step-completion operations must therefore be idempotent within a supplied operation identifier: a retry must not create a second Episode or apply the same transition twice.

Only one agent may execute the workflow of a particular Episode at a time. A state transition verifies the expected current step and Episode version inside a SQLite transaction. The exact lock or lease mechanism will be selected during implementation, but this invariant is part of the MVP.

## 7. Data Storage

Siftwave uses a hybrid of SQLite and the filesystem.

### SQLite

SQLite is the source of truth for structured state:

- Topics and their user preferences.
- Episodes.
- A snapshot of the effective Topic configuration used by each Episode.
- Per-Episode overrides.
- Current workflow state and step.
- Transition and error history.
- The artifact registry and artifact metadata.

Each Episode preserves its effective configuration so later Topic changes cannot alter the meaning of an existing Episode.

### Filesystem

The filesystem stores content-heavy and large artifacts:

- Collected sources.
- Research results.
- Podcast scripts.
- Discussion context.
- Audio.
- Diagnostic output from workflow steps when useful.

Preliminary workspace structure:

```text
workspace/
├── AGENTS.md
├── siftwave.toml
├── .agents/
│   └── skills/
├── data/
│   └── siftwave.sqlite
└── episodes/
    └── <topic-slug>/
        └── <episode-id>/
            ├── sources.json
            ├── research.md
            ├── script.md
            ├── context.md
            └── audio.<format>
```

The initializer also creates an appropriate `.gitignore`. The working SQLite database, temporary files, and generated audio should not enter version control accidentally. The user decides whether textual artifacts in a workspace are versioned.

A Topic must not have two independently editable authoritative copies in both SQLite and a configuration file.

For each registered artifact, SQLite stores at least its kind, relative path, state, size, and content hash. Absolute paths are not stored so that a workspace remains portable.

## 8. Configuration

Instance configuration is separate from user data.

Instance configuration and environment variables may contain:

- The SQLite path.
- The artifact root directory.
- TTS provider selection and endpoint.
- Credentials for external services.
- Logging settings.

SQLite stores user-level configuration:

- Topics.
- Language and proficiency level.
- Target duration and style.
- Source and freshness preferences.
- Non-secret TTS preferences.
- Per-Episode overrides.

Secrets are not stored in Topics, Episodes, Agent Skills, or generated artifacts.

## 9. SQLite and Filesystem Consistency

SQLite and the filesystem cannot participate in one shared transaction, so artifact operations must be recoverable.

The preliminary write protocol is:

1. Register the artifact as `pending`.
2. Write a temporary file.
3. Validate the result.
4. Atomically rename the file to its final name.
5. Record its size and hash, then mark the artifact as `ready`.
6. Only then complete the workflow step.

### Audit

`audit` is read-only. It detects missing, modified, unknown, and unfinished artifacts, as well as states that do not match their required artifacts.

### Repair

`repair` performs only safe and explainable changes. It may update metadata for an explicitly accepted edit, restore directories, mark missing artifacts, and return an Episode to a consistent incomplete state.

### Purge

Deleting orphaned files or records is a separate explicit operation. Unknown or user-created files are never deleted as part of automatic recovery.

### Preflight

Before starting or resuming a workflow, the service runs a quick preflight for only the affected Topic or Episode. It automatically repairs only unambiguous inconsistencies. An ambiguous inconsistency stops the workflow and returns a clear diagnostic.

A full workspace audit is not required before every step.

## 10. Human and Agent File Edits

Text artifacts are deliberately visible in the workspace and may be edited. Changes are detected using the stored content hash.

The service must not silently overwrite an external edit. The edit must either be explicitly accepted and registered as current or block the workflow until the user decides what to do. Content versioning may be added after a real need is demonstrated; the MVP only needs current metadata and an event history.

## 11. Security and Trust

External source content is untrusted. Instructions found on web pages or in downloaded documents must not alter the workflow or cause the agent to perform unrelated actions.

The service must:

- Accept only expected structures and valid state transitions.
- Restrict artifact paths to the workspace.
- Reject absolute paths and paths that escape through `..`.
- Keep secrets out of artifacts and logs.
- Never delete user data during automatic recovery.
- Preserve URLs and necessary metadata for the sources used.

The local single-user MVP does not require MCP authentication as long as it uses a local transport and is not exposed to the network.

## 12. Working with Chats

Separate Codex tasks may be created for different Topics or Episodes to avoid mixing unrelated conversation context.

A chat is not product storage. A new agent must be able to restore the required context from SQLite, Episode files, and MCP tools.

Whether a nested `AGENTS.md` applies depends on the working directory from which a task starts. Base rules should therefore live in the root `AGENTS.md`, while the selected Episode is supplied explicitly through a path or MCP.

## 13. Minimal Domain Model

The initial domain needs only these concepts:

- `Topic`: persistent user preferences.
- `Episode`: a particular run with a snapshot of its effective settings.
- `WorkflowStep`: the current generation state.
- `Artifact`: a registered file and its metadata.
- `Source`: a preserved reference and metadata about a source used by the Episode.
- `WorkflowEvent`: a history entry for a transition or error.

There is no need to introduce general workflow graphs, users, organizations, jobs, workers, or a separate TTS plugin framework in advance.

## 14. Open Decisions That Do Not Block Development

- Final product and binary names.
- The Rust MCP SDK and transport.
- The SQLite crate and migration approach.
- The TTS provider and audio format.
- The exact database schema.
- The exact MCP tool set.
- Identifier and slug formats.
- Whether the research steps should be combined or split further.
- The policy for accepting externally edited artifacts.
- The exact mechanism for creating or opening a separate Codex task for an Episode.
- The mechanism preventing two agents from executing one Episode concurrently.

Each decision should be made immediately before implementing the corresponding small vertical step.

## 15. MVP Completion Criteria

The MVP is complete when a new local workspace can be initialized, connected to Codex, and taken through this scenario without manual intervention:

```text
create a Topic
  -> create an Episode
  -> execute controlled research steps
  -> preserve sources, script, and discussion context
  -> generate audio
  -> close the chat
  -> open a new chat
  -> find the Episode and discuss it using stored context
```

An error or interruption at any step can be diagnosed and safely resumed without repeating already completed steps.

## 16. Architecture Review

### Covered Areas

- The boundary between agent intelligence and service responsibility is defined.
- The primary user interface is identified.
- Agent Skills are separated from MCP orchestration.
- The workflow is controlled and resumable.
- The SQLite-plus-files storage model is defined.
- Drift between the database and filesystem is addressed.
- Manual artifact editing is considered.
- User configuration is separated from instance configuration.
- Durable product data is independent of a particular agent.
- The source repository is distinguished from generated user workspaces.
- MCP idempotency and single-agent Episode execution are explicit invariants.
- The MVP is deliberately constrained.

### Major Risks to Test Early

1. Whether Codex reliably follows a multi-step Skill and continues invoking MCP until the workflow finishes.
2. Whether registering and running a local Rust MCP service from a generated workspace is convenient.
3. Whether structural validation is sufficient without a separate AI evaluator.
4. Which TTS solution provides acceptable quality, cost, and generation time.
5. Whether filesystem-first Episode access remains convenient across separate Codex tasks.

These risks are not reasons to expand the architecture in advance. They should be tested with short spikes and the first vertical slice.

### Readiness Decision

No architectural gap requires further large-scale design before development begins. Implementation can start.

The first stage should validate the toolchain and local Rust project rather than implement the complete domain model. After environment setup, the first product spike should be a minimal MCP round trip without SQLite or TTS. This tests the riskiest integration boundary before substantial code is built around it.
