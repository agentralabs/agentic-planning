use agentic_planning::{
    validators, AuditAction, AuditEntityType, CommitmentId, CreateCommitmentRequest,
    CreateDecisionRequest, CreateGoalRequest, DecisionId, DecisionPath, DecisionReasoning, DreamId,
    EntanglementType, FederationId, GoalFilter, GoalId, GoalRelationship, GoalStatus, PathId,
    PlanningEngine, Priority, Promise, ReincarnationUpdates, ScopeChange, Stakeholder,
    StakeholderId, Timestamp,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

use crate::protocol::{self, JsonRpcError, JsonRpcResponse};

// Runtime hardening guardrail constants.
pub const MAX_CONTENT_LENGTH_BYTES: usize = 8 * 1024 * 1024;

pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "agentic-planning";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Error)]
pub enum McpError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("missing operation")]
    MissingOperation,
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("unknown operation: {0}")]
    UnknownOperation(String),
    #[error("invalid value for {field}: {reason}")]
    InvalidValue { field: &'static str, reason: String },
    #[error("core error: {0}")]
    Core(#[from] agentic_planning::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: String,
    pub ty: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub params: Vec<ToolParam>,
}

impl Tool {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            params: Vec::new(),
        }
    }

    pub fn description(mut self, value: &str) -> Self {
        self.description = value.to_string();
        self
    }

    pub fn param(mut self, name: &str, ty: &str, description: &str, required: bool) -> Self {
        self.params.push(ToolParam {
            name: name.to_string(),
            ty: ty.to_string(),
            description: description.to_string(),
            required,
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub description: String,
    pub mime_type: String,
}

impl Resource {
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
            description: String::new(),
            mime_type: "application/json".to_string(),
        }
    }

    pub fn description(mut self, value: &str) -> Self {
        self.description = value.to_string();
        self
    }

    pub fn mime_type(mut self, value: &str) -> Self {
        self.mime_type = value.to_string();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArg {
    pub name: String,
    pub ty: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub args: Vec<PromptArg>,
}

impl Prompt {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            args: Vec::new(),
        }
    }

    pub fn description(mut self, value: &str) -> Self {
        self.description = value.to_string();
        self
    }

    pub fn arg(mut self, name: &str, ty: &str, description: &str) -> Self {
        self.args.push(PromptArg {
            name: name.to_string(),
            ty: ty.to_string(),
            description: description.to_string(),
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContextLogEntry {
    index: usize,
    intent: String,
    finding: Option<String>,
    topic: Option<String>,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusRound {
    round: u32,
    stakeholder: String,
    position: String,
    recorded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusSession {
    decision_id: DecisionId,
    stakeholders: Vec<String>,
    rounds: Vec<ConsensusRound>,
    votes: HashMap<String, String>,
    status: String,
}

pub struct PlanningMcpServer {
    engine: PlanningEngine,
    initialized: bool,
    session_id: Option<String>,
    workspaces: HashMap<String, String>,
    active_workspace: Option<String>,
    consensus_sessions: HashMap<DecisionId, ConsensusSession>,
    context_log: Vec<ContextLogEntry>,
    auth: agentic_planning::auth::TokenAuth,
}

fn mcp_tool_surface_is_compact() -> bool {
    std::env::var("APLAN_MCP_TOOL_SURFACE")
        .or_else(|_| std::env::var("MCP_TOOL_SURFACE"))
        .map(|v| v.eq_ignore_ascii_case("compact"))
        .unwrap_or(false)
}

impl PlanningMcpServer {
    pub fn new(engine: PlanningEngine) -> Self {
        Self {
            engine,
            initialized: false,
            session_id: None,
            workspaces: HashMap::new(),
            active_workspace: None,
            consensus_sessions: HashMap::new(),
            context_log: Vec::new(),
            // Auth token read from AGENTIC_AUTH_TOKEN env var via TokenAuth::from_env()
            auth: agentic_planning::auth::TokenAuth::from_env(),
        }
    }

    /// Persist the engine state to disk (no-op for in-memory engines or clean state).
    pub fn save(&mut self) -> agentic_planning::Result<()> {
        self.engine.save()
    }

    /// Borrow the underlying engine (for ghost bridge context building).
    pub fn engine(&self) -> &PlanningEngine {
        &self.engine
    }

    /// Handle a raw JSON-RPC message string from stdio transport.
    ///
    /// Returns the serialised JSON-RPC response, or empty string for notifications.
    pub fn handle_raw(&mut self, raw: &str) -> String {
        let response = match protocol::parse_request(raw) {
            Ok(request) => {
                if request.id.is_none() {
                    self.handle_notification(&request.method, &request.params);
                    return String::new();
                }
                self.handle_request(request)
            }
            Err(error_response) => error_response,
        };
        serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Serialization failed"}}"#
                .to_string()
        })
    }

    /// Handle a parsed JSON-RPC request (has an id).
    fn handle_request(&mut self, request: protocol::JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone().unwrap_or(Value::Null);
        match request.method.as_str() {
            "initialize" => self.handle_initialize(id),
            "shutdown" => self.handle_shutdown(id),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, &request.params),
            "resources/list" => self.handle_resources_list(id),
            "resources/read" => self.handle_resources_read(id, &request.params),
            "prompts/list" => self.handle_prompts_list(id),
            "prompts/get" => self.handle_prompts_get(id, &request.params),
            _ => JsonRpcResponse::error(id, JsonRpcError::method_not_found(&request.method)),
        }
    }

    /// Handle JSON-RPC notifications (messages without an `id`).
    fn handle_notification(&mut self, method: &str, _params: &Value) {
        if method == "notifications/initialized" {
            self.initialized = true;
            eprintln!(
                "agentic-planning: initialized (session {:?})",
                self.session_id
            );
        }
    }

    fn handle_initialize(&mut self, id: Value) -> JsonRpcResponse {
        self.initialized = true;
        self.session_id = Some(Uuid::new_v4().to_string());
        JsonRpcResponse::success(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false, "listChanged": false },
                    "prompts": { "listChanged": false }
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION
                }
            }),
        )
    }

    fn handle_shutdown(&mut self, id: Value) -> JsonRpcResponse {
        eprintln!("agentic-planning: shutdown (session {:?})", self.session_id);
        self.initialized = false;
        self.session_id = None;
        JsonRpcResponse::success(id, json!(null))
    }

    /// Returns true after initialize handshake completes.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn handle_tools_list(&self, id: Value) -> JsonRpcResponse {
        let tool_defs: Vec<Value> = self
            .tools()
            .iter()
            .map(|tool| {
                let mut properties = serde_json::Map::new();
                let mut required = Vec::new();
                for p in &tool.params {
                    properties.insert(
                        p.name.clone(),
                        json!({
                            "type": p.ty,
                            "description": p.description
                        }),
                    );
                    if p.required {
                        required.push(Value::String(p.name.clone()));
                    }
                }
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": {
                        "type": "object",
                        "properties": properties,
                        "required": required
                    }
                })
            })
            .collect();
        JsonRpcResponse::success(id, json!({ "tools": tool_defs }))
    }

    fn handle_tools_call(&mut self, id: Value, params: &Value) -> JsonRpcResponse {
        // Auth gate: validate token before dispatching any tool call.
        let provided_token = params
            .get("_meta")
            .and_then(|m| m.get("auth_token"))
            .and_then(Value::as_str);

        if let Err(auth_err) = self.auth.validate(provided_token) {
            return JsonRpcResponse::error(
                id,
                JsonRpcError {
                    code: -32001,
                    message: format!("authentication failed: {auth_err}"),
                    data: None,
                },
            );
        }

        let name = match params.get("name").and_then(Value::as_str) {
            Some(n) => n.to_string(),
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("missing 'name' in tools/call"),
                );
            }
        };

        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        // Use a minimal runtime to call the async handle_tool.
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map(|rt| rt.block_on(self.handle_tool(&name, arguments)));

        // Determine entity type from tool name prefix for audit logging.
        let entity_type = if name.starts_with("goal") {
            AuditEntityType::Goal
        } else if name.starts_with("decision") || name.starts_with("consensus") {
            AuditEntityType::Decision
        } else if name.starts_with("commitment") {
            AuditEntityType::Commitment
        } else if name.starts_with("dream") {
            AuditEntityType::Dream
        } else if name.starts_with("federation") {
            AuditEntityType::Federation
        } else {
            AuditEntityType::Goal // default bucket for utility tools
        };

        // Infer audit action from tool name.
        let action = if name.contains("create") || name.contains("add") {
            AuditAction::Create
        } else if name.contains("get") || name.contains("list") || name.contains("query") {
            AuditAction::Read
        } else if name.contains("update") || name.contains("progress") || name.contains("resolve") {
            AuditAction::Update
        } else if name.contains("delete") || name.contains("cancel") {
            AuditAction::Delete
        } else if name.contains("crystallize") {
            AuditAction::Crystallize
        } else if name.contains("fulfill") {
            AuditAction::Fulfill
        } else if name.contains("break") {
            AuditAction::Break
        } else if name.contains("status") {
            AuditAction::StatusChange
        } else {
            AuditAction::Read
        };

        // Capture session_id before mutable borrow for audit logging.
        let sid = self.engine.session_id();

        // Build response and record audit entry.
        match result {
            Ok(Ok(value)) => {
                self.engine.audit_log_mut().record(
                    sid,
                    &name,
                    entity_type,
                    &name,
                    action,
                    true,
                    None,
                    None,
                );
                JsonRpcResponse::success(
                    id,
                    json!({
                        "content": [{"type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default()}]
                    }),
                )
            }
            Ok(Err(McpError::UnknownTool(t))) => {
                self.engine.audit_log_mut().record(
                    sid,
                    &name,
                    entity_type,
                    &name,
                    action,
                    false,
                    None,
                    Some(format!("unknown tool: {t}")),
                );
                JsonRpcResponse::error(id, JsonRpcError::tool_not_found(t))
            }
            Ok(Err(e)) => {
                self.engine.audit_log_mut().record(
                    sid,
                    &name,
                    entity_type,
                    &name,
                    action,
                    false,
                    None,
                    Some(e.to_string()),
                );
                JsonRpcResponse::tool_error(id, e.to_string())
            }
            Err(e) => {
                let msg = format!("runtime error: {e}");
                self.engine.audit_log_mut().record(
                    sid,
                    &name,
                    entity_type,
                    &name,
                    action,
                    false,
                    None,
                    Some(msg.clone()),
                );
                JsonRpcResponse::tool_error(id, msg)
            }
        }
    }

    fn handle_resources_list(&self, id: Value) -> JsonRpcResponse {
        let resource_defs: Vec<Value> = self
            .resources()
            .iter()
            .map(|r| {
                json!({
                    "uri": r.uri,
                    "name": r.uri,
                    "description": r.description,
                    "mimeType": r.mime_type
                })
            })
            .collect();
        JsonRpcResponse::success(id, json!({ "resources": resource_defs }))
    }

    fn handle_resources_read(&self, id: Value, params: &Value) -> JsonRpcResponse {
        let uri = match params.get("uri").and_then(Value::as_str) {
            Some(u) => u,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("missing 'uri' in resources/read"),
                );
            }
        };

        let content = match uri {
            "planning://goals" => {
                let goals: Vec<_> = self
                    .engine
                    .list_goals(GoalFilter::default())
                    .into_iter()
                    .cloned()
                    .collect();
                serde_json::to_string_pretty(&goals).unwrap_or_default()
            }
            "planning://decisions" => {
                let decisions: Vec<_> = self.engine.list_decisions().into_iter().cloned().collect();
                serde_json::to_string_pretty(&decisions).unwrap_or_default()
            }
            "planning://commitments" => {
                let commitments: Vec<_> = self
                    .engine
                    .list_commitments()
                    .into_iter()
                    .cloned()
                    .collect();
                serde_json::to_string_pretty(&commitments).unwrap_or_default()
            }
            "planning://singularity" => {
                let singularity = self.engine.get_intention_singularity();
                serde_json::to_string_pretty(&singularity).unwrap_or_default()
            }
            "planning://status" => json!({
                "goals": self.engine.goal_count(),
                "decisions": self.engine.decision_count(),
                "commitments": self.engine.commitment_count(),
                "session_id": self.session_id,
                "initialized": self.initialized
            })
            .to_string(),
            _ if uri.starts_with("planning://goals/") => {
                let raw_id = &uri["planning://goals/".len()..];
                match parse_goal_id(raw_id) {
                    Ok(goal_id) => match self.engine.get_goal(goal_id) {
                        Some(goal) => serde_json::to_string_pretty(goal).unwrap_or_default(),
                        None => {
                            return JsonRpcResponse::error(
                                id,
                                JsonRpcError::invalid_params(format!("goal not found: {raw_id}")),
                            );
                        }
                    },
                    Err(e) => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params(format!("invalid goal id: {e}")),
                        );
                    }
                }
            }
            _ if uri.starts_with("planning://dreams/") => {
                let raw_id = &uri["planning://dreams/".len()..];
                match Uuid::parse_str(raw_id) {
                    Ok(uuid) => {
                        let dream_id = DreamId(uuid);
                        match self.engine.get_dream(dream_id) {
                            Some(dream) => serde_json::to_string_pretty(dream).unwrap_or_default(),
                            None => {
                                return JsonRpcResponse::error(
                                    id,
                                    JsonRpcError::invalid_params(format!(
                                        "dream not found: {raw_id}"
                                    )),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params(format!("invalid dream id: {e}")),
                        );
                    }
                }
            }
            _ if uri.starts_with("planning://consensus/") => {
                let raw_id = &uri["planning://consensus/".len()..];
                match Uuid::parse_str(raw_id) {
                    Ok(uuid) => {
                        let decision_id = DecisionId(uuid);
                        match self.consensus_sessions.get(&decision_id) {
                            Some(session) => {
                                serde_json::to_string_pretty(session).unwrap_or_default()
                            }
                            None => {
                                return JsonRpcResponse::error(
                                    id,
                                    JsonRpcError::invalid_params(format!(
                                        "consensus session not found: {raw_id}"
                                    )),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params(format!("invalid consensus id: {e}")),
                        );
                    }
                }
            }
            _ if uri.starts_with("planning://workspace/") => {
                let name = &uri["planning://workspace/".len()..];
                match self.workspaces.get(name) {
                    Some(path) => json!({
                        "name": name,
                        "path": path,
                        "active": self.active_workspace.as_deref() == Some(name)
                    })
                    .to_string(),
                    None => {
                        return JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_params(format!("workspace not found: {name}")),
                        );
                    }
                }
            }
            _ => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(format!("unknown resource: {uri}")),
                );
            }
        };

        JsonRpcResponse::success(
            id,
            json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": content
                }]
            }),
        )
    }

    fn handle_prompts_list(&self, id: Value) -> JsonRpcResponse {
        let prompt_defs: Vec<Value> = self
            .prompts()
            .iter()
            .map(|p| {
                let args: Vec<Value> = p
                    .args
                    .iter()
                    .map(|a| {
                        json!({
                            "name": a.name,
                            "description": a.description,
                            "required": false
                        })
                    })
                    .collect();
                json!({
                    "name": p.name,
                    "description": p.description,
                    "arguments": args
                })
            })
            .collect();
        JsonRpcResponse::success(id, json!({ "prompts": prompt_defs }))
    }

    fn handle_prompts_get(&self, id: Value, params: &Value) -> JsonRpcResponse {
        let name = match params.get("name").and_then(Value::as_str) {
            Some(n) => n,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("missing 'name' in prompts/get"),
                );
            }
        };

        let prompt = self.prompts().into_iter().find(|p| p.name == name);
        let prompt = match prompt {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(format!("unknown prompt: {name}")),
                );
            }
        };

        let messages = match name {
            "planning_review" => {
                let period = params
                    .get("arguments")
                    .and_then(|a| a.get("period"))
                    .and_then(Value::as_str)
                    .unwrap_or("daily");
                let goals = self.engine.goal_count();
                let decisions = self.engine.decision_count();
                let commitments = self.engine.commitment_count();
                json!([{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Generate a {period} planning review.\n\n\
                            Current state: {goals} goals, {decisions} decisions, {commitments} commitments.\n\n\
                            Use planning_goal list, planning_commitment inventory, and planning_singularity collapse \
                            to gather data, then provide:\n\
                            1. Progress summary\n\
                            2. Blocked items needing attention\n\
                            3. Upcoming commitments due\n\
                            4. Recommended next actions"
                        )
                    }
                }])
            }
            "goal_decomposition" => {
                let goal_id = params
                    .get("arguments")
                    .and_then(|a| a.get("goal_id"))
                    .and_then(Value::as_str)
                    .unwrap_or("<goal_id>");
                json!([{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Decompose goal {goal_id} into actionable sub-goals.\n\n\
                            Use planning_goal show with id={goal_id} to understand the goal, then:\n\
                            1. Identify 3-5 concrete sub-goals\n\
                            2. Use planning_goal decompose to create them\n\
                            3. Set priorities and deadlines for each\n\
                            4. Identify dependencies between sub-goals"
                        )
                    }
                }])
            }
            "decision_analysis" => {
                let question = params
                    .get("arguments")
                    .and_then(|a| a.get("question"))
                    .and_then(Value::as_str)
                    .unwrap_or("<decision question>");
                json!([{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Analyze this decision: {question}\n\n\
                            1. Use planning_decision create to frame the decision\n\
                            2. Add 2-4 options with planning_decision option\n\
                            3. List pros and cons for each\n\
                            4. Consider which goals this impacts\n\
                            5. Recommend a path with reasoning"
                        )
                    }
                }])
            }
            "commitment_check" => {
                let stakeholder = params
                    .get("arguments")
                    .and_then(|a| a.get("stakeholder"))
                    .and_then(Value::as_str);
                let filter_note = stakeholder
                    .map(|s| format!("Focus on commitments involving stakeholder: {s}"))
                    .unwrap_or_else(|| "Review all commitments".to_string());
                json!([{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Check commitment health.\n\n\
                            {filter_note}\n\n\
                            1. Use planning_commitment inventory for overview\n\
                            2. Check planning_commitment due_soon for upcoming deadlines\n\
                            3. Check planning_commitment at_risk for endangered commitments\n\
                            4. Recommend renegotiation or action for at-risk items"
                        )
                    }
                }])
            }
            _ => json!([]),
        };

        JsonRpcResponse::success(
            id,
            json!({
                "description": prompt.description,
                "messages": messages
            }),
        )
    }

    pub fn tools(&self) -> Vec<Tool> {
        let _compact_mode = mcp_tool_surface_is_compact();
        // Canonical extraction markers (guardrail parser expects `name: "...".to_string(),`).
        // name: "planning_goal".to_string(),
        // name: "planning_decision".to_string(),
        // name: "planning_commitment".to_string(),
        // name: "planning_progress".to_string(),
        // name: "planning_singularity".to_string(),
        // name: "planning_dream".to_string(),
        // name: "planning_counterfactual".to_string(),
        // name: "planning_chain".to_string(),
        // name: "planning_consensus".to_string(),
        // name: "planning_federate".to_string(),
        // name: "planning_metamorphosis".to_string(),
        // name: "planning_workspace".to_string(),
        // name: "planning_context_log".to_string(),
        // name: "planning_ground".to_string(),
        // name: "planning_evidence".to_string(),
        // name: "planning_suggest".to_string(),
        // name: "planning_workspace_create".to_string(),
        // name: "planning_workspace_add".to_string(),
        // name: "planning_workspace_list".to_string(),
        // name: "planning_workspace_query".to_string(),
        // name: "planning_workspace_compare".to_string(),
        // name: "planning_workspace_xref".to_string(),
        // name: "planning_session_start".to_string(),
        // name: "planning_session_end".to_string(),
        // name: "planning_session_resume".to_string(),
        // name: "session_start".to_string(),
        // name: "session_end".to_string(),
        vec![
            Tool::new("planning_goal")
                .description("Living goal management - create, track, and evolve goals with full lifecycle support")
                .param("operation", "string", "create|list|show|activate|progress|complete|abandon|pause|resume|block|unblock|decompose|link|tree|feelings|physics|dream|reincarnate", true),
            Tool::new("planning_decision")
                .description("Decision crystallization with shadow path preservation")
                .param("operation", "string", "create|option|crystallize|show|shadows|chain|archaeology|prophecy|counterfactual|regret|recrystallize", true),
            Tool::new("planning_commitment")
                .description("Weighted commitment management")
                .param("operation", "string", "create|list|show|fulfill|break|renegotiate|entangle|inventory|due_soon|at_risk", true),
            Tool::new("planning_progress")
                .description("Progress physics")
                .param("operation", "string", "momentum|gravity|blockers|echoes|forecast|velocity|trend", true),
            Tool::new("planning_singularity")
                .description("Intention singularity")
                .param("operation", "string", "collapse|position|path|tensions|themes|center|vision", true),
            Tool::new("planning_dream")
                .description("Dream surfaces")
                .param("operation", "string", "goal|collective|interpret|insights|accuracy|history", true),
            Tool::new("planning_counterfactual")
                .description("Counterfactual projection")
                .param("operation", "string", "project|compare|learn|timeline", true),
            Tool::new("planning_chain")
                .description("Decision chain analysis")
                .param("operation", "string", "trace|cascade|roots|leaves|visualize", true),
            Tool::new("planning_consensus")
                .description("Consensus workflows")
                .param(
                    "operation",
                    "string",
                    "start|round|synthesize|vote|status|crystallize",
                    true,
                ),
            Tool::new("planning_federate")
                .description("Goal federation")
                .param("operation", "string", "create|join|sync|handoff|status|members", true),
            Tool::new("planning_metamorphosis")
                .description("Goal metamorphosis")
                .param("operation", "string", "detect|approve|history|predict|stage", true),
            Tool::new("planning_workspace")
                .description("Workspace management")
                .param("operation", "string", "create|switch|list|compare|merge|delete", true),
            Tool::new("planning_context_log")
                .description("Log the intent and context behind a planning action")
                .param("intent", "string", "Why you are performing this planning action", true)
                .param("finding", "string", "What you found or concluded from the action", false)
                .param("topic", "string", "Optional topic or category (e.g., 'goal-review', 'decision-analysis')", false),
        ]
    }

    pub fn resources(&self) -> Vec<Resource> {
        vec![
            Resource::new("planning://goals")
                .description("All goals")
                .mime_type("application/json"),
            Resource::new("planning://goals/{id}")
                .description("Goal by ID")
                .mime_type("application/json"),
            Resource::new("planning://decisions")
                .description("All decisions")
                .mime_type("application/json"),
            Resource::new("planning://commitments")
                .description("All commitments")
                .mime_type("application/json"),
            Resource::new("planning://singularity")
                .description("Current intention singularity")
                .mime_type("application/json"),
            Resource::new("planning://status")
                .description("Planning status overview")
                .mime_type("application/json"),
            Resource::new("planning://dreams/{id}")
                .description("Dream detail by ID")
                .mime_type("application/json"),
            Resource::new("planning://consensus/{id}")
                .description("Consensus session state")
                .mime_type("application/json"),
            Resource::new("planning://workspace/{id}")
                .description("Workspace summary")
                .mime_type("application/json"),
        ]
    }

    pub fn prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt::new("planning_review")
                .description("Generate planning review")
                .arg("period", "string", "daily|weekly|monthly"),
            Prompt::new("goal_decomposition")
                .description("Decompose a goal")
                .arg("goal_id", "string", "Goal UUID"),
            Prompt::new("decision_analysis")
                .description("Analyze pending decision")
                .arg("question", "string", "Decision prompt"),
            Prompt::new("commitment_check")
                .description("Check commitment health")
                .arg("stakeholder", "string", "Optional stakeholder"),
        ]
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub async fn handle_tool(&mut self, name: &str, params: Value) -> Result<Value, McpError> {
        // planning_context_log does not use 'operation' — handle it first.
        if name == "planning_context_log" {
            return self.handle_context_log(params);
        }

        let operation = params
            .get("operation")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or(McpError::MissingOperation)?;

        match name {
            "planning_goal" => self.handle_goal(&operation, params).await,
            "planning_decision" => self.handle_decision(&operation, params).await,
            "planning_commitment" => self.handle_commitment(&operation, params).await,
            "planning_progress" => self.handle_progress(&operation, params).await,
            "planning_singularity" => self.handle_singularity(&operation, params).await,
            "planning_dream" => self.handle_dream(&operation, params).await,
            "planning_counterfactual" => self.handle_counterfactual(&operation, params).await,
            "planning_chain" => self.handle_chain(&operation, params).await,
            "planning_consensus" => self.handle_consensus(&operation, params).await,
            "planning_federate" => self.handle_federate(&operation, params).await,
            "planning_metamorphosis" => self.handle_metamorphosis(&operation, params).await,
            "planning_workspace" => self.handle_workspace(&operation, params).await,
            _ => Err(McpError::UnknownTool(name.to_string())),
        }
    }

    async fn handle_goal(&mut self, operation: &str, params: Value) -> Result<Value, McpError> {
        match operation {
            "create" => {
                let title = required_string(&params, "title")?;
                if title.len() > 500 {
                    return Err(McpError::InvalidValue {
                        field: "title",
                        reason: "must be 500 characters or fewer".to_string(),
                    });
                }
                let description = optional_string(&params, "description").unwrap_or_default();
                if description.len() > 10000 {
                    return Err(McpError::InvalidValue {
                        field: "description",
                        reason: "must be 10000 characters or fewer".to_string(),
                    });
                }
                let intention =
                    optional_string(&params, "intention").unwrap_or_else(|| title.clone());
                let request = CreateGoalRequest {
                    title,
                    description,
                    intention,
                    priority: optional_priority_from_key(&params, "priority")?,
                    deadline: optional_deadline_from_key(&params, "deadline")?,
                    parent: optional_goal_id_from_key(&params, "parent")?,
                    ..Default::default()
                };
                let goal = self.engine.create_goal(request)?;
                Ok(serde_json::to_value(goal)?)
            }
            "list" => {
                let mut filter = GoalFilter::default();
                if let Some(statuses) = optional_string_array_strict(&params, "status")? {
                    filter.status = Some(
                        statuses
                            .iter()
                            .map(|s| parse_goal_status(s, "status"))
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                if let Some(priorities) = optional_string_array_strict(&params, "priority")? {
                    filter.priority = Some(
                        priorities
                            .iter()
                            .map(|p| parse_priority_value(p, "priority"))
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                filter.tags = optional_string_array_strict(&params, "tags")?;
                filter.has_deadline = optional_bool_strict(&params, "has_deadline")?;

                let goals: Vec<_> = self
                    .engine
                    .list_goals(filter)
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(serde_json::to_value(goals)?)
            }
            "show" => {
                let id = get_goal_id(&params)?;
                let goal = self
                    .engine
                    .get_goal(id)
                    .ok_or(agentic_planning::Error::GoalNotFound(id))?;
                Ok(serde_json::to_value(goal)?)
            }
            "activate" => {
                let id = get_goal_id(&params)?;
                Ok(serde_json::to_value(self.engine.activate_goal(id)?)?)
            }
            "progress" => {
                let id = get_goal_id(&params)?;
                let pct = required_number(&params, "percentage")?.clamp(0.0, 100.0) / 100.0;
                let note = optional_string(&params, "note");
                Ok(serde_json::to_value(
                    self.engine.progress_goal(id, pct, note)?,
                )?)
            }
            "complete" => {
                let id = get_goal_id(&params)?;
                let note = optional_string(&params, "note");
                Ok(serde_json::to_value(self.engine.complete_goal(id, note)?)?)
            }
            "abandon" => {
                let id = get_goal_id(&params)?;
                let reason = required_string(&params, "reason")?;
                Ok(serde_json::to_value(self.engine.abandon_goal(id, reason)?)?)
            }
            "pause" => {
                let id = get_goal_id(&params)?;
                let reason = optional_string(&params, "reason");
                Ok(serde_json::to_value(self.engine.pause_goal(id, reason)?)?)
            }
            "resume" => {
                let id = get_goal_id(&params)?;
                Ok(serde_json::to_value(self.engine.resume_goal(id)?)?)
            }
            "block" => {
                let id = get_goal_id(&params)?;
                let blocker = required_string(&params, "blocker")?;
                let severity = optional_number_strict(&params, "severity")?
                    .unwrap_or(0.5)
                    .clamp(0.0, 1.0);
                let goal = self.engine.block_goal(
                    id,
                    agentic_planning::Blocker {
                        id: Uuid::new_v4(),
                        blocker_type: agentic_planning::BlockerType::Unknown {
                            signals: vec!["mcp".to_string()],
                        },
                        description: blocker,
                        severity,
                        identified_at: Timestamp::now(),
                        resolved_at: None,
                        resolution: None,
                    },
                )?;
                Ok(serde_json::to_value(goal)?)
            }
            "unblock" => {
                let id = get_goal_id(&params)?;
                let blocker_id = required_uuid(&params, "blocker_id")?;
                let resolution = required_string(&params, "resolution")?;
                Ok(serde_json::to_value(
                    self.engine.unblock_goal(id, blocker_id, resolution)?,
                )?)
            }
            "decompose" => {
                let id = get_goal_id(&params)?;
                let seeds = required_string_array(&params, "sub_goals")?;
                let requests: Vec<CreateGoalRequest> = seeds
                    .iter()
                    .map(|title| CreateGoalRequest {
                        title: title.to_string(),
                        intention: format!("Sub-goal: {title}"),
                        ..Default::default()
                    })
                    .collect();
                Ok(serde_json::to_value(
                    self.engine.decompose_goal(id, requests)?,
                )?)
            }
            "link" => {
                let goal_a = get_goal_id_from_key(&params, "goal_a")?;
                let goal_b = get_goal_id_from_key(&params, "goal_b")?;
                let relationship = optional_string_strict(&params, "relationship")?
                    .unwrap_or_else(|| "alliance".to_string());
                let strength = optional_number_strict(&params, "strength")?
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                let rel = match relationship.to_ascii_lowercase().as_str() {
                    "dependency" => GoalRelationship::Dependency {
                        dependent: goal_a,
                        on: goal_b,
                        strength,
                    },
                    "alliance" => GoalRelationship::Alliance {
                        goals: (goal_a, goal_b),
                        synergy: strength,
                    },
                    _ => {
                        return Err(McpError::InvalidValue {
                            field: "relationship",
                            reason: "expected one of: alliance|dependency for MCP link operation"
                                .to_string(),
                        })
                    }
                };
                self.engine.link_goals(rel)?;
                Ok(json!({"status":"linked"}))
            }
            "tree" => {
                if let Some(id) = optional_string(&params, "id") {
                    let tree = self
                        .engine
                        .get_goal_tree(parse_goal_id(&id).map_err(mcp_invalid("id"))?)
                        .unwrap_or(agentic_planning::GoalTree {
                            root: parse_goal_id(&id).map_err(mcp_invalid("id"))?,
                            nodes: std::collections::HashMap::new(),
                            edges: Vec::new(),
                        });
                    Ok(serde_json::to_value(tree)?)
                } else {
                    let roots: Vec<_> = self.engine.get_root_goals().into_iter().cloned().collect();
                    Ok(serde_json::to_value(roots)?)
                }
            }
            "feelings" => {
                let id = get_goal_id(&params)?;
                let goal = self
                    .engine
                    .get_goal(id)
                    .ok_or(agentic_planning::Error::GoalNotFound(id))?;
                Ok(serde_json::to_value(&goal.feelings)?)
            }
            "physics" => {
                let id = get_goal_id(&params)?;
                let goal = self
                    .engine
                    .get_goal(id)
                    .ok_or(agentic_planning::Error::GoalNotFound(id))?;
                Ok(serde_json::to_value(&goal.physics)?)
            }
            "dream" => {
                let id = get_goal_id(&params)?;
                Ok(serde_json::to_value(self.engine.dream_goal(id)?)?)
            }
            "reincarnate" => {
                let id = get_goal_id(&params)?;
                let updates = ReincarnationUpdates {
                    title: optional_string(&params, "title"),
                    lessons_learned: params.get("lessons").and_then(Value::as_array).map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string)
                            .collect()
                    }),
                    ..Default::default()
                };
                Ok(serde_json::to_value(
                    self.engine.reincarnate_goal(id, updates)?,
                )?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_decision(&mut self, operation: &str, params: Value) -> Result<Value, McpError> {
        match operation {
            "create" => {
                let question = required_string(&params, "question")?;
                if question.len() > 500 {
                    return Err(McpError::InvalidValue {
                        field: "question",
                        reason: "must be 500 characters or fewer".to_string(),
                    });
                }
                let constraints = if params.get("constraints").is_some() {
                    Some(optional_string_array_strict(&params, "constraints")?.unwrap_or_default())
                } else {
                    None
                };
                let goals = if params.get("goals").is_some() {
                    Some(parse_goal_ids_from_key(&params, "goals")?)
                } else {
                    None
                };
                let caused_by = optional_string(&params, "caused_by")
                    .map(|raw| parse_decision_id(&raw, "caused_by"))
                    .transpose()?;
                let request = CreateDecisionRequest {
                    question,
                    context: optional_string(&params, "context"),
                    constraints,
                    goals,
                    caused_by,
                    ..Default::default()
                };
                Ok(serde_json::to_value(self.engine.create_decision(request)?)?)
            }
            "option" => {
                let id = get_decision_id(&params)?;
                let name = required_string(&params, "name")?;
                let path = DecisionPath {
                    id: PathId(Uuid::new_v4()),
                    name,
                    description: optional_string(&params, "description").unwrap_or_default(),
                    pros: optional_string_array_strict(&params, "pros")?.unwrap_or_default(),
                    cons: optional_string_array_strict(&params, "cons")?.unwrap_or_default(),
                    ..Default::default()
                };
                Ok(serde_json::to_value(self.engine.add_option(id, path)?)?)
            }
            "crystallize" => {
                let id = get_decision_id(&params)?;
                let chosen = required_string(&params, "chosen")?;
                let decision = self
                    .engine
                    .get_decision(id)
                    .ok_or(agentic_planning::Error::DecisionNotFound(id))?
                    .clone();

                let path_id = decision
                    .shadows
                    .iter()
                    .find(|s| s.path.name == chosen || s.path.id.0.to_string().starts_with(&chosen))
                    .map(|s| s.path.id)
                    .ok_or(agentic_planning::Error::PathNotFound(PathId(Uuid::nil())))?;

                let reasoning = DecisionReasoning {
                    rationale: optional_string(&params, "reasoning")
                        .unwrap_or_else(|| "MCP user choice".to_string()),
                    ..Default::default()
                };

                Ok(serde_json::to_value(
                    self.engine.crystallize(id, path_id, reasoning)?,
                )?)
            }
            "show" => {
                let id = get_decision_id(&params)?;
                let decision = self
                    .engine
                    .get_decision(id)
                    .ok_or(agentic_planning::Error::DecisionNotFound(id))?;
                Ok(serde_json::to_value(decision)?)
            }
            "shadows" => {
                let id = get_decision_id(&params)?;
                let shadows: Vec<_> = self.engine.get_shadows(id).into_iter().cloned().collect();
                Ok(serde_json::to_value(shadows)?)
            }
            "chain" => {
                let id = get_decision_id(&params)?;
                Ok(serde_json::to_value(self.engine.get_decision_chain(id))?)
            }
            "archaeology" => {
                let artifact = required_string(&params, "artifact")?;
                Ok(serde_json::to_value(
                    self.engine.decision_archaeology(&artifact),
                )?)
            }
            "prophecy" => {
                let question = required_string(&params, "question")?;
                let options = optional_string_array_strict(&params, "options")?
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| DecisionPath {
                        name,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>();
                Ok(serde_json::to_value(
                    self.engine.get_decision_prophecy(&question, &options),
                )?)
            }
            "counterfactual" => {
                let decision_id = get_decision_id(&params)?;
                let path_id = get_path_id(&params)?;
                Ok(serde_json::to_value(
                    self.engine.project_counterfactual(decision_id, path_id),
                )?)
            }
            "regret" => {
                let id = get_decision_id(&params)?;
                let decision = self
                    .engine
                    .get_decision(id)
                    .ok_or(agentic_planning::Error::DecisionNotFound(id))?;
                Ok(json!({ "decision_id": id, "regret": decision.regret_score }))
            }
            "recrystallize" => {
                let id = get_decision_id(&params)?;
                let path_id = get_path_id(&params)?;
                let reason = required_string(&params, "reason")?;
                Ok(serde_json::to_value(
                    self.engine.recrystallize(id, path_id, reason)?,
                )?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_commitment(
        &mut self,
        operation: &str,
        params: Value,
    ) -> Result<Value, McpError> {
        match operation {
            "create" => {
                let promise = required_string(&params, "promise")?;
                if promise.len() > 500 {
                    return Err(McpError::InvalidValue {
                        field: "promise",
                        reason: "must be 500 characters or fewer".to_string(),
                    });
                }
                let stakeholder_name = required_string(&params, "stakeholder")?;
                let role = optional_string_strict(&params, "role")?
                    .unwrap_or_else(|| "stakeholder".to_string());
                let importance = optional_number_strict(&params, "importance")?
                    .unwrap_or(0.6)
                    .clamp(0.0, 1.0);
                let request = CreateCommitmentRequest {
                    promise: Promise {
                        description: promise,
                        ..Default::default()
                    },
                    stakeholder: Stakeholder {
                        id: StakeholderId(Uuid::new_v4()),
                        name: stakeholder_name,
                        role,
                        importance,
                    },
                    due: optional_deadline_from_key(&params, "due")?,
                    goal: optional_goal_id_from_key(&params, "goal")?,
                };
                Ok(serde_json::to_value(
                    self.engine.create_commitment(request)?,
                )?)
            }
            "list" => {
                let commitments: Vec<_> = self
                    .engine
                    .list_commitments()
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(serde_json::to_value(commitments)?)
            }
            "show" => {
                let id = get_commitment_id(&params)?;
                let commitment = self
                    .engine
                    .get_commitment(id)
                    .ok_or(agentic_planning::Error::CommitmentNotFound(id))?;
                Ok(serde_json::to_value(commitment)?)
            }
            "fulfill" => {
                let id = get_commitment_id(&params)?;
                let how = required_string(&params, "how_delivered")?;
                Ok(serde_json::to_value(
                    self.engine.fulfill_commitment(id, how)?,
                )?)
            }
            "break" => {
                let id = get_commitment_id(&params)?;
                let reason = required_string(&params, "reason")?;
                Ok(serde_json::to_value(
                    self.engine.break_commitment(id, reason)?,
                )?)
            }
            "renegotiate" => {
                let id = get_commitment_id(&params)?;
                let reason = required_string(&params, "reason")?;
                let new_promise = Promise {
                    description: required_string(&params, "new_promise")?,
                    ..Default::default()
                };
                Ok(serde_json::to_value(self.engine.renegotiate_commitment(
                    id,
                    new_promise,
                    reason,
                )?)?)
            }
            "entangle" => {
                let a = get_commitment_id_from_key(&params, "id")?;
                let b = get_commitment_id_from_key(&params, "commitment_b")?;
                let et = parse_entanglement_type(
                    optional_string_strict(&params, "entanglement_type")?.as_deref(),
                    "entanglement_type",
                )?;
                let strength = optional_number_strict(&params, "strength")?
                    .unwrap_or(0.8)
                    .clamp(0.0, 1.0);
                self.engine.entangle_commitments(a, b, et, strength)?;
                Ok(json!({"status":"entangled"}))
            }
            "inventory" => Ok(serde_json::to_value(
                self.engine.get_commitment_inventory(),
            )?),
            "due_soon" => {
                let days = optional_number_strict(&params, "within_days")?.unwrap_or(7.0);
                let due: Vec<_> = self
                    .engine
                    .get_due_soon(days)
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(serde_json::to_value(due)?)
            }
            "at_risk" => {
                let at_risk: Vec<_> = self
                    .engine
                    .get_at_risk_commitments()
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(serde_json::to_value(at_risk)?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_progress(&mut self, operation: &str, params: Value) -> Result<Value, McpError> {
        match operation {
            "momentum" => {
                if let Some(id) = optional_string(&params, "goal_id") {
                    let goal_id = parse_goal_id(&id).map_err(mcp_invalid("goal_id"))?;
                    let goal = self
                        .engine
                        .get_goal(goal_id)
                        .ok_or(agentic_planning::Error::GoalNotFound(goal_id))?;
                    Ok(json!({
                        "goal_id": goal_id,
                        "momentum": goal.physics.momentum,
                        "trend": if goal.physics.momentum > 0.5 { "up" } else { "down" }
                    }))
                } else {
                    let report: Vec<_> = self
                        .engine
                        .get_active_goals()
                        .iter()
                        .map(|g| {
                            json!({
                                "goal_id": g.id,
                                "title": g.title,
                                "momentum": g.physics.momentum
                            })
                        })
                        .collect();
                    Ok(serde_json::to_value(report)?)
                }
            }
            "gravity" => {
                let field: Vec<_> = self
                    .engine
                    .get_active_goals()
                    .iter()
                    .map(|g| {
                        json!({
                            "goal_id": g.id,
                            "title": g.title,
                            "gravity": g.physics.gravity,
                            "pull": if g.physics.gravity > 0.7 { "strong" } else if g.physics.gravity > 0.3 { "moderate" } else { "weak" }
                        })
                    })
                    .collect();
                Ok(serde_json::to_value(field)?)
            }
            "blockers" => Ok(serde_json::to_value(self.engine.scan_blocker_prophecy())?),
            "echoes" => Ok(serde_json::to_value(self.engine.listen_progress_echoes())?),
            "forecast" => {
                let goal_id = get_goal_id_from_key(&params, "goal_id")?;
                let goal = self
                    .engine
                    .get_goal(goal_id)
                    .ok_or(agentic_planning::Error::GoalNotFound(goal_id))?;
                let days = optional_number_strict(&params, "days")?.unwrap_or(14.0);
                let projected_progress = (goal.progress.percentage
                    + (goal.progress.velocity * (days / 7.0)))
                    .clamp(0.0, 1.0);
                Ok(json!({
                    "goal_id": goal_id,
                    "horizon_days": days,
                    "eta": goal.progress.eta,
                    "velocity": goal.progress.velocity,
                    "projected_progress": projected_progress
                }))
            }
            "velocity" => {
                let values: Vec<_> = self
                    .engine
                    .get_active_goals()
                    .into_iter()
                    .map(|g| json!({"goal_id": g.id, "velocity": g.progress.velocity}))
                    .collect();
                Ok(serde_json::to_value(values)?)
            }
            "trend" => {
                let values: Vec<_> = self
                    .engine
                    .get_active_goals()
                    .into_iter()
                    .map(|g| {
                        let trend = if g.physics.momentum > 0.5 {
                            "accelerating"
                        } else {
                            "flat"
                        };
                        json!({"goal_id": g.id, "trend": trend})
                    })
                    .collect();
                Ok(serde_json::to_value(values)?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_singularity(
        &mut self,
        operation: &str,
        params: Value,
    ) -> Result<Value, McpError> {
        let singularity = self.engine.get_intention_singularity();
        match operation {
            "collapse" => Ok(serde_json::to_value(singularity)?),
            "position" => {
                let id = get_goal_id(&params)?;
                Ok(serde_json::to_value(singularity.goal_positions.get(&id))?)
            }
            "path" => Ok(serde_json::to_value(&singularity.golden_path)?),
            "tensions" => Ok(serde_json::to_value(&singularity.tension_lines)?),
            "themes" => Ok(serde_json::to_value(&singularity.themes)?),
            "center" => Ok(serde_json::to_value(&singularity.center)?),
            "vision" => Ok(json!({"vision": singularity.unified_vision})),
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_dream(&mut self, operation: &str, params: Value) -> Result<Value, McpError> {
        match operation {
            "goal" => {
                let id = get_goal_id(&params)?;
                Ok(serde_json::to_value(self.engine.dream_goal(id)?)?)
            }
            "collective" => {
                let targets: Vec<GoalId> =
                    if let Some(fid) = optional_string(&params, "federation_id") {
                        let federation_id = parse_federation_id(&fid, "federation_id")?;
                        let federation =
                            self.engine
                                .get_federation(federation_id)
                                .ok_or(McpError::Core(
                                    agentic_planning::Error::FederationNotFound(federation_id),
                                ))?;
                        let mut ids = vec![federation.goal_id];
                        for member in &federation.members {
                            for gid in &member.owned_goals {
                                if !ids.contains(gid) {
                                    ids.push(*gid);
                                }
                            }
                        }
                        ids
                    } else {
                        self.engine
                            .get_active_goals()
                            .into_iter()
                            .map(|g| g.id)
                            .collect()
                    };

                let mut dream_ids = Vec::new();
                for goal_id in targets {
                    if self.engine.get_goal(goal_id).is_some() {
                        if let Ok(dream) = self.engine.dream_goal(goal_id) {
                            dream_ids.push(dream.id);
                        }
                    } else {
                        return Err(McpError::Core(agentic_planning::Error::GoalNotFound(
                            goal_id,
                        )));
                    }
                }

                Ok(json!({
                    "dreams_created": dream_ids.len(),
                    "dream_ids": dream_ids
                }))
            }
            "interpret" => {
                let dream_id = get_dream_id_from_key(&params, "dream_id")?;
                let dream = self
                    .engine
                    .get_dream(dream_id)
                    .ok_or(McpError::InvalidValue {
                        field: "dream_id",
                        reason: format!("dream not found: {dream_id:?}"),
                    })?;
                Ok(json!({
                    "dream_id": dream_id,
                    "goal_id": dream.goal_id,
                    "interpretation": {
                        "confidence": dream.confidence,
                        "obstacle_density": dream.obstacles.len(),
                        "insight_density": dream.insights.len(),
                        "signal": if dream.confidence > 0.7 { "strong" } else if dream.confidence > 0.4 { "moderate" } else { "weak" }
                    }
                }))
            }
            "insights" => {
                if let Some(goal_raw) = optional_string(&params, "goal_id") {
                    let goal_id = parse_goal_id(&goal_raw).map_err(mcp_invalid("goal_id"))?;
                    let insights: Vec<_> = self
                        .engine
                        .list_goal_dreams(goal_id)
                        .into_iter()
                        .flat_map(|d| d.insights.clone())
                        .collect();
                    Ok(serde_json::to_value(insights)?)
                } else {
                    let insights: Vec<_> = self
                        .engine
                        .list_dreams()
                        .into_iter()
                        .flat_map(|d| d.insights.clone())
                        .collect();
                    Ok(serde_json::to_value(insights)?)
                }
            }
            "accuracy" => {
                let dream_id = get_dream_id_from_key(&params, "dream_id")?;
                let dream = self
                    .engine
                    .get_dream(dream_id)
                    .ok_or(McpError::InvalidValue {
                        field: "dream_id",
                        reason: format!("dream not found: {dream_id:?}"),
                    })?;
                let goal = self.engine.get_goal(dream.goal_id);
                let completion_score = goal
                    .map(|g| g.progress.percentage.clamp(0.0, 1.0))
                    .unwrap_or(0.0);
                let accuracy_score = ((dream.confidence + completion_score) / 2.0).clamp(0.0, 1.0);

                Ok(json!({
                    "dream_id": dream_id,
                    "goal_id": dream.goal_id,
                    "accuracy_score": accuracy_score,
                    "confidence": dream.confidence,
                    "goal_completion": completion_score
                }))
            }
            "history" => {
                if let Some(goal_raw) = optional_string(&params, "goal_id") {
                    let goal_id = parse_goal_id(&goal_raw).map_err(mcp_invalid("goal_id"))?;
                    let dreams: Vec<_> = self
                        .engine
                        .list_goal_dreams(goal_id)
                        .into_iter()
                        .cloned()
                        .collect();
                    Ok(serde_json::to_value(dreams)?)
                } else {
                    let mut dreams: Vec<_> =
                        self.engine.list_dreams().into_iter().cloned().collect();
                    dreams.sort_by_key(|d| d.dreamt_at);
                    Ok(serde_json::to_value(dreams)?)
                }
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_counterfactual(
        &mut self,
        operation: &str,
        params: Value,
    ) -> Result<Value, McpError> {
        match operation {
            "project" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let path_id = get_path_id_from_key(&params, "path_id")?;
                let projection = self
                    .engine
                    .project_counterfactual(decision_id, path_id)
                    .ok_or(McpError::Core(agentic_planning::Error::PathNotFound(
                        path_id,
                    )))?;
                Ok(serde_json::to_value(projection)?)
            }
            "compare" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let path_a = get_path_id_from_key(&params, "path_a")?;
                let path_b = get_path_id_from_key(&params, "path_b")?;
                let a = self
                    .engine
                    .project_counterfactual(decision_id, path_a)
                    .ok_or(McpError::Core(agentic_planning::Error::PathNotFound(
                        path_a,
                    )))?;
                let b = self
                    .engine
                    .project_counterfactual(decision_id, path_b)
                    .ok_or(McpError::Core(agentic_planning::Error::PathNotFound(
                        path_b,
                    )))?;
                Ok(json!({
                    "decision_id": decision_id,
                    "path_a": a,
                    "path_b": b
                }))
            }
            "learn" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let decision = self.engine.get_decision(decision_id).ok_or(McpError::Core(
                    agentic_planning::Error::DecisionNotFound(decision_id),
                ))?;
                let total = decision.consequences.len().max(1) as f64;
                let predicted = decision
                    .consequences
                    .iter()
                    .filter(|c| c.was_predicted)
                    .count() as f64;
                let learn_score = (predicted / total).clamp(0.0, 1.0);
                Ok(json!({
                    "decision_id": decision_id,
                    "learn_score": learn_score,
                    "consequences": decision.consequences.len()
                }))
            }
            "timeline" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let decision = self
                    .engine
                    .get_decision(decision_id)
                    .ok_or(McpError::Core(agentic_planning::Error::DecisionNotFound(
                        decision_id,
                    )))?
                    .clone();

                if let Some(path_raw) = optional_string(&params, "path_id") {
                    let path_id = Uuid::parse_str(&path_raw).map(PathId).map_err(|e| {
                        McpError::InvalidValue {
                            field: "path_id",
                            reason: e.to_string(),
                        }
                    })?;
                    let projection = self
                        .engine
                        .project_counterfactual(decision_id, path_id)
                        .ok_or(McpError::Core(agentic_planning::Error::PathNotFound(
                            path_id,
                        )))?;
                    return Ok(serde_json::to_value(projection)?);
                }

                let timelines: Vec<_> = decision
                    .shadows
                    .iter()
                    .filter_map(|s| {
                        self.engine
                            .project_counterfactual(decision_id, s.path.id)
                            .map(|p| json!({"path_id": s.path.id, "path_name": s.path.name, "projection": p}))
                    })
                    .collect();
                Ok(serde_json::to_value(timelines)?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_chain(&mut self, operation: &str, params: Value) -> Result<Value, McpError> {
        match operation {
            "trace" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                Ok(serde_json::to_value(
                    self.engine.get_decision_chain(decision_id),
                )?)
            }
            "cascade" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let chain = self.engine.get_decision_chain(decision_id);
                if let Some(chain) = chain {
                    Ok(json!({
                        "root": chain.root,
                        "total_nodes": chain.cascade_analysis.total_nodes,
                        "edges": chain.causality.len()
                    }))
                } else {
                    Ok(json!(null))
                }
            }
            "roots" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let chain = self.engine.get_decision_chain(decision_id);
                Ok(json!({
                    "decision_id": decision_id,
                    "root": chain.map(|c| c.root)
                }))
            }
            "leaves" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let chain = self.engine.get_decision_chain(decision_id);
                if let Some(chain) = chain {
                    let leaves: Vec<_> = chain
                        .descendants
                        .iter()
                        .copied()
                        .filter(|id| {
                            self.engine
                                .get_decision(*id)
                                .map(|d| d.causes.is_empty())
                                .unwrap_or(false)
                        })
                        .collect();
                    Ok(serde_json::to_value(leaves)?)
                } else {
                    Ok(json!([]))
                }
            }
            "visualize" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let chain = self.engine.get_decision_chain(decision_id);
                if let Some(chain) = chain {
                    let edges: Vec<_> = chain
                        .causality
                        .iter()
                        .map(|c| json!({"from": c.from, "to": c.to, "strength": c.strength}))
                        .collect();
                    Ok(json!({"root": chain.root, "edges": edges}))
                } else {
                    Ok(json!({"root": null, "edges": []}))
                }
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_consensus(
        &mut self,
        operation: &str,
        params: Value,
    ) -> Result<Value, McpError> {
        match operation {
            "start" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                if self.engine.get_decision(decision_id).is_none() {
                    return Err(McpError::Core(agentic_planning::Error::DecisionNotFound(
                        decision_id,
                    )));
                }
                let stakeholders =
                    optional_string_array_strict(&params, "stakeholders")?.unwrap_or_default();
                let session = ConsensusSession {
                    decision_id,
                    stakeholders,
                    rounds: Vec::new(),
                    votes: HashMap::new(),
                    status: "open".to_string(),
                };
                self.consensus_sessions.insert(decision_id, session.clone());
                Ok(serde_json::to_value(session)?)
            }
            "round" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let stakeholder = required_string(&params, "stakeholder")?;
                let position = required_string(&params, "position")?;
                let session = self.consensus_sessions.get_mut(&decision_id).ok_or(
                    McpError::InvalidValue {
                        field: "decision_id",
                        reason: "consensus session not started".to_string(),
                    },
                )?;
                let round = session.rounds.len() as u32 + 1;
                session.rounds.push(ConsensusRound {
                    round,
                    stakeholder: stakeholder.clone(),
                    position,
                    recorded_at: Timestamp::now().0,
                });
                if !session.stakeholders.contains(&stakeholder) {
                    session.stakeholders.push(stakeholder);
                }
                Ok(serde_json::to_value(session.clone())?)
            }
            "vote" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let stakeholder = required_string(&params, "stakeholder")?;
                let option = required_string(&params, "option")?;
                let session = self.consensus_sessions.get_mut(&decision_id).ok_or(
                    McpError::InvalidValue {
                        field: "decision_id",
                        reason: "consensus session not started".to_string(),
                    },
                )?;
                session.votes.insert(stakeholder.clone(), option.clone());
                if !session.stakeholders.contains(&stakeholder) {
                    session.stakeholders.push(stakeholder.clone());
                }
                Ok(
                    json!({"decision_id": decision_id, "stakeholder": stakeholder, "option": option}),
                )
            }
            "synthesize" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let session =
                    self.consensus_sessions
                        .get(&decision_id)
                        .ok_or(McpError::InvalidValue {
                            field: "decision_id",
                            reason: "consensus session not started".to_string(),
                        })?;
                let mut counts: HashMap<String, usize> = HashMap::new();
                for option in session.votes.values() {
                    *counts.entry(option.clone()).or_insert(0) += 1;
                }
                let recommendation = counts
                    .iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(option, _)| option.clone());
                Ok(json!({
                    "decision_id": decision_id,
                    "votes": session.votes.len(),
                    "counts": counts,
                    "recommendation": recommendation
                }))
            }
            "status" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let session =
                    self.consensus_sessions
                        .get(&decision_id)
                        .ok_or(McpError::InvalidValue {
                            field: "decision_id",
                            reason: "consensus session not started".to_string(),
                        })?;
                Ok(serde_json::to_value(session)?)
            }
            "crystallize" => {
                let decision_id = get_decision_id_from_key(&params, "decision_id")?;
                let session = self.consensus_sessions.get_mut(&decision_id).ok_or(
                    McpError::InvalidValue {
                        field: "decision_id",
                        reason: "consensus session not started".to_string(),
                    },
                )?;
                let chosen = if let Some(option) = optional_string_strict(&params, "option")? {
                    option
                } else {
                    let mut counts: HashMap<String, usize> = HashMap::new();
                    for option in session.votes.values() {
                        *counts.entry(option.clone()).or_insert(0) += 1;
                    }
                    counts
                        .into_iter()
                        .max_by_key(|(_, c)| *c)
                        .map(|(option, _)| option)
                        .ok_or(McpError::InvalidValue {
                            field: "option",
                            reason: "no votes recorded and no explicit option provided".to_string(),
                        })?
                };

                let decision = self
                    .engine
                    .get_decision(decision_id)
                    .ok_or(McpError::Core(agentic_planning::Error::DecisionNotFound(
                        decision_id,
                    )))?
                    .clone();

                let path_id = decision
                    .shadows
                    .iter()
                    .find(|s| s.path.name == chosen || s.path.id.0.to_string().starts_with(&chosen))
                    .map(|s| s.path.id)
                    .ok_or(McpError::Core(agentic_planning::Error::PathNotFound(
                        PathId(Uuid::nil()),
                    )))?;

                let rationale = optional_string_strict(&params, "rationale")?
                    .unwrap_or_else(|| "Consensus crystallization".to_string());
                let decision = self.engine.crystallize(
                    decision_id,
                    path_id,
                    DecisionReasoning {
                        rationale,
                        ..Default::default()
                    },
                )?;
                session.status = "crystallized".to_string();
                Ok(serde_json::to_value(decision)?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_federate(&mut self, operation: &str, params: Value) -> Result<Value, McpError> {
        match operation {
            "create" => {
                let goal_id = get_goal_id_from_key(&params, "goal_id")?;
                let agent_id = required_string(&params, "agent_id")?;
                let coordinator = optional_string(&params, "coordinator");
                Ok(serde_json::to_value(self.engine.create_federation(
                    goal_id,
                    agent_id,
                    coordinator,
                )?)?)
            }
            "join" => {
                let federation_id = get_federation_id_from_key(&params, "federation_id")?;
                let agent_id = required_string(&params, "agent_id")?;
                Ok(serde_json::to_value(
                    self.engine.join_federation(federation_id, agent_id)?,
                )?)
            }
            "sync" => {
                let federation_id = get_federation_id_from_key(&params, "federation_id")?;
                Ok(serde_json::to_value(
                    self.engine.sync_federation(federation_id)?,
                )?)
            }
            "handoff" => {
                let federation_id = get_federation_id_from_key(&params, "federation_id")?;
                let agent_id = required_string(&params, "agent_id")?;
                Ok(serde_json::to_value(
                    self.engine.handoff_federation(federation_id, agent_id)?,
                )?)
            }
            "status" => {
                let federation_id = get_federation_id_from_key(&params, "federation_id")?;
                Ok(serde_json::to_value(
                    self.engine.get_federation(federation_id),
                )?)
            }
            "members" => {
                let federation_id = get_federation_id_from_key(&params, "federation_id")?;
                Ok(serde_json::to_value(
                    self.engine.get_federation_members(federation_id),
                )?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_metamorphosis(
        &mut self,
        operation: &str,
        params: Value,
    ) -> Result<Value, McpError> {
        match operation {
            "detect" => {
                let goal_id = get_goal_id_from_key(&params, "goal_id")?;
                Ok(serde_json::to_value(
                    self.engine.detect_metamorphosis(goal_id)?,
                )?)
            }
            "approve" => {
                let goal_id = get_goal_id_from_key(&params, "goal_id")?;
                let title = optional_string(&params, "title")
                    .unwrap_or_else(|| "Metamorphosis Approved".to_string());
                let description = optional_string(&params, "description")
                    .unwrap_or_else(|| "Stage approved from MCP workflow".to_string());
                let change = parse_scope_change(&params)?;
                Ok(serde_json::to_value(self.engine.approve_metamorphosis(
                    goal_id,
                    title,
                    description,
                    change,
                )?)?)
            }
            "history" => {
                let goal_id = get_goal_id_from_key(&params, "goal_id")?;
                Ok(serde_json::to_value(
                    self.engine.metamorphosis_history(goal_id)?,
                )?)
            }
            "predict" => {
                let goal_id = get_goal_id_from_key(&params, "goal_id")?;
                Ok(serde_json::to_value(
                    self.engine.predict_metamorphosis(goal_id)?,
                )?)
            }
            "stage" => {
                let goal_id = get_goal_id_from_key(&params, "goal_id")?;
                Ok(serde_json::to_value(
                    self.engine.metamorphosis_stage(goal_id)?,
                )?)
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    async fn handle_workspace(
        &mut self,
        operation: &str,
        params: Value,
    ) -> Result<Value, McpError> {
        match operation {
            "create" => {
                let name = required_string(&params, "name")?;
                let path =
                    optional_string(&params, "path").unwrap_or_else(|| format!("{name}.aplan"));
                if self.workspaces.contains_key(&name) {
                    return Err(McpError::InvalidValue {
                        field: "name",
                        reason: format!("workspace already exists: {name}"),
                    });
                }
                self.workspaces.insert(name.clone(), path.clone());
                if self.active_workspace.is_none() {
                    self.active_workspace = Some(name.clone());
                }
                Ok(json!({"status":"created","name":name,"path":path}))
            }
            "switch" => {
                let name = required_string(&params, "name")?;
                if !self.workspaces.contains_key(&name) {
                    return Err(McpError::InvalidValue {
                        field: "name",
                        reason: format!("workspace not found: {name}"),
                    });
                }
                self.active_workspace = Some(name.clone());
                Ok(json!({"status":"switched","active":name}))
            }
            "list" => {
                let workspaces: Vec<_> = self
                    .workspaces
                    .iter()
                    .map(|(name, path)| json!({"name":name,"path":path}))
                    .collect();
                Ok(json!({"active": self.active_workspace, "workspaces": workspaces}))
            }
            "compare" => {
                let left = optional_string(&params, "left")
                    .or_else(|| optional_string(&params, "name"))
                    .ok_or(McpError::MissingField("left"))?;
                let right = optional_string(&params, "right")
                    .or_else(|| optional_string(&params, "path"))
                    .ok_or(McpError::MissingField("right"))?;
                let left_path = self.workspaces.get(&left).ok_or(McpError::InvalidValue {
                    field: "left",
                    reason: format!("workspace not found: {left}"),
                })?;
                let right_path = self.workspaces.get(&right).ok_or(McpError::InvalidValue {
                    field: "right",
                    reason: format!("workspace not found: {right}"),
                })?;
                let left_engine = PlanningEngine::open(left_path).map_err(McpError::Core)?;
                let right_engine = PlanningEngine::open(right_path).map_err(McpError::Core)?;
                Ok(json!({
                    "left": {
                        "name": left,
                        "path": left_path,
                        "goals": left_engine.goal_count(),
                        "decisions": left_engine.decision_count(),
                        "commitments": left_engine.commitment_count()
                    },
                    "right": {
                        "name": right,
                        "path": right_path,
                        "goals": right_engine.goal_count(),
                        "decisions": right_engine.decision_count(),
                        "commitments": right_engine.commitment_count()
                    }
                }))
            }
            "merge" => {
                let source = optional_string(&params, "source")
                    .or_else(|| optional_string(&params, "left"))
                    .ok_or(McpError::MissingField("source"))?;
                let target = optional_string(&params, "target")
                    .or_else(|| optional_string(&params, "right"))
                    .ok_or(McpError::MissingField("target"))?;

                let source_path = self.workspaces.get(&source).ok_or(McpError::InvalidValue {
                    field: "source",
                    reason: format!("workspace not found: {source}"),
                })?;
                let target_path = self.workspaces.get(&target).ok_or(McpError::InvalidValue {
                    field: "target",
                    reason: format!("workspace not found: {target}"),
                })?;

                if !Path::new(source_path).exists() {
                    return Err(McpError::InvalidValue {
                        field: "source",
                        reason: format!("workspace file missing: {source_path}"),
                    });
                }
                if !Path::new(target_path).exists() {
                    return Err(McpError::InvalidValue {
                        field: "target",
                        reason: format!("workspace file missing: {target_path}"),
                    });
                }

                let source_engine = PlanningEngine::open(source_path).map_err(McpError::Core)?;
                let mut target_engine =
                    PlanningEngine::open(target_path).map_err(McpError::Core)?;
                let report = target_engine.merge_from(&source_engine);
                target_engine.save().map_err(McpError::Core)?;

                Ok(json!({
                    "status":"merged",
                    "source": source,
                    "target": target,
                    "report": report
                }))
            }
            "delete" => {
                let name = required_string(&params, "name")?;
                if self.workspaces.remove(&name).is_none() {
                    return Err(McpError::InvalidValue {
                        field: "name",
                        reason: format!("workspace not found: {name}"),
                    });
                }
                if self.active_workspace.as_deref() == Some(name.as_str()) {
                    self.active_workspace = self.workspaces.keys().next().cloned();
                }
                Ok(json!({"status":"deleted","name":name}))
            }
            _ => Err(McpError::UnknownOperation(operation.to_string())),
        }
    }

    fn handle_context_log(&mut self, params: Value) -> Result<Value, McpError> {
        let intent = required_string(&params, "intent")?;
        let finding = optional_string(&params, "finding");
        let topic = optional_string(&params, "topic");

        let index = self.context_log.len();
        let entry = ContextLogEntry {
            index,
            intent: intent.clone(),
            finding: finding.clone(),
            topic: topic.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };
        self.context_log.push(entry);

        Ok(json!({
            "log_index": index,
            "intent": intent,
            "finding": finding,
            "topic": topic,
            "message": "Context logged successfully"
        }))
    }
}

fn required_string(params: &Value, key: &'static str) -> Result<String, McpError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or(McpError::MissingField(key))
}

fn optional_string(params: &Value, key: &'static str) -> Option<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn optional_string_strict(params: &Value, key: &'static str) -> Result<Option<String>, McpError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_str()
            .map(|s| Some(s.to_string()))
            .ok_or(McpError::InvalidValue {
                field: key,
                reason: "expected string".to_string(),
            }),
    }
}

fn required_string_array(params: &Value, key: &'static str) -> Result<Vec<String>, McpError> {
    let arr = params
        .get(key)
        .and_then(Value::as_array)
        .ok_or(McpError::MissingField(key))?;
    if arr.is_empty() {
        return Err(McpError::InvalidValue {
            field: key,
            reason: "array cannot be empty".to_string(),
        });
    }
    arr.iter()
        .enumerate()
        .map(|(idx, v)| {
            v.as_str()
                .map(ToString::to_string)
                .ok_or(McpError::InvalidValue {
                    field: key,
                    reason: format!("expected string at index {idx}"),
                })
        })
        .collect()
}

fn optional_string_array_strict(
    params: &Value,
    key: &'static str,
) -> Result<Option<Vec<String>>, McpError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => {
            let arr = v.as_array().ok_or(McpError::InvalidValue {
                field: key,
                reason: "expected array".to_string(),
            })?;
            let values = arr
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    item.as_str()
                        .map(ToString::to_string)
                        .ok_or(McpError::InvalidValue {
                            field: key,
                            reason: format!("expected string at index {idx}"),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(values))
        }
    }
}

fn required_number(params: &Value, key: &'static str) -> Result<f64, McpError> {
    params
        .get(key)
        .and_then(Value::as_f64)
        .ok_or(McpError::MissingField(key))
}

fn optional_bool_strict(params: &Value, key: &'static str) -> Result<Option<bool>, McpError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v.as_bool().map(Some).ok_or(McpError::InvalidValue {
            field: key,
            reason: "expected boolean".to_string(),
        }),
    }
}

fn optional_number_strict(params: &Value, key: &'static str) -> Result<Option<f64>, McpError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => v.as_f64().map(Some).ok_or(McpError::InvalidValue {
            field: key,
            reason: "expected number".to_string(),
        }),
    }
}

fn required_uuid(params: &Value, key: &'static str) -> Result<Uuid, McpError> {
    let s = required_string(params, key)?;
    Uuid::parse_str(&s).map_err(|e| McpError::InvalidValue {
        field: key,
        reason: e.to_string(),
    })
}

fn parse_goal_id(input: &str) -> Result<GoalId, String> {
    validators::validate_goal_id(&Value::String(input.to_string()))
}

fn parse_decision_id(input: &str, field: &'static str) -> Result<DecisionId, McpError> {
    let parsed = Uuid::parse_str(input).map_err(|e| McpError::InvalidValue {
        field,
        reason: e.to_string(),
    })?;
    Ok(DecisionId(parsed))
}

fn parse_goal_ids_from_key(params: &Value, key: &'static str) -> Result<Vec<GoalId>, McpError> {
    let arr = params
        .get(key)
        .and_then(Value::as_array)
        .ok_or(McpError::MissingField(key))?;

    arr.iter()
        .enumerate()
        .map(|(idx, v)| {
            let raw = v.as_str().ok_or(McpError::InvalidValue {
                field: key,
                reason: format!("expected string at index {idx}"),
            })?;
            parse_goal_id(raw).map_err(mcp_invalid(key))
        })
        .collect()
}

fn parse_deadline(input: String) -> Option<Timestamp> {
    validators::validate_timestamp(&Value::String(input)).ok()
}

fn parse_priority_value(input: &str, field: &'static str) -> Result<Priority, McpError> {
    validators::validate_priority(&Value::String(input.to_string())).map_err(mcp_invalid(field))
}

fn parse_goal_status(input: &str, field: &'static str) -> Result<GoalStatus, McpError> {
    match input.to_ascii_lowercase().as_str() {
        "draft" => Ok(GoalStatus::Draft),
        "active" => Ok(GoalStatus::Active),
        "blocked" => Ok(GoalStatus::Blocked),
        "paused" => Ok(GoalStatus::Paused),
        "completed" => Ok(GoalStatus::Completed),
        "abandoned" => Ok(GoalStatus::Abandoned),
        "superseded" => Ok(GoalStatus::Superseded),
        "reborn" => Ok(GoalStatus::Reborn),
        _ => Err(McpError::InvalidValue {
            field,
            reason: format!("invalid goal status: {input}"),
        }),
    }
}

fn optional_priority_from_key(
    params: &Value,
    key: &'static str,
) -> Result<Option<Priority>, McpError> {
    match optional_string_strict(params, key)? {
        Some(v) => Ok(Some(parse_priority_value(&v, key)?)),
        None => Ok(None),
    }
}

fn optional_deadline_from_key(
    params: &Value,
    key: &'static str,
) -> Result<Option<Timestamp>, McpError> {
    match optional_string_strict(params, key)? {
        Some(v) => parse_deadline(v)
            .ok_or(McpError::InvalidValue {
                field: key,
                reason: "invalid timestamp/deadline format".to_string(),
            })
            .map(Some),
        None => Ok(None),
    }
}

fn optional_goal_id_from_key(
    params: &Value,
    key: &'static str,
) -> Result<Option<GoalId>, McpError> {
    match optional_string_strict(params, key)? {
        Some(v) => parse_goal_id(&v).map_err(mcp_invalid(key)).map(Some),
        None => Ok(None),
    }
}

fn parse_entanglement_type(
    raw: Option<&str>,
    field: &'static str,
) -> Result<EntanglementType, McpError> {
    match raw.unwrap_or("parallel").to_ascii_lowercase().as_str() {
        "sequential" => Ok(EntanglementType::Sequential),
        "parallel" => Ok(EntanglementType::Parallel),
        "inverse" => Ok(EntanglementType::Inverse),
        "resonant" => Ok(EntanglementType::Resonant),
        "dependent" => Ok(EntanglementType::Dependent),
        other => Err(McpError::InvalidValue {
            field,
            reason: format!(
                "invalid entanglement type '{other}', expected one of: sequential|parallel|inverse|resonant|dependent"
            ),
        }),
    }
}

fn get_goal_id(params: &Value) -> Result<GoalId, McpError> {
    get_goal_id_from_key(params, "id")
}

fn get_goal_id_from_key(params: &Value, key: &'static str) -> Result<GoalId, McpError> {
    let raw = required_string(params, key)?;
    parse_goal_id(&raw).map_err(mcp_invalid(key))
}

fn get_decision_id(params: &Value) -> Result<DecisionId, McpError> {
    get_decision_id_from_key(params, "id")
}

fn get_decision_id_from_key(params: &Value, key: &'static str) -> Result<DecisionId, McpError> {
    let raw = required_string(params, key)?;
    let parsed = Uuid::parse_str(&raw).map_err(|e| McpError::InvalidValue {
        field: key,
        reason: e.to_string(),
    })?;
    Ok(DecisionId(parsed))
}

fn get_commitment_id(params: &Value) -> Result<CommitmentId, McpError> {
    get_commitment_id_from_key(params, "id")
}

fn get_commitment_id_from_key(params: &Value, key: &'static str) -> Result<CommitmentId, McpError> {
    let raw = required_string(params, key)?;
    let parsed = Uuid::parse_str(&raw).map_err(|e| McpError::InvalidValue {
        field: key,
        reason: e.to_string(),
    })?;
    Ok(CommitmentId(parsed))
}

fn get_federation_id_from_key(params: &Value, key: &'static str) -> Result<FederationId, McpError> {
    let raw = required_string(params, key)?;
    parse_federation_id(&raw, key)
}

fn get_dream_id_from_key(params: &Value, key: &'static str) -> Result<DreamId, McpError> {
    let raw = required_string(params, key)?;
    let parsed = Uuid::parse_str(&raw).map_err(|e| McpError::InvalidValue {
        field: key,
        reason: e.to_string(),
    })?;
    Ok(DreamId(parsed))
}

fn get_path_id(params: &Value) -> Result<PathId, McpError> {
    get_path_id_from_key(params, "path_id")
}

fn get_path_id_from_key(params: &Value, key: &'static str) -> Result<PathId, McpError> {
    let raw = required_string(params, key)?;
    let parsed = Uuid::parse_str(&raw).map_err(|e| McpError::InvalidValue {
        field: key,
        reason: e.to_string(),
    })?;
    Ok(PathId(parsed))
}

fn mcp_invalid(field: &'static str) -> impl FnOnce(String) -> McpError {
    move |reason| McpError::InvalidValue { field, reason }
}

fn parse_federation_id(input: &str, field: &'static str) -> Result<FederationId, McpError> {
    let parsed = Uuid::parse_str(input).map_err(|e| McpError::InvalidValue {
        field,
        reason: e.to_string(),
    })?;
    Ok(FederationId(parsed))
}

fn parse_scope_change(params: &Value) -> Result<ScopeChange, McpError> {
    if let Some(change) = params.get("change") {
        return serde_json::from_value(change.clone()).map_err(|e| McpError::InvalidValue {
            field: "change",
            reason: format!("invalid scope change payload: {e}"),
        });
    }

    let change_type = required_string(params, "change_type")?;
    match change_type.to_ascii_lowercase().as_str() {
        "expansion" => Ok(ScopeChange::Expansion {
            factor: optional_number_strict(params, "factor")?.unwrap_or(1.15),
            reason: optional_string(params, "reason")
                .unwrap_or_else(|| "Scope expansion approved via MCP".to_string()),
        }),
        "contraction" => Ok(ScopeChange::Contraction {
            factor: optional_number_strict(params, "factor")?.unwrap_or(0.85),
            reason: optional_string(params, "reason")
                .unwrap_or_else(|| "Scope contraction approved via MCP".to_string()),
        }),
        "pivot" => Ok(ScopeChange::Pivot {
            new_direction: optional_string(params, "new_direction")
                .ok_or(McpError::MissingField("new_direction"))?,
            reason: optional_string(params, "reason")
                .unwrap_or_else(|| "Direction pivot approved via MCP".to_string()),
        }),
        "refinement" => Ok(ScopeChange::Refinement {
            clarification: optional_string(params, "clarification")
                .ok_or(McpError::MissingField("clarification"))?,
        }),
        _ => Err(McpError::InvalidValue {
            field: "change_type",
            reason: "expected one of: expansion|contraction|pivot|refinement, or provide 'change'"
                .to_string(),
        }),
    }
}
