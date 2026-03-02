use agentic_planning::{
    Blocker, BlockerType, CommitmentId, CreateCommitmentRequest, CreateDecisionRequest,
    CreateGoalRequest, DecisionId, DecisionPath, DecisionReasoning, DreamId, EntanglementType,
    FederationId, GoalFilter, GoalId, GoalRelationship, GoalStatus, PathId, PlanningEngine,
    Priority, Promise, ReincarnationUpdates, ScopeChange, Stakeholder, StakeholderId, Timestamp,
};
use agentic_planning_mcp::{PlanningMcpServer, MAX_CONTENT_LENGTH_BYTES};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

type CliResult<T> = Result<T, String>;

#[derive(Parser)]
#[command(name = "agentic-planning")]
#[command(bin_name = "aplan")]
#[command(about = "Persistent intention infrastructure for AI agents")]
#[command(version)]
struct Cli {
    #[arg(short, long, global = true)]
    file: Option<std::path::PathBuf>,

    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Table,
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum ServerMode {
    #[default]
    Stdio,
    Http,
}

#[derive(Subcommand)]
enum Commands {
    // Canonical Tier-A command markers for parity guardrails:
    // "init" "info" "query" "export" "ground" "evidence" "suggest"
    // workspace_add workspace_query workspace_xref
    Goal(GoalCommand),
    Decision(DecisionCommand),
    Commitment(CommitmentCommand),
    Progress(ProgressCommand),
    Singularity(SingularityCommand),
    Dream(DreamCommand),
    Counterfactual(CounterfactualCommand),
    Chain(ChainCommand),
    Metamorphosis(MetamorphosisCommand),
    Consensus(ConsensusCommand),
    Federation(FederationCommand),
    Workspace(WorkspaceCommand),
    Daemon(DaemonCommand),
    ContextLog {
        /// Why you are performing this planning action
        intent: String,
        /// What you found or concluded
        #[arg(long)]
        finding: Option<String>,
        /// Topic or category
        #[arg(long)]
        topic: Option<String>,
    },
    Status,
    Serve {
        #[arg(long, default_value = "stdio")]
        mode: ServerMode,
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
    Version,
}

#[derive(Args)]
struct GoalCommand {
    #[command(subcommand)]
    command: GoalSubcommand,
}

#[derive(Subcommand)]
enum GoalSubcommand {
    Create {
        title: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        intention: Option<String>,
        #[arg(short, long)]
        priority: Option<String>,
        #[arg(long)]
        deadline: Option<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        tags: Option<String>,
        #[arg(long)]
        activate: bool,
    },
    List {
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        priority: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(long)]
        active: bool,
        #[arg(long)]
        blocked: bool,
        #[arg(long)]
        urgent: bool,
        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },
    Show {
        id: String,
        #[arg(long)]
        full: bool,
    },
    Activate {
        id: String,
    },
    Progress {
        id: String,
        percentage: f64,
        #[arg(short, long)]
        note: Option<String>,
    },
    Complete {
        id: String,
        #[arg(short, long)]
        note: Option<String>,
    },
    Abandon {
        id: String,
        reason: String,
    },
    Pause {
        id: String,
        #[arg(short, long)]
        reason: Option<String>,
    },
    Resume {
        id: String,
    },
    Block {
        id: String,
        blocker: String,
        #[arg(short, long)]
        severity: Option<f64>,
    },
    Unblock {
        id: String,
        blocker_id: String,
        resolution: String,
    },
    Decompose {
        id: String,
        #[arg(long)]
        sub_goals: String,
    },
    Link {
        goal_a: String,
        goal_b: String,
        #[arg(short, long, default_value = "alliance")]
        relationship: String,
    },
    Tree {
        id: Option<String>,
    },
    Feelings {
        id: String,
    },
    Physics {
        id: String,
    },
    Dream {
        id: String,
    },
    Reincarnate {
        id: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(long)]
        lessons: Option<String>,
    },
}

#[derive(Args)]
struct DecisionCommand {
    #[command(subcommand)]
    command: DecisionSubcommand,
}

#[derive(Subcommand)]
enum DecisionSubcommand {
    Create {
        question: String,
        #[arg(short, long)]
        context: Option<String>,
    },
    Option {
        id: String,
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(long)]
        pros: Option<String>,
        #[arg(long)]
        cons: Option<String>,
    },
    Crystallize {
        id: String,
        chosen: String,
        #[arg(short, long)]
        reasoning: Option<String>,
    },
    Show {
        id: String,
    },
    Shadows {
        id: String,
    },
    Chain {
        id: String,
    },
    Archaeology {
        artifact: String,
    },
    Prophecy {
        question: String,
        #[arg(long)]
        options: Option<String>,
    },
    Regret {
        id: String,
    },
    Recrystallize {
        id: String,
        chosen: String,
        reason: String,
    },
}

#[derive(Args)]
struct CommitmentCommand {
    #[command(subcommand)]
    command: CommitmentSubcommand,
}

#[derive(Subcommand)]
enum CommitmentSubcommand {
    Create {
        promise: String,
        stakeholder: String,
        #[arg(long)]
        due: Option<String>,
        #[arg(long)]
        goal: Option<String>,
        #[arg(long, default_value = "stakeholder")]
        role: String,
        #[arg(long, default_value_t = 0.6)]
        importance: f64,
    },
    List,
    Show {
        id: String,
    },
    Fulfill {
        id: String,
        how_delivered: String,
    },
    Break {
        id: String,
        reason: String,
    },
    Renegotiate {
        id: String,
        new_promise: String,
        reason: String,
    },
    Entangle {
        id: String,
        commitment_b: String,
        #[arg(long, default_value = "parallel")]
        entanglement_type: String,
        #[arg(long, default_value_t = 0.8)]
        strength: f64,
    },
    Inventory,
    DueSoon {
        #[arg(long, default_value = "7")]
        within_days: f64,
    },
    AtRisk,
}

#[derive(Args)]
struct ProgressCommand {
    #[command(subcommand)]
    command: ProgressSubcommand,
}

#[derive(Subcommand)]
enum ProgressSubcommand {
    Momentum {
        #[arg(long)]
        goal_id: Option<String>,
    },
    Gravity,
    Blockers,
    Echoes,
    Forecast {
        goal_id: String,
    },
    Velocity,
    Trend,
}

#[derive(Args)]
struct SingularityCommand {
    #[command(subcommand)]
    command: SingularitySubcommand,
}

#[derive(Subcommand)]
enum SingularitySubcommand {
    Collapse,
    Position { goal_id: String },
    Path,
    Tensions,
    Themes,
    Center,
    Vision,
}

#[derive(Args)]
struct DreamCommand {
    #[command(subcommand)]
    command: DreamSubcommand,
}

#[derive(Subcommand)]
enum DreamSubcommand {
    Goal {
        id: String,
    },
    Collective {
        #[arg(long)]
        federation_id: Option<String>,
    },
    Interpret {
        dream_id: String,
    },
    Insights {
        #[arg(long)]
        goal_id: Option<String>,
    },
    Accuracy {
        dream_id: String,
    },
    History {
        #[arg(long)]
        goal_id: Option<String>,
    },
}

#[derive(Args)]
struct CounterfactualCommand {
    #[command(subcommand)]
    command: CounterfactualSubcommand,
}

#[derive(Subcommand)]
enum CounterfactualSubcommand {
    Project {
        decision_id: String,
        path_id: String,
    },
    Compare {
        decision_id: String,
        path_a: String,
        path_b: String,
    },
    Learn {
        decision_id: String,
    },
    Timeline {
        decision_id: String,
        #[arg(long)]
        path_id: Option<String>,
    },
}

#[derive(Args)]
struct ChainCommand {
    #[command(subcommand)]
    command: ChainSubcommand,
}

#[derive(Subcommand)]
enum ChainSubcommand {
    Trace { decision_id: String },
    Cascade { decision_id: String },
    Roots { decision_id: String },
    Leaves { decision_id: String },
    Visualize { decision_id: String },
}

#[derive(Args)]
struct MetamorphosisCommand {
    #[command(subcommand)]
    command: MetamorphosisSubcommand,
}

#[derive(Subcommand)]
enum MetamorphosisSubcommand {
    Detect {
        goal_id: String,
    },
    Approve {
        goal_id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        change_type: String,
        #[arg(long)]
        factor: Option<f64>,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        new_direction: Option<String>,
        #[arg(long)]
        clarification: Option<String>,
    },
    History {
        goal_id: String,
    },
    Predict {
        goal_id: String,
    },
    Stage {
        goal_id: String,
    },
}

#[derive(Args)]
struct ConsensusCommand {
    #[command(subcommand)]
    command: ConsensusSubcommand,
}

#[derive(Subcommand)]
enum ConsensusSubcommand {
    Start {
        decision_id: String,
        #[arg(long)]
        stakeholders: Option<String>,
    },
    Round {
        decision_id: String,
        stakeholder: String,
        position: String,
    },
    Vote {
        decision_id: String,
        stakeholder: String,
        option: String,
    },
    Synthesize {
        decision_id: String,
    },
    Status {
        decision_id: String,
    },
    Crystallize {
        decision_id: String,
        #[arg(long)]
        option: Option<String>,
        #[arg(long)]
        rationale: Option<String>,
    },
}

#[derive(Args)]
struct FederationCommand {
    #[command(subcommand)]
    command: FederationSubcommand,
}

#[derive(Subcommand)]
enum FederationSubcommand {
    Create {
        goal_id: String,
        agent_id: String,
        #[arg(long)]
        coordinator: Option<String>,
    },
    Join {
        federation_id: String,
        agent_id: String,
    },
    Sync {
        federation_id: String,
    },
    Handoff {
        federation_id: String,
        agent_id: String,
    },
    Status {
        federation_id: String,
    },
    Members {
        federation_id: String,
    },
    List,
}

#[derive(Args)]
struct WorkspaceCommand {
    #[command(subcommand)]
    command: WorkspaceSubcommand,
}

#[derive(Subcommand)]
enum WorkspaceSubcommand {
    Create {
        name: String,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    Switch {
        name: String,
    },
    List,
    Compare {
        left: String,
        right: String,
    },
    Merge {
        source: String,
        target: String,
    },
    Delete {
        name: String,
    },
}

#[derive(Args)]
struct DaemonCommand {
    #[command(subcommand)]
    command: DaemonSubcommand,
}

#[derive(Subcommand)]
enum DaemonSubcommand {
    Start {
        #[arg(long, default_value_t = 60)]
        interval_secs: u64,
    },
    Stop,
    Status,
    Logs {
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceEntry {
    name: String,
    file: String,
    created_at: i64,
    last_used_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WorkspaceRegistry {
    active: Option<String>,
    workspaces: Vec<WorkspaceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DaemonState {
    running: bool,
    started_at: Option<i64>,
    stopped_at: Option<i64>,
    interval_secs: u64,
    last_heartbeat: i64,
    log_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusRoundState {
    round: u32,
    stakeholder: String,
    position: String,
    recorded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusSessionState {
    decision_id: String,
    stakeholders: Vec<String>,
    rounds: Vec<ConsensusRoundState>,
    votes: std::collections::HashMap<String, String>,
    status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ConsensusRegistry {
    sessions: Vec<ConsensusSessionState>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let state_root = state_root(cli.file.as_deref())?;

    let mut engine = if let Some(path) = &cli.file {
        PlanningEngine::open(path)
            .map_err(|e| format!("failed to open planning file {}: {e}", path.display()))?
    } else {
        PlanningEngine::in_memory()
    };

    let mut mutated = false;

    let output: Value = match cli.command {
        Commands::Goal(goal) => handle_goal(goal.command, &mut engine, &mut mutated)?,
        Commands::Decision(decision) => {
            handle_decision(decision.command, &mut engine, &mut mutated)?
        }
        Commands::Commitment(commitment) => {
            handle_commitment(commitment.command, &mut engine, &mut mutated)?
        }
        Commands::Progress(progress) => handle_progress(progress.command, &engine)?,
        Commands::Singularity(singularity) => handle_singularity(singularity.command, &engine)?,
        Commands::Dream(dream) => handle_dream(dream.command, &mut engine, &mut mutated)?,
        Commands::Counterfactual(counterfactual) => {
            handle_counterfactual(counterfactual.command, &engine)?
        }
        Commands::Chain(chain) => handle_chain(chain.command, &engine)?,
        Commands::Metamorphosis(metamorphosis) => {
            handle_metamorphosis(metamorphosis.command, &mut engine, &mut mutated)?
        }
        Commands::Consensus(consensus) => {
            handle_consensus(consensus.command, &mut engine, &mut mutated, &state_root)?
        }
        Commands::Federation(federation) => {
            handle_federation(federation.command, &mut engine, &mut mutated)?
        }
        Commands::Workspace(workspace) => handle_workspace(workspace.command, &state_root)?,
        Commands::Daemon(daemon) => handle_daemon(daemon.command, &state_root)?,
        Commands::ContextLog {
            intent,
            finding,
            topic,
        } => {
            let mut server = PlanningMcpServer::new(engine.clone());
            let params = json!({
                "intent": intent,
                "finding": finding,
                "topic": topic,
            });
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("runtime error: {e}"))?
                .block_on(server.handle_tool("planning_context_log", params))
                .map_err(|e| format!("context log error: {e}"))?;
            result
        }
        Commands::Status => {
            let inv = engine.get_commitment_inventory();
            json!({
                "active_goals": engine.get_active_goals().len(),
                "blocked_goals": engine.get_blocked_goals().len(),
                "active_commitments": inv.active_count,
                "total_commitments": inv.total_count,
                "is_overloaded": inv.is_overloaded,
                "total_weight": inv.total_weight,
            })
        }
        Commands::Serve { mode, port } => match mode {
            ServerMode::Stdio => {
                let mut server = PlanningMcpServer::new(engine.clone());
                let stdin = std::io::stdin();
                let stdout = std::io::stdout();
                let mut reader = BufReader::new(stdin.lock());
                let mut writer = stdout.lock();
                run_stdio_loop(&mut reader, &mut writer, &mut server);
                return Ok(());
            }
            ServerMode::Http => {
                return Err(format!(
                        "http mode requested on port {port}, but this CLI currently supports stdio-only runtime bootstrapping"
                    ));
            }
        },
        Commands::Version => json!({ "version": env!("CARGO_PKG_VERSION") }),
    };

    if mutated {
        engine
            .save()
            .map_err(|e| format!("failed to persist planning state: {e}"))?;
    }

    emit_output(&output, cli.format, cli.json);
    Ok(())
}

fn handle_goal(
    cmd: GoalSubcommand,
    engine: &mut PlanningEngine,
    mutated: &mut bool,
) -> CliResult<Value> {
    match cmd {
        GoalSubcommand::Create {
            title,
            description,
            intention,
            priority,
            deadline,
            parent,
            tags,
            activate,
        } => {
            let request = CreateGoalRequest {
                title: title.clone(),
                description: description.unwrap_or_default(),
                intention: intention.unwrap_or(title),
                priority: priority.as_deref().map(parse_priority).transpose()?,
                deadline: deadline.as_deref().map(parse_deadline).transpose()?,
                parent: parent.as_deref().map(parse_goal_id).transpose()?,
                tags: tags.map(|v| csv_values(&v)),
                ..Default::default()
            };
            let mut goal = engine.create_goal(request).map_err(|e| e.to_string())?;
            if activate {
                goal = engine.activate_goal(goal.id).map_err(|e| e.to_string())?;
            }
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::List {
            status,
            priority,
            tag,
            active,
            blocked,
            urgent,
            limit,
        } => {
            if urgent {
                let mut goals: Vec<_> = engine.get_urgent_goals(7.0).into_iter().cloned().collect();
                if let Some(n) = limit {
                    goals.truncate(n);
                }
                return Ok(json!(goals));
            }

            let status_filter = if active {
                Some(vec![GoalStatus::Active])
            } else if blocked {
                Some(vec![GoalStatus::Blocked])
            } else {
                status.as_deref().map(parse_goal_status_list).transpose()?
            };

            let priority_filter = priority.as_deref().map(parse_priority_list).transpose()?;

            let filter = GoalFilter {
                status: status_filter,
                priority: priority_filter,
                tags: tag.map(|t| vec![t]),
                limit,
                ..Default::default()
            };
            let goals: Vec<_> = engine.list_goals(filter).into_iter().cloned().collect();
            Ok(json!(goals))
        }
        GoalSubcommand::Show { id, full } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine
                .get_goal(goal_id)
                .ok_or_else(|| format!("goal not found: {id}"))?;
            if full {
                Ok(json!(goal))
            } else {
                Ok(json!({
                    "id": goal.id,
                    "title": goal.title,
                    "status": goal.status,
                    "progress": goal.progress.percentage,
                    "priority": goal.priority,
                    "deadline": goal.deadline,
                    "blockers": goal.blockers.len()
                }))
            }
        }
        GoalSubcommand::Activate { id } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine.activate_goal(goal_id).map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Progress {
            id,
            percentage,
            note,
        } => {
            let goal_id = parse_goal_id(&id)?;
            let normalized = (percentage / 100.0).clamp(0.0, 1.0);
            let goal = engine
                .progress_goal(goal_id, normalized, note)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Complete { id, note } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine
                .complete_goal(goal_id, note)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Abandon { id, reason } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine
                .abandon_goal(goal_id, reason)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Pause { id, reason } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine
                .pause_goal(goal_id, reason)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Resume { id } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine.resume_goal(goal_id).map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Block {
            id,
            blocker,
            severity,
        } => {
            let goal_id = parse_goal_id(&id)?;
            let blocker = Blocker {
                id: Uuid::new_v4(),
                blocker_type: BlockerType::Unknown {
                    signals: vec!["cli".to_string()],
                },
                description: blocker,
                severity: severity.unwrap_or(0.5).clamp(0.0, 1.0),
                identified_at: Timestamp::now(),
                resolved_at: None,
                resolution: None,
            };
            let goal = engine
                .block_goal(goal_id, blocker)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Unblock {
            id,
            blocker_id,
            resolution,
        } => {
            let goal_id = parse_goal_id(&id)?;
            let blocker_uuid =
                Uuid::parse_str(&blocker_id).map_err(|e| format!("invalid blocker_id: {e}"))?;
            let goal = engine
                .unblock_goal(goal_id, blocker_uuid, resolution)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        GoalSubcommand::Decompose { id, sub_goals } => {
            let parent_id = parse_goal_id(&id)?;
            let requests: Vec<CreateGoalRequest> = csv_values(&sub_goals)
                .into_iter()
                .map(|title| CreateGoalRequest {
                    title: title.clone(),
                    intention: format!("Sub-goal: {title}"),
                    ..Default::default()
                })
                .collect();
            let created = engine
                .decompose_goal(parent_id, requests)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(created))
        }
        GoalSubcommand::Link {
            goal_a,
            goal_b,
            relationship,
        } => {
            let a = parse_goal_id(&goal_a)?;
            let b = parse_goal_id(&goal_b)?;
            let rel = if relationship.eq_ignore_ascii_case("dependency") {
                GoalRelationship::Dependency {
                    dependent: a,
                    on: b,
                    strength: 1.0,
                }
            } else {
                GoalRelationship::Alliance {
                    goals: (a, b),
                    synergy: 0.7,
                }
            };
            engine.link_goals(rel).map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!({"status": "linked", "goal_a": a, "goal_b": b}))
        }
        GoalSubcommand::Tree { id } => {
            if let Some(id) = id {
                let root = parse_goal_id(&id)?;
                let tree = engine
                    .get_goal_tree(root)
                    .unwrap_or(agentic_planning::GoalTree {
                        root,
                        nodes: std::collections::HashMap::new(),
                        edges: Vec::new(),
                    });
                Ok(json!(tree))
            } else {
                let roots: Vec<_> = engine.get_root_goals().into_iter().cloned().collect();
                Ok(json!(roots))
            }
        }
        GoalSubcommand::Feelings { id } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine
                .get_goal(goal_id)
                .ok_or_else(|| format!("goal not found: {id}"))?;
            Ok(json!(goal.feelings))
        }
        GoalSubcommand::Physics { id } => {
            let goal_id = parse_goal_id(&id)?;
            let goal = engine
                .get_goal(goal_id)
                .ok_or_else(|| format!("goal not found: {id}"))?;
            Ok(json!(goal.physics))
        }
        GoalSubcommand::Dream { id } => {
            let goal_id = parse_goal_id(&id)?;
            let dream = engine.dream_goal(goal_id).map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(dream))
        }
        GoalSubcommand::Reincarnate { id, title, lessons } => {
            let goal_id = parse_goal_id(&id)?;
            let updates = ReincarnationUpdates {
                title,
                description: None,
                lessons_learned: lessons.map(|v| csv_values(&v)),
            };
            let goal = engine
                .reincarnate_goal(goal_id, updates)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
    }
}

fn handle_decision(
    cmd: DecisionSubcommand,
    engine: &mut PlanningEngine,
    mutated: &mut bool,
) -> CliResult<Value> {
    match cmd {
        DecisionSubcommand::Create { question, context } => {
            let decision = engine
                .create_decision(CreateDecisionRequest {
                    question,
                    context,
                    ..Default::default()
                })
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(decision))
        }
        DecisionSubcommand::Option {
            id,
            name,
            description,
            pros,
            cons,
        } => {
            let decision_id = parse_decision_id(&id)?;
            let decision = engine
                .add_option(
                    decision_id,
                    DecisionPath {
                        id: PathId(Uuid::new_v4()),
                        name,
                        description: description.unwrap_or_default(),
                        pros: pros.map(|v| csv_values(&v)).unwrap_or_default(),
                        cons: cons.map(|v| csv_values(&v)).unwrap_or_default(),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(decision))
        }
        DecisionSubcommand::Crystallize {
            id,
            chosen,
            reasoning,
        } => {
            let decision_id = parse_decision_id(&id)?;
            let decision = engine
                .get_decision(decision_id)
                .ok_or_else(|| format!("decision not found: {id}"))?
                .clone();

            let path_id = decision
                .shadows
                .iter()
                .find(|s| s.path.name == chosen || s.path.id.0.to_string().starts_with(&chosen))
                .map(|s| s.path.id)
                .ok_or_else(|| format!("no matching path for '{chosen}'"))?;

            let out = engine
                .crystallize(
                    decision_id,
                    path_id,
                    DecisionReasoning {
                        rationale: reasoning.unwrap_or_else(|| "CLI user choice".to_string()),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(out))
        }
        DecisionSubcommand::Show { id } => {
            let decision_id = parse_decision_id(&id)?;
            let decision = engine
                .get_decision(decision_id)
                .ok_or_else(|| format!("decision not found: {id}"))?;
            Ok(json!(decision))
        }
        DecisionSubcommand::Shadows { id } => {
            let decision_id = parse_decision_id(&id)?;
            let shadows: Vec<_> = engine
                .get_shadows(decision_id)
                .into_iter()
                .cloned()
                .collect();
            Ok(json!(shadows))
        }
        DecisionSubcommand::Chain { id } => {
            let decision_id = parse_decision_id(&id)?;
            let chain = engine
                .get_decision_chain(decision_id)
                .ok_or_else(|| format!("decision not found: {id}"))?;
            Ok(json!(chain))
        }
        DecisionSubcommand::Archaeology { artifact } => {
            Ok(json!(engine.decision_archaeology(&artifact)))
        }
        DecisionSubcommand::Prophecy { question, options } => {
            let option_paths: Vec<DecisionPath> = options
                .map(|v| {
                    csv_values(&v)
                        .into_iter()
                        .map(|name| DecisionPath {
                            name,
                            ..Default::default()
                        })
                        .collect()
                })
                .unwrap_or_default();
            Ok(json!(engine.get_decision_prophecy(&question, &option_paths)))
        }
        DecisionSubcommand::Regret { id } => {
            let decision_id = parse_decision_id(&id)?;
            let decision = engine
                .get_decision(decision_id)
                .ok_or_else(|| format!("decision not found: {id}"))?;
            Ok(json!({"decision_id": decision_id, "regret": decision.regret_score}))
        }
        DecisionSubcommand::Recrystallize { id, chosen, reason } => {
            let decision_id = parse_decision_id(&id)?;
            let decision = engine
                .get_decision(decision_id)
                .ok_or_else(|| format!("decision not found: {id}"))?
                .clone();

            let path_id = decision
                .shadows
                .iter()
                .find(|s| s.path.name == chosen || s.path.id.0.to_string().starts_with(&chosen))
                .map(|s| s.path.id)
                .ok_or_else(|| format!("no matching path for '{chosen}'"))?;

            let updated = engine
                .recrystallize(decision_id, path_id, reason)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(updated))
        }
    }
}

fn handle_commitment(
    cmd: CommitmentSubcommand,
    engine: &mut PlanningEngine,
    mutated: &mut bool,
) -> CliResult<Value> {
    match cmd {
        CommitmentSubcommand::Create {
            promise,
            stakeholder,
            due,
            goal,
            role,
            importance,
        } => {
            let commitment = engine
                .create_commitment(CreateCommitmentRequest {
                    promise: Promise {
                        description: promise,
                        ..Default::default()
                    },
                    stakeholder: Stakeholder {
                        id: StakeholderId(Uuid::new_v4()),
                        name: stakeholder,
                        role,
                        importance,
                    },
                    due: due.as_deref().map(parse_deadline).transpose()?,
                    goal: goal.as_deref().map(parse_goal_id).transpose()?,
                })
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(commitment))
        }
        CommitmentSubcommand::List => {
            let list: Vec<_> = engine.list_commitments().into_iter().cloned().collect();
            Ok(json!(list))
        }
        CommitmentSubcommand::Show { id } => {
            let commitment_id = parse_commitment_id(&id)?;
            let commitment = engine
                .get_commitment(commitment_id)
                .ok_or_else(|| format!("commitment not found: {id}"))?;
            Ok(json!(commitment))
        }
        CommitmentSubcommand::Fulfill { id, how_delivered } => {
            let commitment_id = parse_commitment_id(&id)?;
            let updated = engine
                .fulfill_commitment(commitment_id, how_delivered)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(updated))
        }
        CommitmentSubcommand::Break { id, reason } => {
            let commitment_id = parse_commitment_id(&id)?;
            let updated = engine
                .break_commitment(commitment_id, reason)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(updated))
        }
        CommitmentSubcommand::Renegotiate {
            id,
            new_promise,
            reason,
        } => {
            let commitment_id = parse_commitment_id(&id)?;
            let updated = engine
                .renegotiate_commitment(
                    commitment_id,
                    Promise {
                        description: new_promise,
                        ..Default::default()
                    },
                    reason,
                )
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(updated))
        }
        CommitmentSubcommand::Entangle {
            id,
            commitment_b,
            entanglement_type,
            strength,
        } => {
            let a = parse_commitment_id(&id)?;
            let b = parse_commitment_id(&commitment_b)?;
            let entanglement = parse_entanglement_type(&entanglement_type)?;
            engine
                .entangle_commitments(a, b, entanglement, strength)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!({"status": "entangled", "a": a, "b": b}))
        }
        CommitmentSubcommand::Inventory => Ok(json!(engine.get_commitment_inventory())),
        CommitmentSubcommand::DueSoon { within_days } => {
            let due: Vec<_> = engine
                .get_due_soon(within_days)
                .into_iter()
                .cloned()
                .collect();
            Ok(json!(due))
        }
        CommitmentSubcommand::AtRisk => {
            let at_risk: Vec<_> = engine
                .get_at_risk_commitments()
                .into_iter()
                .cloned()
                .collect();
            Ok(json!(at_risk))
        }
    }
}

fn handle_progress(cmd: ProgressSubcommand, engine: &PlanningEngine) -> CliResult<Value> {
    match cmd {
        ProgressSubcommand::Momentum { goal_id } => {
            if let Some(goal_id) = goal_id {
                let id = parse_goal_id(&goal_id)?;
                let goal = engine
                    .get_goal(id)
                    .ok_or_else(|| format!("goal not found: {goal_id}"))?;
                Ok(json!({
                    "goal_id": id,
                    "title": goal.title,
                    "momentum": goal.physics.momentum,
                    "trend": if goal.physics.momentum > 0.5 { "up" } else { "down" }
                }))
            } else {
                let report: Vec<_> = engine
                    .get_active_goals()
                    .into_iter()
                    .map(|g| {
                        json!({"goal_id": g.id, "title": g.title, "momentum": g.physics.momentum})
                    })
                    .collect();
                Ok(json!(report))
            }
        }
        ProgressSubcommand::Gravity => {
            let report: Vec<_> = engine
                .get_active_goals()
                .into_iter()
                .map(|g| json!({"goal_id": g.id, "title": g.title, "gravity": g.physics.gravity}))
                .collect();
            Ok(json!(report))
        }
        ProgressSubcommand::Blockers => Ok(json!(engine.scan_blocker_prophecy())),
        ProgressSubcommand::Echoes => Ok(json!(engine.listen_progress_echoes())),
        ProgressSubcommand::Forecast { goal_id } => {
            let id = parse_goal_id(&goal_id)?;
            let goal = engine
                .get_goal(id)
                .ok_or_else(|| format!("goal not found: {goal_id}"))?;
            Ok(json!({"goal_id": id, "eta": goal.progress.eta, "velocity": goal.progress.velocity}))
        }
        ProgressSubcommand::Velocity => {
            let report: Vec<_> = engine
                .get_active_goals()
                .into_iter()
                .map(
                    |g| json!({"goal_id": g.id, "title": g.title, "velocity": g.progress.velocity}),
                )
                .collect();
            Ok(json!(report))
        }
        ProgressSubcommand::Trend => {
            let report: Vec<_> = engine
                .get_active_goals()
                .into_iter()
                .map(|g| {
                    let trend = if g.physics.momentum > 0.5 {
                        "accelerating"
                    } else {
                        "flat"
                    };
                    json!({"goal_id": g.id, "title": g.title, "trend": trend})
                })
                .collect();
            Ok(json!(report))
        }
    }
}

fn handle_singularity(cmd: SingularitySubcommand, engine: &PlanningEngine) -> CliResult<Value> {
    let singularity = engine.get_intention_singularity();
    match cmd {
        SingularitySubcommand::Collapse => Ok(json!(singularity)),
        SingularitySubcommand::Position { goal_id } => {
            let id = parse_goal_id(&goal_id)?;
            let position = singularity
                .goal_positions
                .get(&id)
                .ok_or_else(|| format!("goal has no singularity position: {goal_id}"))?;
            Ok(json!(position))
        }
        SingularitySubcommand::Path => Ok(json!(singularity.golden_path)),
        SingularitySubcommand::Tensions => Ok(json!(singularity.tension_lines)),
        SingularitySubcommand::Themes => Ok(json!(singularity.themes)),
        SingularitySubcommand::Center => Ok(json!(singularity.center)),
        SingularitySubcommand::Vision => Ok(json!({"vision": singularity.unified_vision})),
    }
}

fn handle_dream(
    cmd: DreamSubcommand,
    engine: &mut PlanningEngine,
    mutated: &mut bool,
) -> CliResult<Value> {
    match cmd {
        DreamSubcommand::Goal { id } => {
            let goal_id = parse_goal_id(&id)?;
            let dream = engine.dream_goal(goal_id).map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(dream))
        }
        DreamSubcommand::Collective { federation_id } => {
            let targets: Vec<GoalId> = if let Some(federation_id) = federation_id {
                let federation_id = parse_federation_id(&federation_id)?;
                let federation = engine
                    .get_federation(federation_id)
                    .ok_or_else(|| format!("federation not found: {}", federation_id.0))?;
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
                engine
                    .get_active_goals()
                    .into_iter()
                    .map(|g| g.id)
                    .collect()
            };

            let mut dream_ids = Vec::new();
            for goal_id in targets {
                if engine.get_goal(goal_id).is_none() {
                    return Err(format!("goal not found: {}", goal_id.0));
                }
                let dream = engine.dream_goal(goal_id).map_err(|e| e.to_string())?;
                dream_ids.push(dream.id);
            }
            *mutated = true;
            Ok(json!({"dreams_created": dream_ids.len(), "dream_ids": dream_ids}))
        }
        DreamSubcommand::Interpret { dream_id } => {
            let dream_id = parse_dream_id(&dream_id)?;
            let dream = engine
                .get_dream(dream_id)
                .ok_or_else(|| format!("dream not found: {}", dream_id.0))?;
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
        DreamSubcommand::Insights { goal_id } => {
            if let Some(goal_id) = goal_id {
                let goal_id = parse_goal_id(&goal_id)?;
                let insights: Vec<_> = engine
                    .list_goal_dreams(goal_id)
                    .into_iter()
                    .flat_map(|d| d.insights.clone())
                    .collect();
                Ok(json!(insights))
            } else {
                let insights: Vec<_> = engine
                    .list_dreams()
                    .into_iter()
                    .flat_map(|d| d.insights.clone())
                    .collect();
                Ok(json!(insights))
            }
        }
        DreamSubcommand::Accuracy { dream_id } => {
            let dream_id = parse_dream_id(&dream_id)?;
            let dream = engine
                .get_dream(dream_id)
                .ok_or_else(|| format!("dream not found: {}", dream_id.0))?;
            let completion_score = engine
                .get_goal(dream.goal_id)
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
        DreamSubcommand::History { goal_id } => {
            if let Some(goal_id) = goal_id {
                let goal_id = parse_goal_id(&goal_id)?;
                let dreams: Vec<_> = engine
                    .list_goal_dreams(goal_id)
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(json!(dreams))
            } else {
                let mut dreams: Vec<_> = engine.list_dreams().into_iter().cloned().collect();
                dreams.sort_by_key(|d| d.dreamt_at);
                Ok(json!(dreams))
            }
        }
    }
}

fn handle_counterfactual(
    cmd: CounterfactualSubcommand,
    engine: &PlanningEngine,
) -> CliResult<Value> {
    match cmd {
        CounterfactualSubcommand::Project {
            decision_id,
            path_id,
        } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let path_id = parse_path_id(&path_id)?;
            let projection = engine
                .project_counterfactual(decision_id, path_id)
                .ok_or_else(|| format!("path not found: {}", path_id.0))?;
            Ok(json!(projection))
        }
        CounterfactualSubcommand::Compare {
            decision_id,
            path_a,
            path_b,
        } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let path_a = parse_path_id(&path_a)?;
            let path_b = parse_path_id(&path_b)?;
            let a = engine
                .project_counterfactual(decision_id, path_a)
                .ok_or_else(|| format!("path not found: {}", path_a.0))?;
            let b = engine
                .project_counterfactual(decision_id, path_b)
                .ok_or_else(|| format!("path not found: {}", path_b.0))?;
            Ok(json!({"decision_id": decision_id, "path_a": a, "path_b": b}))
        }
        CounterfactualSubcommand::Learn { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let decision = engine
                .get_decision(decision_id)
                .ok_or_else(|| format!("decision not found: {}", decision_id.0))?;
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
        CounterfactualSubcommand::Timeline {
            decision_id,
            path_id,
        } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let decision = engine
                .get_decision(decision_id)
                .ok_or_else(|| format!("decision not found: {}", decision_id.0))?
                .clone();

            if let Some(path_id) = path_id {
                let path_id = parse_path_id(&path_id)?;
                let projection = engine
                    .project_counterfactual(decision_id, path_id)
                    .ok_or_else(|| format!("path not found: {}", path_id.0))?;
                return Ok(json!(projection));
            }

            let timelines: Vec<_> = decision
                .shadows
                .iter()
                .filter_map(|s| {
                    engine.project_counterfactual(decision_id, s.path.id).map(
                        |p| json!({"path_id": s.path.id, "path_name": s.path.name, "projection": p}),
                    )
                })
                .collect();
            Ok(json!(timelines))
        }
    }
}

fn handle_chain(cmd: ChainSubcommand, engine: &PlanningEngine) -> CliResult<Value> {
    match cmd {
        ChainSubcommand::Trace { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            Ok(json!(engine.get_decision_chain(decision_id)))
        }
        ChainSubcommand::Cascade { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            if let Some(chain) = engine.get_decision_chain(decision_id) {
                Ok(json!({
                    "root": chain.root,
                    "total_nodes": chain.cascade_analysis.total_nodes,
                    "edges": chain.causality.len()
                }))
            } else {
                Ok(json!(null))
            }
        }
        ChainSubcommand::Roots { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let chain = engine.get_decision_chain(decision_id);
            Ok(json!({"decision_id": decision_id, "root": chain.map(|c| c.root)}))
        }
        ChainSubcommand::Leaves { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            if let Some(chain) = engine.get_decision_chain(decision_id) {
                let leaves: Vec<_> = chain
                    .descendants
                    .iter()
                    .copied()
                    .filter(|id| {
                        engine
                            .get_decision(*id)
                            .map(|d| d.causes.is_empty())
                            .unwrap_or(false)
                    })
                    .collect();
                Ok(json!(leaves))
            } else {
                Ok(json!([]))
            }
        }
        ChainSubcommand::Visualize { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            if let Some(chain) = engine.get_decision_chain(decision_id) {
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
    }
}

fn handle_metamorphosis(
    cmd: MetamorphosisSubcommand,
    engine: &mut PlanningEngine,
    mutated: &mut bool,
) -> CliResult<Value> {
    match cmd {
        MetamorphosisSubcommand::Detect { goal_id } => {
            let goal_id = parse_goal_id(&goal_id)?;
            Ok(json!(engine
                .detect_metamorphosis(goal_id)
                .map_err(|e| e.to_string())?))
        }
        MetamorphosisSubcommand::Approve {
            goal_id,
            title,
            description,
            change_type,
            factor,
            reason,
            new_direction,
            clarification,
        } => {
            let goal_id = parse_goal_id(&goal_id)?;
            let change =
                parse_scope_change_cli(&change_type, factor, reason, new_direction, clarification)?;
            let goal = engine
                .approve_metamorphosis(
                    goal_id,
                    title.unwrap_or_else(|| "Metamorphosis Approved".to_string()),
                    description.unwrap_or_else(|| "Stage approved from CLI workflow".to_string()),
                    change,
                )
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(goal))
        }
        MetamorphosisSubcommand::History { goal_id } => {
            let goal_id = parse_goal_id(&goal_id)?;
            let history = engine
                .metamorphosis_history(goal_id)
                .map_err(|e| e.to_string())?;
            Ok(json!(history))
        }
        MetamorphosisSubcommand::Predict { goal_id } => {
            let goal_id = parse_goal_id(&goal_id)?;
            Ok(json!(engine
                .predict_metamorphosis(goal_id)
                .map_err(|e| e.to_string())?))
        }
        MetamorphosisSubcommand::Stage { goal_id } => {
            let goal_id = parse_goal_id(&goal_id)?;
            Ok(json!(engine
                .metamorphosis_stage(goal_id)
                .map_err(|e| e.to_string())?))
        }
    }
}

fn handle_consensus(
    cmd: ConsensusSubcommand,
    engine: &mut PlanningEngine,
    mutated: &mut bool,
    root: &Path,
) -> CliResult<Value> {
    let state_path = consensus_state_path(root);
    let mut registry = load_consensus_registry(&state_path)?;

    match cmd {
        ConsensusSubcommand::Start {
            decision_id,
            stakeholders,
        } => {
            let decision_id = parse_decision_id(&decision_id)?;
            if engine.get_decision(decision_id).is_none() {
                return Err(format!("decision not found: {}", decision_id.0));
            }

            let stakeholders = stakeholders.map(|s| csv_values(&s)).unwrap_or_default();
            let id_s = decision_id.0.to_string();
            if let Some(existing) = registry.sessions.iter_mut().find(|s| s.decision_id == id_s) {
                existing.stakeholders = stakeholders;
                existing.rounds.clear();
                existing.votes.clear();
                existing.status = "open".to_string();
                let snapshot = existing.clone();
                save_consensus_registry(&state_path, &registry)?;
                return Ok(json!(snapshot));
            }

            let session = ConsensusSessionState {
                decision_id: id_s,
                stakeholders,
                rounds: Vec::new(),
                votes: std::collections::HashMap::new(),
                status: "open".to_string(),
            };
            registry.sessions.push(session.clone());
            save_consensus_registry(&state_path, &registry)?;
            Ok(json!(session))
        }
        ConsensusSubcommand::Round {
            decision_id,
            stakeholder,
            position,
        } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let session = get_consensus_session_mut(&mut registry, decision_id)?;
            let round = session.rounds.len() as u32 + 1;
            session.rounds.push(ConsensusRoundState {
                round,
                stakeholder: stakeholder.clone(),
                position,
                recorded_at: Timestamp::now().as_nanos(),
            });
            if !session.stakeholders.contains(&stakeholder) {
                session.stakeholders.push(stakeholder);
            }
            let snapshot = session.clone();
            save_consensus_registry(&state_path, &registry)?;
            Ok(json!(snapshot))
        }
        ConsensusSubcommand::Vote {
            decision_id,
            stakeholder,
            option,
        } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let session = get_consensus_session_mut(&mut registry, decision_id)?;
            session.votes.insert(stakeholder.clone(), option.clone());
            if !session.stakeholders.contains(&stakeholder) {
                session.stakeholders.push(stakeholder.clone());
            }
            save_consensus_registry(&state_path, &registry)?;
            Ok(json!({
                "decision_id": decision_id,
                "stakeholder": stakeholder,
                "option": option
            }))
        }
        ConsensusSubcommand::Synthesize { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let session = get_consensus_session(&registry, decision_id)?;
            let mut counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for option in session.votes.values() {
                *counts.entry(option.clone()).or_insert(0) += 1;
            }
            let recommendation = counts
                .iter()
                .max_by_key(|(_, c)| *c)
                .map(|(k, _)| k.clone());
            Ok(json!({
                "decision_id": decision_id,
                "votes": session.votes.len(),
                "counts": counts,
                "recommendation": recommendation
            }))
        }
        ConsensusSubcommand::Status { decision_id } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let session = get_consensus_session(&registry, decision_id)?;
            Ok(json!(session))
        }
        ConsensusSubcommand::Crystallize {
            decision_id,
            option,
            rationale,
        } => {
            let decision_id = parse_decision_id(&decision_id)?;
            let session = get_consensus_session_mut(&mut registry, decision_id)?;
            let chosen = if let Some(opt) = option {
                opt
            } else {
                let mut counts: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                for opt in session.votes.values() {
                    *counts.entry(opt.clone()).or_insert(0) += 1;
                }
                counts
                    .into_iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(opt, _)| opt)
                    .ok_or_else(|| {
                        "cannot crystallize without votes; pass --option to force".to_string()
                    })?
            };

            let decision = engine
                .get_decision(decision_id)
                .ok_or_else(|| format!("decision not found: {}", decision_id.0))?
                .clone();
            let path_id = decision
                .shadows
                .iter()
                .find(|s| s.path.name == chosen || s.path.id.0.to_string().starts_with(&chosen))
                .map(|s| s.path.id)
                .ok_or_else(|| format!("no matching decision path for '{chosen}'"))?;

            let decision = engine
                .crystallize(
                    decision_id,
                    path_id,
                    DecisionReasoning {
                        rationale: rationale
                            .unwrap_or_else(|| "Consensus crystallization".to_string()),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;
            session.status = "crystallized".to_string();
            save_consensus_registry(&state_path, &registry)?;
            *mutated = true;
            Ok(json!(decision))
        }
    }
}

fn handle_federation(
    cmd: FederationSubcommand,
    engine: &mut PlanningEngine,
    mutated: &mut bool,
) -> CliResult<Value> {
    match cmd {
        FederationSubcommand::Create {
            goal_id,
            agent_id,
            coordinator,
        } => {
            let goal_id = parse_goal_id(&goal_id)?;
            let federation = engine
                .create_federation(goal_id, agent_id, coordinator)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(federation))
        }
        FederationSubcommand::Join {
            federation_id,
            agent_id,
        } => {
            let federation_id = parse_federation_id(&federation_id)?;
            let federation = engine
                .join_federation(federation_id, agent_id)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(federation))
        }
        FederationSubcommand::Sync { federation_id } => {
            let federation_id = parse_federation_id(&federation_id)?;
            let federation = engine
                .sync_federation(federation_id)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(federation))
        }
        FederationSubcommand::Handoff {
            federation_id,
            agent_id,
        } => {
            let federation_id = parse_federation_id(&federation_id)?;
            let federation = engine
                .handoff_federation(federation_id, agent_id)
                .map_err(|e| e.to_string())?;
            *mutated = true;
            Ok(json!(federation))
        }
        FederationSubcommand::Status { federation_id } => {
            let federation_id = parse_federation_id(&federation_id)?;
            let federation = engine
                .get_federation(federation_id)
                .ok_or_else(|| format!("federation not found: {federation_id:?}"))?;
            Ok(json!(federation))
        }
        FederationSubcommand::Members { federation_id } => {
            let federation_id = parse_federation_id(&federation_id)?;
            let members = engine
                .get_federation_members(federation_id)
                .ok_or_else(|| format!("federation not found: {federation_id:?}"))?;
            Ok(json!(members))
        }
        FederationSubcommand::List => {
            let federations: Vec<_> = engine.list_federations().into_iter().cloned().collect();
            Ok(json!(federations))
        }
    }
}

fn handle_workspace(cmd: WorkspaceSubcommand, root: &Path) -> CliResult<Value> {
    let registry_path = workspace_registry_path(root);
    let mut registry = load_workspace_registry(&registry_path)?;
    let now = Timestamp::now().as_nanos();

    match cmd {
        WorkspaceSubcommand::Create { name, file } => {
            if registry.workspaces.iter().any(|w| w.name == name) {
                return Err(format!("workspace already exists: {name}"));
            }
            let file = file.unwrap_or_else(|| root.join(format!("{name}.aplan")));
            registry.workspaces.push(WorkspaceEntry {
                name: name.clone(),
                file: file.to_string_lossy().to_string(),
                created_at: now,
                last_used_at: now,
            });
            if registry.active.is_none() {
                registry.active = Some(name.clone());
            }
            save_workspace_registry(&registry_path, &registry)?;
            Ok(json!({"status":"created","name":name,"file":file}))
        }
        WorkspaceSubcommand::Switch { name } => {
            let entry = registry
                .workspaces
                .iter_mut()
                .find(|w| w.name == name)
                .ok_or_else(|| format!("workspace not found: {name}"))?;
            entry.last_used_at = now;
            let file = entry.file.clone();
            registry.active = Some(name.clone());
            save_workspace_registry(&registry_path, &registry)?;
            Ok(json!({"status":"switched","active":name,"file":file}))
        }
        WorkspaceSubcommand::List => Ok(json!({
            "active": registry.active,
            "workspaces": registry.workspaces
        })),
        WorkspaceSubcommand::Compare { left, right } => {
            let left_ws = registry
                .workspaces
                .iter()
                .find(|w| w.name == left)
                .ok_or_else(|| format!("workspace not found: {left}"))?;
            let right_ws = registry
                .workspaces
                .iter()
                .find(|w| w.name == right)
                .ok_or_else(|| format!("workspace not found: {right}"))?;

            if !Path::new(&left_ws.file).exists() {
                return Err(format!("workspace file missing: {}", left_ws.file));
            }
            if !Path::new(&right_ws.file).exists() {
                return Err(format!("workspace file missing: {}", right_ws.file));
            }

            let left_engine = PlanningEngine::open(PathBuf::from(&left_ws.file))
                .map_err(|e| format!("cannot open {}: {e}", left_ws.file))?;
            let right_engine = PlanningEngine::open(PathBuf::from(&right_ws.file))
                .map_err(|e| format!("cannot open {}: {e}", right_ws.file))?;

            Ok(json!({
                "left": {
                    "name": left_ws.name,
                    "file": left_ws.file,
                    "goals": left_engine.goal_count(),
                    "decisions": left_engine.decision_count(),
                    "commitments": left_engine.commitment_count()
                },
                "right": {
                    "name": right_ws.name,
                    "file": right_ws.file,
                    "goals": right_engine.goal_count(),
                    "decisions": right_engine.decision_count(),
                    "commitments": right_engine.commitment_count()
                }
            }))
        }
        WorkspaceSubcommand::Merge { source, target } => {
            let source_ws = registry
                .workspaces
                .iter()
                .find(|w| w.name == source)
                .ok_or_else(|| format!("workspace not found: {source}"))?;
            let target_ws = registry
                .workspaces
                .iter()
                .find(|w| w.name == target)
                .ok_or_else(|| format!("workspace not found: {target}"))?;

            if !Path::new(&source_ws.file).exists() {
                return Err(format!("workspace file missing: {}", source_ws.file));
            }
            if !Path::new(&target_ws.file).exists() {
                return Err(format!("workspace file missing: {}", target_ws.file));
            }

            let source_engine = PlanningEngine::open(PathBuf::from(&source_ws.file))
                .map_err(|e| format!("cannot open {}: {e}", source_ws.file))?;
            let mut target_engine = PlanningEngine::open(PathBuf::from(&target_ws.file))
                .map_err(|e| format!("cannot open {}: {e}", target_ws.file))?;

            let report = target_engine.merge_from(&source_engine);
            target_engine
                .save()
                .map_err(|e| format!("failed to persist merge target {}: {e}", target_ws.file))?;

            Ok(json!({
                "status":"merged",
                "source": source,
                "target": target,
                "report": report
            }))
        }
        WorkspaceSubcommand::Delete { name } => {
            let before = registry.workspaces.len();
            registry.workspaces.retain(|w| w.name != name);
            if before == registry.workspaces.len() {
                return Err(format!("workspace not found: {name}"));
            }
            if registry.active.as_deref() == Some(name.as_str()) {
                registry.active = registry.workspaces.first().map(|w| w.name.clone());
            }
            save_workspace_registry(&registry_path, &registry)?;
            Ok(json!({"status":"deleted","name":name}))
        }
    }
}

fn handle_daemon(cmd: DaemonSubcommand, root: &Path) -> CliResult<Value> {
    let state_path = daemon_state_path(root);
    let log_path = daemon_log_path(root);
    let now = Timestamp::now().as_nanos();
    let mut state = load_daemon_state(&state_path, &log_path);

    match cmd {
        DaemonSubcommand::Start { interval_secs } => {
            state.running = true;
            state.started_at = Some(now);
            state.stopped_at = None;
            state.interval_secs = interval_secs.max(1);
            state.last_heartbeat = now;
            append_daemon_log(&log_path, "daemon started")?;
            save_daemon_state(&state_path, &state)?;
            Ok(json!({"status":"running","interval_secs":state.interval_secs}))
        }
        DaemonSubcommand::Stop => {
            state.running = false;
            state.stopped_at = Some(now);
            append_daemon_log(&log_path, "daemon stopped")?;
            save_daemon_state(&state_path, &state)?;
            Ok(json!({"status":"stopped"}))
        }
        DaemonSubcommand::Status => Ok(json!(state)),
        DaemonSubcommand::Logs { lines } => {
            let content = fs::read_to_string(&log_path).unwrap_or_default();
            let tail: Vec<_> = content
                .lines()
                .rev()
                .take(lines)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            Ok(json!({ "lines": tail }))
        }
    }
}

fn parse_goal_id(value: &str) -> CliResult<GoalId> {
    Uuid::parse_str(value)
        .map(GoalId)
        .map_err(|e| format!("invalid goal id '{value}': {e}"))
}

fn parse_decision_id(value: &str) -> CliResult<DecisionId> {
    Uuid::parse_str(value)
        .map(DecisionId)
        .map_err(|e| format!("invalid decision id '{value}': {e}"))
}

fn parse_commitment_id(value: &str) -> CliResult<CommitmentId> {
    Uuid::parse_str(value)
        .map(CommitmentId)
        .map_err(|e| format!("invalid commitment id '{value}': {e}"))
}

fn parse_federation_id(value: &str) -> CliResult<FederationId> {
    Uuid::parse_str(value)
        .map(FederationId)
        .map_err(|e| format!("invalid federation id '{value}': {e}"))
}

fn parse_dream_id(value: &str) -> CliResult<DreamId> {
    Uuid::parse_str(value)
        .map(DreamId)
        .map_err(|e| format!("invalid dream id '{value}': {e}"))
}

fn parse_path_id(value: &str) -> CliResult<PathId> {
    Uuid::parse_str(value)
        .map(PathId)
        .map_err(|e| format!("invalid path id '{value}': {e}"))
}

fn parse_deadline(value: &str) -> CliResult<Timestamp> {
    let trimmed = value.trim();

    if let Ok(nanos) = trimmed.parse::<i64>() {
        return Ok(Timestamp::from_nanos(nanos));
    }

    if let Some(rest) = trimmed.strip_prefix("in ") {
        let parts: Vec<_> = rest.split_whitespace().collect();
        if parts.len() == 2 {
            let amount = parts[0]
                .parse::<f64>()
                .map_err(|e| format!("invalid relative deadline '{value}': {e}"))?;
            let unit = parts[1].to_ascii_lowercase();
            let days = if unit.starts_with("day") {
                amount
            } else if unit.starts_with("week") {
                amount * 7.0
            } else if unit.starts_with("hour") {
                amount / 24.0
            } else if unit.starts_with("min") {
                amount / (24.0 * 60.0)
            } else {
                return Err(format!(
                    "unsupported relative deadline unit in '{value}' (use day/week/hour/minute)"
                ));
            };
            return Ok(Timestamp::days_from_now(days));
        }
    }

    Err(format!(
        "invalid deadline '{value}' (use nanos timestamp or 'in N days')"
    ))
}

fn parse_goal_status(value: &str) -> CliResult<GoalStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "draft" => Ok(GoalStatus::Draft),
        "active" => Ok(GoalStatus::Active),
        "blocked" => Ok(GoalStatus::Blocked),
        "paused" => Ok(GoalStatus::Paused),
        "completed" => Ok(GoalStatus::Completed),
        "abandoned" => Ok(GoalStatus::Abandoned),
        "superseded" => Ok(GoalStatus::Superseded),
        "reborn" => Ok(GoalStatus::Reborn),
        other => Err(format!("invalid goal status: {other}")),
    }
}

fn parse_goal_status_list(values: &str) -> CliResult<Vec<GoalStatus>> {
    csv_values(values)
        .into_iter()
        .map(|v| parse_goal_status(&v))
        .collect()
}

fn parse_priority(value: &str) -> CliResult<Priority> {
    match value.trim().to_ascii_lowercase().as_str() {
        "critical" => Ok(Priority::Critical),
        "high" => Ok(Priority::High),
        "medium" => Ok(Priority::Medium),
        "low" => Ok(Priority::Low),
        "someday" => Ok(Priority::Someday),
        other => Err(format!("invalid priority: {other}")),
    }
}

fn parse_priority_list(values: &str) -> CliResult<Vec<Priority>> {
    csv_values(values)
        .into_iter()
        .map(|v| parse_priority(&v))
        .collect()
}

fn parse_entanglement_type(value: &str) -> CliResult<EntanglementType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "sequential" => Ok(EntanglementType::Sequential),
        "parallel" => Ok(EntanglementType::Parallel),
        "inverse" => Ok(EntanglementType::Inverse),
        "resonant" => Ok(EntanglementType::Resonant),
        "dependent" => Ok(EntanglementType::Dependent),
        other => Err(format!("invalid entanglement_type: {other}")),
    }
}

fn parse_scope_change_cli(
    change_type: &str,
    factor: Option<f64>,
    reason: Option<String>,
    new_direction: Option<String>,
    clarification: Option<String>,
) -> CliResult<ScopeChange> {
    match change_type.trim().to_ascii_lowercase().as_str() {
        "expansion" => Ok(ScopeChange::Expansion {
            factor: factor.unwrap_or(1.15),
            reason: reason.unwrap_or_else(|| "Scope expansion approved via CLI".to_string()),
        }),
        "contraction" => Ok(ScopeChange::Contraction {
            factor: factor.unwrap_or(0.85),
            reason: reason.unwrap_or_else(|| "Scope contraction approved via CLI".to_string()),
        }),
        "pivot" => Ok(ScopeChange::Pivot {
            new_direction: new_direction
                .ok_or_else(|| "pivot requires --new-direction".to_string())?,
            reason: reason.unwrap_or_else(|| "Direction pivot approved via CLI".to_string()),
        }),
        "refinement" => Ok(ScopeChange::Refinement {
            clarification: clarification
                .ok_or_else(|| "refinement requires --clarification".to_string())?,
        }),
        other => Err(format!(
            "invalid change_type: {other} (expected expansion|contraction|pivot|refinement)"
        )),
    }
}

fn state_root(file: Option<&Path>) -> CliResult<PathBuf> {
    if let Some(file) = file {
        let root = file.parent().unwrap_or_else(|| Path::new("."));
        return Ok(root.to_path_buf());
    }
    std::env::current_dir().map_err(|e| format!("failed to resolve current directory: {e}"))
}

fn workspace_registry_path(root: &Path) -> PathBuf {
    root.join(".aplan-workspaces.json")
}

fn load_workspace_registry(path: &Path) -> CliResult<WorkspaceRegistry> {
    if !path.exists() {
        return Ok(WorkspaceRegistry::default());
    }
    let bytes = fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn save_workspace_registry(path: &Path, registry: &WorkspaceRegistry) -> CliResult<()> {
    let data =
        serde_json::to_vec_pretty(registry).map_err(|e| format!("json encode failed: {e}"))?;
    fs::write(path, data).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn daemon_state_path(root: &Path) -> PathBuf {
    root.join(".aplan-daemon.json")
}

fn daemon_log_path(root: &Path) -> PathBuf {
    root.join(".aplan-daemon.log")
}

fn consensus_state_path(root: &Path) -> PathBuf {
    root.join(".aplan-consensus.json")
}

fn load_consensus_registry(path: &Path) -> CliResult<ConsensusRegistry> {
    if !path.exists() {
        return Ok(ConsensusRegistry::default());
    }
    let bytes = fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn save_consensus_registry(path: &Path, registry: &ConsensusRegistry) -> CliResult<()> {
    let data =
        serde_json::to_vec_pretty(registry).map_err(|e| format!("json encode failed: {e}"))?;
    fs::write(path, data).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn get_consensus_session_mut(
    registry: &mut ConsensusRegistry,
    decision_id: DecisionId,
) -> CliResult<&mut ConsensusSessionState> {
    registry
        .sessions
        .iter_mut()
        .find(|s| s.decision_id == decision_id.0.to_string())
        .ok_or_else(|| format!("consensus session not found for decision {}", decision_id.0))
}

fn get_consensus_session(
    registry: &ConsensusRegistry,
    decision_id: DecisionId,
) -> CliResult<&ConsensusSessionState> {
    registry
        .sessions
        .iter()
        .find(|s| s.decision_id == decision_id.0.to_string())
        .ok_or_else(|| format!("consensus session not found for decision {}", decision_id.0))
}

fn load_daemon_state(path: &Path, log_path: &Path) -> DaemonState {
    if path.exists() {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(state) = serde_json::from_slice::<DaemonState>(&bytes) {
                return state;
            }
        }
    }
    DaemonState {
        running: false,
        started_at: None,
        stopped_at: None,
        interval_secs: 60,
        last_heartbeat: Timestamp::now().as_nanos(),
        log_file: log_path.to_string_lossy().to_string(),
    }
}

fn save_daemon_state(path: &Path, state: &DaemonState) -> CliResult<()> {
    let data = serde_json::to_vec_pretty(state).map_err(|e| format!("json encode failed: {e}"))?;
    fs::write(path, data).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn append_daemon_log(path: &Path, event: &str) -> CliResult<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    let ts = Timestamp::now().as_nanos();
    writeln!(file, "[{ts}] {event}").map_err(|e| format!("failed to write daemon log: {e}"))
}

fn csv_values(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn run_stdio_loop<R: BufRead + Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    server: &mut PlanningMcpServer,
) {
    let mut line = String::new();
    let mut content_length: Option<usize> = None;

    loop {
        line.clear();
        let bytes = match reader.read_line(&mut line) {
            Ok(n) => n,
            Err(_) => break,
        };
        if bytes == 0 {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);

        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("content-length:") {
            let rest = trimmed.split_once(':').map(|(_, rhs)| rhs).unwrap_or("");
            match rest.trim().parse::<usize>() {
                Ok(n) if n <= MAX_CONTENT_LENGTH_BYTES => content_length = Some(n),
                Ok(n) => {
                    eprintln!(
                        "Content-Length {n} exceeds max frame size of {MAX_CONTENT_LENGTH_BYTES} bytes"
                    );
                    break;
                }
                Err(_) => {
                    eprintln!("Invalid Content-Length header: {trimmed}");
                    content_length = None;
                }
            }
            continue;
        }

        if let Some(n) = content_length {
            if trimmed.is_empty() {
                let mut buf = vec![0u8; n];
                if reader.read_exact(&mut buf).is_err() {
                    break;
                }
                let raw = String::from_utf8_lossy(&buf).to_string();
                let response = server.handle_raw(raw.trim());
                // Persist after every request — save() is a no-op when engine is clean
                if let Err(e) = server.save() {
                    eprintln!("Warning: failed to persist after request: {e}");
                }
                if !response.is_empty() && write_framed(writer, &response).is_err() {
                    break;
                }
                content_length = None;
                continue;
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        let response = server.handle_raw(trimmed);
        // Persist after every request — save() is a no-op when engine is clean
        if let Err(e) = server.save() {
            eprintln!("Warning: failed to persist after request: {e}");
        }
        if response.is_empty() {
            continue;
        }
        if writeln!(writer, "{}", response).is_err() {
            break;
        }
        if writer.flush().is_err() {
            break;
        }
    }
}

fn write_framed<W: Write>(writer: &mut W, response: &str) -> std::io::Result<()> {
    let len = response.len();
    write!(writer, "Content-Length: {}\r\n\r\n{}", len, response)?;
    writer.flush()
}

fn emit_output(value: &Value, format: OutputFormat, force_json: bool) {
    if force_json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
        );
        return;
    }
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
            );
        }
        OutputFormat::Text => {
            emit_text(value, 0);
        }
        OutputFormat::Table => {
            emit_table(value);
        }
    }
}

fn emit_text(value: &Value, indent: usize) {
    let pad = "  ".repeat(indent);
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                match v {
                    Value::Object(_) | Value::Array(_) => {
                        println!("{pad}{k}:");
                        emit_text(v, indent + 1);
                    }
                    _ => {
                        println!("{pad}{k}: {}", format_scalar(v));
                    }
                }
            }
        }
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                println!("{pad}[{i}]");
                emit_text(item, indent + 1);
            }
        }
        _ => {
            println!("{pad}{}", format_scalar(value));
        }
    }
}

fn format_scalar(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => value.to_string(),
    }
}

fn emit_table(value: &Value) {
    match value {
        Value::Array(arr) if !arr.is_empty() => {
            // Collect all keys from all objects for the header row
            let mut columns: Vec<String> = Vec::new();
            for item in arr {
                if let Value::Object(map) = item {
                    for key in map.keys() {
                        if !columns.contains(key) {
                            columns.push(key.clone());
                        }
                    }
                }
            }
            if columns.is_empty() {
                // Not an array of objects — fall back to indexed rows
                for (i, item) in arr.iter().enumerate() {
                    println!("{i}\t{}", format_scalar(item));
                }
                return;
            }
            // Compute column widths
            let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
            let rows: Vec<Vec<String>> = arr
                .iter()
                .map(|item| {
                    columns
                        .iter()
                        .enumerate()
                        .map(|(ci, col)| {
                            let cell = item.get(col).map(format_scalar).unwrap_or_default();
                            if cell.len() > widths[ci] {
                                widths[ci] = cell.len();
                            }
                            cell
                        })
                        .collect()
                })
                .collect();
            // Print header
            let header: String = columns
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{:<width$}", c, width = widths[i]))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{header}");
            let separator: String = widths
                .iter()
                .map(|w| "-".repeat(*w))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{separator}");
            // Print rows
            for row in &rows {
                let line: String = row
                    .iter()
                    .enumerate()
                    .map(|(i, cell)| format!("{:<width$}", cell, width = widths[i]))
                    .collect::<Vec<_>>()
                    .join("  ");
                println!("{line}");
            }
        }
        Value::Object(_) => {
            // Single object — display as key-value table
            emit_text(value, 0);
        }
        _ => {
            println!("{}", format_scalar(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_priority_list() {
        let parsed = parse_priority_list("high,medium,low").expect("priority list should parse");
        assert_eq!(parsed.len(), 3);
        assert!(matches!(parsed[0], Priority::High));
        assert!(matches!(parsed[1], Priority::Medium));
        assert!(matches!(parsed[2], Priority::Low));
    }

    #[test]
    fn parses_goal_status_list() {
        let parsed =
            parse_goal_status_list("draft,active,blocked").expect("status list should parse");
        assert_eq!(parsed.len(), 3);
        assert!(matches!(parsed[0], GoalStatus::Draft));
        assert!(matches!(parsed[1], GoalStatus::Active));
        assert!(matches!(parsed[2], GoalStatus::Blocked));
    }

    #[test]
    fn rejects_invalid_goal_id() {
        let err = parse_goal_id("not-a-uuid").expect_err("invalid id should fail");
        assert!(err.contains("invalid goal id"));
    }

    #[test]
    fn rejects_invalid_federation_id() {
        let err = parse_federation_id("not-a-uuid").expect_err("invalid federation id should fail");
        assert!(err.contains("invalid federation id"));
    }

    #[test]
    fn parses_relative_deadline_days() {
        let now = Timestamp::now().as_nanos();
        let parsed = parse_deadline("in 2 days").expect("relative deadline should parse");
        assert!(parsed.as_nanos() > now);
    }

    #[test]
    fn rejects_invalid_dream_id() {
        let err = parse_dream_id("not-a-uuid").expect_err("invalid id should fail");
        assert!(err.contains("invalid dream id"));
    }

    #[test]
    fn parse_scope_change_requires_fields() {
        let err = parse_scope_change_cli("pivot", None, None, None, None)
            .expect_err("pivot should require direction");
        assert!(err.contains("requires --new-direction"));
        let err = parse_scope_change_cli("refinement", None, None, None, None)
            .expect_err("refinement should require clarification");
        assert!(err.contains("requires --clarification"));
    }

    #[test]
    fn goal_create_supports_parent_tags_and_activate() {
        let mut engine = PlanningEngine::in_memory();
        let parent = engine
            .create_goal(CreateGoalRequest {
                title: "Parent".to_string(),
                intention: "root".to_string(),
                ..Default::default()
            })
            .expect("create parent");

        let mut mutated = false;
        let created = handle_goal(
            GoalSubcommand::Create {
                title: "Child".to_string(),
                description: None,
                intention: Some("child intention".to_string()),
                priority: Some("high".to_string()),
                deadline: None,
                parent: Some(parent.id.0.to_string()),
                tags: Some("ai,planning".to_string()),
                activate: true,
            },
            &mut engine,
            &mut mutated,
        )
        .expect("create child");
        assert!(mutated);
        assert_eq!(
            created.get("status").and_then(|v| v.as_str()),
            Some("Active")
        );

        let child_id = created
            .get("id")
            .and_then(|v| v.as_str())
            .expect("child id should exist")
            .to_string();

        let summary = handle_goal(
            GoalSubcommand::Show {
                id: child_id.clone(),
                full: false,
            },
            &mut engine,
            &mut mutated,
        )
        .expect("show summary");
        assert!(summary.get("blockers").is_some());
        assert!(summary.get("soul").is_none());

        let full = handle_goal(
            GoalSubcommand::Show {
                id: child_id,
                full: true,
            },
            &mut engine,
            &mut mutated,
        )
        .expect("show full");
        assert!(full.get("soul").is_some());
        assert!(full.get("tags").is_some());
    }

    #[test]
    fn consensus_flow_crystallizes_decision() {
        let mut engine = PlanningEngine::in_memory();
        let decision = engine
            .create_decision(CreateDecisionRequest {
                question: "Pick runtime".to_string(),
                ..Default::default()
            })
            .expect("create decision");
        engine
            .add_option(
                decision.id,
                DecisionPath {
                    id: PathId(Uuid::new_v4()),
                    name: "rust".to_string(),
                    description: "native".to_string(),
                    ..Default::default()
                },
            )
            .expect("add rust option");
        engine
            .add_option(
                decision.id,
                DecisionPath {
                    id: PathId(Uuid::new_v4()),
                    name: "python".to_string(),
                    description: "scripting".to_string(),
                    ..Default::default()
                },
            )
            .expect("add python option");

        let root = std::env::temp_dir().join(format!("aplan-consensus-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("create temp root");

        let mut mutated = false;
        handle_consensus(
            ConsensusSubcommand::Start {
                decision_id: decision.id.0.to_string(),
                stakeholders: Some("architect,pm".to_string()),
            },
            &mut engine,
            &mut mutated,
            &root,
        )
        .expect("start consensus");
        handle_consensus(
            ConsensusSubcommand::Vote {
                decision_id: decision.id.0.to_string(),
                stakeholder: "architect".to_string(),
                option: "rust".to_string(),
            },
            &mut engine,
            &mut mutated,
            &root,
        )
        .expect("vote 1");
        handle_consensus(
            ConsensusSubcommand::Vote {
                decision_id: decision.id.0.to_string(),
                stakeholder: "pm".to_string(),
                option: "rust".to_string(),
            },
            &mut engine,
            &mut mutated,
            &root,
        )
        .expect("vote 2");

        let synth = handle_consensus(
            ConsensusSubcommand::Synthesize {
                decision_id: decision.id.0.to_string(),
            },
            &mut engine,
            &mut mutated,
            &root,
        )
        .expect("synthesize");
        assert_eq!(
            synth.get("recommendation").and_then(|v| v.as_str()),
            Some("rust")
        );

        let crystallized = handle_consensus(
            ConsensusSubcommand::Crystallize {
                decision_id: decision.id.0.to_string(),
                option: None,
                rationale: Some("team alignment".to_string()),
            },
            &mut engine,
            &mut mutated,
            &root,
        )
        .expect("crystallize");
        assert_eq!(
            crystallized.get("status").and_then(|v| v.as_str()),
            Some("Crystallized")
        );
        assert!(mutated);
    }

    #[test]
    fn invention_cli_handlers_work() {
        let mut engine = PlanningEngine::in_memory();
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: "Invention Goal".to_string(),
                intention: "exercise cli invention handlers".to_string(),
                ..Default::default()
            })
            .expect("create goal");
        engine.activate_goal(goal.id).expect("activate goal");

        let mut mutated = false;
        let dream = handle_dream(
            DreamSubcommand::Goal {
                id: goal.id.0.to_string(),
            },
            &mut engine,
            &mut mutated,
        )
        .expect("dream goal");
        let dream_id = dream
            .get("id")
            .and_then(|v| v.as_str())
            .expect("dream id should exist")
            .to_string();
        assert!(mutated);

        let interpreted = handle_dream(
            DreamSubcommand::Interpret { dream_id },
            &mut engine,
            &mut mutated,
        )
        .expect("interpret dream");
        assert!(interpreted.get("interpretation").is_some());

        let detect = handle_metamorphosis(
            MetamorphosisSubcommand::Detect {
                goal_id: goal.id.0.to_string(),
            },
            &mut engine,
            &mut mutated,
        )
        .expect("detect metamorphosis");
        assert!(detect.get("should_transform").is_some());

        let _approved = handle_metamorphosis(
            MetamorphosisSubcommand::Approve {
                goal_id: goal.id.0.to_string(),
                title: Some("Refine".to_string()),
                description: Some("narrow scope".to_string()),
                change_type: "refinement".to_string(),
                factor: None,
                reason: None,
                new_direction: None,
                clarification: Some("focus on first slice".to_string()),
            },
            &mut engine,
            &mut mutated,
        )
        .expect("approve metamorphosis");

        let decision = engine
            .create_decision(CreateDecisionRequest {
                question: "Choose runtime".to_string(),
                ..Default::default()
            })
            .expect("create decision");
        let path_a = PathId(Uuid::new_v4());
        let path_b = PathId(Uuid::new_v4());
        engine
            .add_option(
                decision.id,
                DecisionPath {
                    id: path_a,
                    name: "rust".to_string(),
                    description: "native".to_string(),
                    ..Default::default()
                },
            )
            .expect("add option a");
        engine
            .add_option(
                decision.id,
                DecisionPath {
                    id: path_b,
                    name: "python".to_string(),
                    description: "scripting".to_string(),
                    ..Default::default()
                },
            )
            .expect("add option b");
        let child = engine
            .create_decision(CreateDecisionRequest {
                question: "Choose persistence".to_string(),
                caused_by: Some(decision.id),
                ..Default::default()
            })
            .expect("create child decision");

        let projection = handle_counterfactual(
            CounterfactualSubcommand::Project {
                decision_id: decision.id.0.to_string(),
                path_id: path_a.0.to_string(),
            },
            &engine,
        )
        .expect("project counterfactual");
        assert!(projection.get("final_state").is_some());

        let roots = handle_chain(
            ChainSubcommand::Roots {
                decision_id: child.id.0.to_string(),
            },
            &engine,
        )
        .expect("chain roots");
        let root_id = decision.id.0.to_string();
        assert_eq!(
            roots.get("root").and_then(|v| v.as_str()),
            Some(root_id.as_str())
        );
    }
}
