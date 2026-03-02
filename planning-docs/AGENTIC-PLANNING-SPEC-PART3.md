# AGENTIC PLANNING SPECIFICATION — PART 3

> **Specs Covered:** SPEC-09 through SPEC-12
> **Sister:** #8 of 25
> **Continues from:** Part 2

---

# SPEC-09: CLI

## 9.1 Command Structure

```
agentic-planning (aplan)
├── goal
│   ├── create          Create a new goal
│   ├── list            List goals with filters
│   ├── show            Show goal details
│   ├── activate        Activate a draft goal
│   ├── progress        Record progress
│   ├── complete        Mark goal as completed
│   ├── abandon         Abandon a goal
│   ├── pause           Pause a goal
│   ├── resume          Resume a paused goal
│   ├── block           Add a blocker
│   ├── unblock         Remove a blocker
│   ├── decompose       Create sub-goals
│   ├── link            Create relationship between goals
│   ├── tree            Show goal hierarchy
│   ├── feelings        Show goal's feelings
│   ├── physics         Show goal's physics
│   ├── dream           Trigger goal dreaming
│   └── reincarnate     Rebirth a dead goal
├── decision
│   ├── create          Start a decision
│   ├── option          Add an option
│   ├── crystallize     Make the choice
│   ├── show            Show decision details
│   ├── shadows         Show unchosen paths
│   ├── chain           Show decision chain
│   ├── archaeology     Excavate decisions behind state
│   ├── prophecy        Preview decision consequences
│   └── regret          Show regret analysis
├── commitment
│   ├── create          Make a commitment
│   ├── list            List commitments
│   ├── show            Show commitment details
│   ├── fulfill         Mark as fulfilled
│   ├── break           Mark as broken
│   ├── renegotiate     Change terms
│   ├── entangle        Link commitments
│   └── inventory       Show commitment inventory
├── progress
│   ├── momentum        Show momentum report
│   ├── gravity         Show gravity field
│   ├── blockers        Scan for predicted blockers
│   ├── echoes          Listen for completion echoes
│   └── forecast        Progress forecast
├── singularity
│   ├── collapse        Compute unified intention field
│   ├── position        Show goal's position in field
│   ├── path            Show optimal path
│   └── tensions        Show hidden tensions
├── federation
│   ├── create          Create federated goal
│   ├── join            Join federation
│   ├── sync            Sync with federation
│   ├── status          Show federation status
│   └── dream           Collective dreaming
├── workspace
│   ├── create          Create new workspace
│   ├── switch          Switch workspace
│   ├── list            List workspaces
│   └── compare         Compare workspaces
├── daemon
│   ├── start           Start background daemon
│   ├── stop            Stop daemon
│   ├── status          Show daemon status
│   └── logs            Show daemon logs
├── status              Quick status overview
├── serve               Start MCP server
└── version             Show version
```

## 9.2 CLI Implementation

```rust
//! src/cli/mod.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agentic-planning")]
#[command(bin_name = "aplan")]
#[command(about = "Persistent intention infrastructure for AI agents")]
#[command(version)]
pub struct Cli {
    /// Path to planning file
    #[arg(short, long, global = true)]
    pub file: Option<std::path::PathBuf>,
    
    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,
    
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// JSON output
    #[arg(long, global = true)]
    pub json: bool,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Table,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Goal management
    Goal(GoalCommand),
    
    /// Decision management
    Decision(DecisionCommand),
    
    /// Commitment management
    Commitment(CommitmentCommand),
    
    /// Progress physics
    Progress(ProgressCommand),
    
    /// Intention singularity
    Singularity(SingularityCommand),
    
    /// Federation management
    Federation(FederationCommand),
    
    /// Workspace management
    Workspace(WorkspaceCommand),
    
    /// Daemon management
    Daemon(DaemonCommand),
    
    /// Quick status overview
    Status,
    
    /// Start MCP server
    Serve {
        /// Server mode
        #[arg(long, default_value = "stdio")]
        mode: ServerMode,
        
        /// HTTP port (if mode is http)
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}
```

## 9.3 Goal Commands

```rust
//! src/cli/goal.rs

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct GoalCommand {
    #[command(subcommand)]
    pub command: GoalSubcommand,
}

#[derive(Subcommand)]
pub enum GoalSubcommand {
    /// Create a new goal
    Create {
        /// Goal title
        title: String,
        
        /// Goal description
        #[arg(short, long)]
        description: Option<String>,
        
        /// Original intention
        #[arg(short, long)]
        intention: Option<String>,
        
        /// Priority
        #[arg(short, long)]
        priority: Option<String>,
        
        /// Deadline (ISO 8601 or "in 7 days")
        #[arg(long)]
        deadline: Option<String>,
        
        /// Parent goal ID
        #[arg(long)]
        parent: Option<String>,
        
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        
        /// Activate immediately
        #[arg(long)]
        activate: bool,
    },
    
    /// List goals
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
        
        /// Filter by priority
        #[arg(short, long)]
        priority: Option<String>,
        
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
        
        /// Show only active
        #[arg(long)]
        active: bool,
        
        /// Show only blocked
        #[arg(long)]
        blocked: bool,
        
        /// Show only urgent (due soon)
        #[arg(long)]
        urgent: bool,
        
        /// Limit results
        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },
    
    /// Show goal details
    Show {
        /// Goal ID
        id: String,
        
        /// Show full details including dreams
        #[arg(long)]
        full: bool,
    },
    
    /// Activate a draft goal
    Activate {
        /// Goal ID
        id: String,
    },
    
    /// Record progress
    Progress {
        /// Goal ID
        id: String,
        
        /// Progress percentage (0-100)
        percentage: f64,
        
        /// Note
        #[arg(short, long)]
        note: Option<String>,
    },
    
    /// Complete a goal
    Complete {
        /// Goal ID
        id: String,
        
        /// Completion note
        #[arg(short, long)]
        note: Option<String>,
    },
    
    /// Abandon a goal
    Abandon {
        /// Goal ID
        id: String,
        
        /// Reason for abandonment
        reason: String,
    },
    
    /// Pause a goal
    Pause {
        /// Goal ID
        id: String,
        
        /// Reason
        #[arg(short, long)]
        reason: Option<String>,
    },
    
    /// Resume a paused goal
    Resume {
        /// Goal ID
        id: String,
    },
    
    /// Add a blocker
    Block {
        /// Goal ID
        id: String,
        
        /// Blocker description
        blocker: String,
        
        /// Blocker type
        #[arg(short, long, default_value = "unknown")]
        blocker_type: String,
        
        /// Severity (0-1)
        #[arg(short, long, default_value = "0.5")]
        severity: f64,
    },
    
    /// Remove a blocker
    Unblock {
        /// Goal ID
        id: String,
        
        /// Blocker ID
        blocker_id: String,
        
        /// Resolution description
        resolution: String,
    },
    
    /// Decompose into sub-goals
    Decompose {
        /// Goal ID
        id: String,
        
        /// Sub-goal titles (comma-separated or interactive)
        #[arg(short, long)]
        sub_goals: Option<String>,
        
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    
    /// Create relationship between goals
    Link {
        /// First goal ID
        goal_a: String,
        
        /// Second goal ID
        goal_b: String,
        
        /// Relationship type
        #[arg(short, long)]
        relationship: String,
    },
    
    /// Show goal hierarchy tree
    Tree {
        /// Root goal ID (or show all roots)
        #[arg(short, long)]
        root: Option<String>,
        
        /// Max depth
        #[arg(short, long)]
        depth: Option<usize>,
    },
    
    /// Show goal's feelings
    Feelings {
        /// Goal ID
        id: String,
    },
    
    /// Show goal's physics
    Physics {
        /// Goal ID
        id: String,
    },
    
    /// Trigger goal dreaming
    Dream {
        /// Goal ID
        id: String,
        
        /// Show dream details
        #[arg(long)]
        show: bool,
    },
    
    /// Reincarnate a dead goal
    Reincarnate {
        /// Original goal ID
        id: String,
        
        /// New title (optional)
        #[arg(short, long)]
        title: Option<String>,
        
        /// Lessons learned
        #[arg(short, long)]
        lessons: Option<String>,
    },
}

impl GoalSubcommand {
    pub async fn execute(self, engine: &mut PlanningEngine) -> Result<()> {
        match self {
            Self::Create { title, description, intention, priority, deadline, parent, tags, activate } => {
                let request = CreateGoalRequest {
                    title,
                    description: description.unwrap_or_default(),
                    intention: intention.unwrap_or_else(|| title.clone()),
                    priority: priority.and_then(|p| parse_priority(&p)),
                    deadline: deadline.and_then(|d| parse_deadline(&d)),
                    parent: parent.and_then(|p| parse_goal_id(&p)),
                    tags: tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
                    ..Default::default()
                };
                
                let goal = engine.create_goal(request)?;
                
                if activate {
                    engine.activate_goal(goal.id)?;
                }
                
                println!("✓ Created goal: {} ({})", goal.title, goal.id.0);
                Ok(())
            }
            
            Self::List { status, priority, tag, active, blocked, urgent, limit } => {
                let filter = GoalFilter {
                    status: status.map(|s| vec![parse_status(&s)]).flatten(),
                    priority: priority.map(|p| vec![parse_priority(&p)]).flatten(),
                    tags: tag.map(|t| vec![t]),
                    limit,
                    ..Default::default()
                };
                
                let goals = if active {
                    engine.get_active_goals()
                } else if blocked {
                    engine.get_blocked_goals()
                } else if urgent {
                    engine.get_urgent_goals(7.0)
                } else {
                    engine.list_goals(filter)
                };
                
                for goal in goals {
                    let status_icon = match goal.status {
                        GoalStatus::Active => "🟢",
                        GoalStatus::Blocked => "🔴",
                        GoalStatus::Completed => "✅",
                        GoalStatus::Paused => "⏸️",
                        GoalStatus::Abandoned => "❌",
                        _ => "⚪",
                    };
                    
                    println!("{} {} ({:.0}%) - {}",
                        status_icon,
                        goal.title,
                        goal.progress.percentage * 100.0,
                        goal.id.0
                    );
                }
                
                Ok(())
            }
            
            Self::Show { id, full } => {
                let goal_id = parse_goal_id(&id)?;
                let goal = engine.get_goal(goal_id)
                    .ok_or(Error::GoalNotFound(goal_id))?;
                
                println!("Goal: {}", goal.title);
                println!("═══════════════════════════════════════");
                println!("ID:          {}", goal.id.0);
                println!("Status:      {:?}", goal.status);
                println!("Priority:    {:?}", goal.priority);
                println!("Progress:    {:.1}%", goal.progress.percentage * 100.0);
                println!("Intention:   {}", goal.soul.intention);
                
                if let Some(deadline) = goal.deadline {
                    println!("Deadline:    {}", format_timestamp(deadline));
                }
                
                if !goal.blockers.is_empty() {
                    println!("\nBlockers:");
                    for b in &goal.blockers {
                        if b.resolved_at.is_none() {
                            println!("  🚫 {}", b.description);
                        }
                    }
                }
                
                println!("\nFeelings:");
                println!("  Urgency:    {:.2}", goal.feelings.urgency);
                println!("  Neglect:    {:.2}", goal.feelings.neglect);
                println!("  Confidence: {:.2}", goal.feelings.confidence);
                println!("  Vitality:   {:.2}", goal.feelings.vitality);
                
                println!("\nPhysics:");
                println!("  Momentum:   {:.2}", goal.physics.momentum);
                println!("  Gravity:    {:.2}", goal.physics.gravity);
                
                if full && !goal.dreams.is_empty() {
                    println!("\nDreams: {}", goal.dreams.len());
                }
                
                Ok(())
            }
            
            Self::Progress { id, percentage, note } => {
                let goal_id = parse_goal_id(&id)?;
                let progress = (percentage / 100.0).clamp(0.0, 1.0);
                
                let goal = engine.progress_goal(goal_id, progress, note)?;
                
                println!("✓ Progress updated: {} → {:.1}%", goal.title, progress * 100.0);
                
                if let Some(eta) = goal.progress.eta {
                    println!("  ETA: {}", format_timestamp(eta));
                }
                
                Ok(())
            }
            
            Self::Complete { id, note } => {
                let goal_id = parse_goal_id(&id)?;
                let goal = engine.complete_goal(goal_id, note)?;
                
                println!("✅ Goal completed: {}", goal.title);
                Ok(())
            }
            
            Self::Abandon { id, reason } => {
                let goal_id = parse_goal_id(&id)?;
                let goal = engine.abandon_goal(goal_id, reason)?;
                
                println!("❌ Goal abandoned: {}", goal.title);
                println!("   Soul preserved for potential reincarnation");
                Ok(())
            }
            
            Self::Dream { id, show } => {
                let goal_id = parse_goal_id(&id)?;
                let dream = engine.dream_goal(goal_id)?;
                
                println!("💭 Goal dreaming complete");
                println!("   Confidence: {:.2}", dream.confidence);
                println!("   Obstacles found: {}", dream.obstacles.len());
                println!("   Insights gained: {}", dream.insights.len());
                println!("   Sub-goals discovered: {}", dream.discovered_goals.len());
                
                if show {
                    println!("\n   Vision:");
                    println!("   {}", dream.scenario.vision);
                    
                    if !dream.obstacles.is_empty() {
                        println!("\n   Obstacles:");
                        for o in &dream.obstacles {
                            println!("   ⚠️  {} (severity: {:.2})", o.description, o.severity);
                        }
                    }
                }
                
                Ok(())
            }
            
            _ => {
                // Handle other subcommands
                Ok(())
            }
        }
    }
}
```

## 9.4 Decision Commands

```rust
//! src/cli/decision.rs

#[derive(Subcommand)]
pub enum DecisionSubcommand {
    /// Start a new decision
    Create {
        /// Decision question
        question: String,
        
        /// Context
        #[arg(short, long)]
        context: Option<String>,
        
        /// Related goal
        #[arg(short, long)]
        goal: Option<String>,
    },
    
    /// Add an option to a decision
    Option {
        /// Decision ID
        id: String,
        
        /// Option name
        name: String,
        
        /// Option description
        #[arg(short, long)]
        description: Option<String>,
        
        /// Pros (comma-separated)
        #[arg(long)]
        pros: Option<String>,
        
        /// Cons (comma-separated)
        #[arg(long)]
        cons: Option<String>,
    },
    
    /// Make the choice (crystallize)
    Crystallize {
        /// Decision ID
        id: String,
        
        /// Chosen path ID or name
        chosen: String,
        
        /// Reasoning
        #[arg(short, long)]
        reason: Option<String>,
    },
    
    /// Show decision details
    Show {
        /// Decision ID
        id: String,
    },
    
    /// Show unchosen paths (shadows)
    Shadows {
        /// Decision ID
        id: String,
    },
    
    /// Show decision chain
    Chain {
        /// Decision ID
        id: String,
    },
    
    /// Decision archaeology
    Archaeology {
        /// Artifact/state to analyze
        artifact: String,
    },
    
    /// Preview decision consequences
    Prophecy {
        /// Question
        question: String,
        
        /// Options (comma-separated)
        options: String,
    },
    
    /// Show regret analysis
    Regret {
        /// Decision ID (or "all" for overview)
        id: String,
    },
}

impl DecisionSubcommand {
    pub async fn execute(self, engine: &mut PlanningEngine) -> Result<()> {
        match self {
            Self::Create { question, context, goal } => {
                let request = CreateDecisionRequest {
                    question,
                    context,
                    goals: goal.and_then(|g| parse_goal_id(&g).ok()).map(|id| vec![id]),
                    ..Default::default()
                };
                
                let decision = engine.create_decision(request)?;
                
                println!("✓ Decision created: {}", decision.id.0);
                println!("  Question: {}", decision.question.question);
                println!("  Status: Pending (add options with `decision option`)");
                
                Ok(())
            }
            
            Self::Crystallize { id, chosen, reason } => {
                let decision_id = parse_decision_id(&id)?;
                let decision = engine.get_decision(decision_id)
                    .ok_or(Error::DecisionNotFound(decision_id))?;
                
                // Find path by name or ID
                let path_id = decision.shadows.iter()
                    .find(|s| s.path.name == chosen || s.path.id.0.to_string().starts_with(&chosen))
                    .map(|s| s.path.id)
                    .ok_or(Error::PathNotFound(PathId(Uuid::nil())))?;
                
                let reasoning = DecisionReasoning {
                    rationale: reason.unwrap_or_else(|| "User choice".to_string()),
                    ..Default::default()
                };
                
                let decision = engine.crystallize(decision_id, path_id, reasoning)?;
                
                println!("💎 Decision crystallized");
                println!("   Chosen: {}", decision.chosen.as_ref().map(|p| &p.name).unwrap_or(&"?".to_string()));
                println!("   Shadows preserved: {}", decision.shadows.len());
                
                Ok(())
            }
            
            Self::Shadows { id } => {
                let decision_id = parse_decision_id(&id)?;
                let shadows = engine.get_shadows(decision_id);
                
                println!("Crystal Shadows (paths not taken):");
                println!("═══════════════════════════════════════");
                
                for shadow in shadows {
                    println!("\n░░ {} ░░", shadow.path.name);
                    println!("   Rejection: {}", shadow.rejection_reason);
                    println!("   Resurrection cost: {:.2}", shadow.resurrection_cost);
                    
                    if let Some(cf) = &shadow.counterfactual {
                        println!("   Projected outcome: {}", cf.final_state);
                    }
                }
                
                Ok(())
            }
            
            Self::Archaeology { artifact } => {
                let dig = engine.decision_archaeology(&artifact);
                
                println!("Decision Archaeology: {}", artifact);
                println!("═══════════════════════════════════════");
                
                for stratum in &dig.strata {
                    println!("\nStratum {} (depth {}):", stratum.depth, stratum.depth);
                    println!("  Decision: {:?}", stratum.decision);
                    println!("  Age: {:?}", stratum.age);
                    println!("  Impact: {}", stratum.impact_on_artifact);
                }
                
                Ok(())
            }
            
            _ => Ok(()),
        }
    }
}
```

## 9.5 Progress Commands

```rust
//! src/cli/progress.rs

#[derive(Subcommand)]
pub enum ProgressSubcommand {
    /// Show momentum report
    Momentum {
        /// Goal ID (or all)
        #[arg(short, long)]
        goal: Option<String>,
    },
    
    /// Show gravity field
    Gravity,
    
    /// Scan for predicted blockers
    Blockers,
    
    /// Listen for completion echoes
    Echoes,
    
    /// Progress forecast
    Forecast {
        /// Goal ID
        id: String,
        
        /// Days to forecast
        #[arg(short, long, default_value = "30")]
        days: u32,
    },
}

impl ProgressSubcommand {
    pub async fn execute(self, engine: &PlanningEngine) -> Result<()> {
        match self {
            Self::Momentum { goal } => {
                if let Some(id) = goal {
                    let goal_id = parse_goal_id(&id)?;
                    let g = engine.get_goal(goal_id)
                        .ok_or(Error::GoalNotFound(goal_id))?;
                    
                    println!("Momentum Report: {}", g.title);
                    println!("═══════════════════════════════════════");
                    println!("  Current:     {:.2}", g.physics.momentum);
                    println!("  Trend:       {}", if g.physics.momentum > 0.5 { "↑" } else { "↓" });
                    println!("  Days active: {}", (Timestamp::now().0 - g.created_at.0) / (86400 * 1_000_000_000));
                } else {
                    println!("Momentum Report (All Active Goals)");
                    println!("═══════════════════════════════════════");
                    
                    for g in engine.get_active_goals() {
                        let bar = "█".repeat((g.physics.momentum * 10.0) as usize);
                        let empty = "░".repeat(10 - (g.physics.momentum * 10.0) as usize);
                        println!("  {} {} {:.2} {}", bar, empty, g.physics.momentum, g.title);
                    }
                }
                
                Ok(())
            }
            
            Self::Blockers => {
                let prophecies = engine.scan_blocker_prophecy();
                
                println!("Blocker Prophecy Scan");
                println!("═══════════════════════════════════════");
                
                if prophecies.is_empty() {
                    println!("✓ No blockers predicted");
                } else {
                    for p in prophecies {
                        println!("\n⚠️  {} (in ~{:.0} days)", 
                            p.predicted_blocker.description,
                            p.days_until_materialization
                        );
                        println!("   Goal: {:?}", p.goal_id);
                        println!("   Confidence: {:.2}", p.prediction_confidence);
                    }
                }
                
                Ok(())
            }
            
            Self::Echoes => {
                let echoes = engine.listen_progress_echoes();
                
                println!("Progress Echoes");
                println!("═══════════════════════════════════════");
                
                if echoes.is_empty() {
                    println!("No completion echoes detected");
                } else {
                    for e in echoes {
                        println!("\n🔮 Echo from: {}", e.source_milestone.name);
                        println!("   Strength: {:.2}", e.echo_strength);
                        println!("   ETA: {:?}", e.estimated_arrival);
                        println!("   Confidence: {:.2}", e.confidence);
                    }
                }
                
                Ok(())
            }
            
            _ => Ok(()),
        }
    }
}
```

## 9.6 Status Command

```rust
impl Commands {
    pub async fn execute_status(engine: &PlanningEngine) -> Result<()> {
        let active = engine.get_active_goals();
        let blocked = engine.get_blocked_goals();
        let urgent = engine.get_urgent_goals(7.0);
        let inventory = engine.get_commitment_inventory();
        
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                    PLANNING STATUS                        ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        
        println!("\n📊 Goals:");
        println!("   Active:  {}", active.len());
        println!("   Blocked: {}", blocked.len());
        println!("   Urgent:  {}", urgent.len());
        
        println!("\n📝 Commitments:");
        println!("   Active:  {}", inventory.active_count);
        println!("   Weight:  {:.2} / {:.2}", inventory.total_weight, inventory.sustainable_weight);
        if inventory.is_overloaded {
            println!("   ⚠️  OVERLOADED");
        }
        
        if !urgent.is_empty() {
            println!("\n🔥 Urgent (due within 7 days):");
            for g in urgent.iter().take(5) {
                println!("   • {} ({:.0}%)", g.title, g.progress.percentage * 100.0);
            }
        }
        
        if !blocked.is_empty() {
            println!("\n🚫 Blocked:");
            for g in blocked.iter().take(5) {
                let blocker = g.blockers.first()
                    .map(|b| b.description.as_str())
                    .unwrap_or("Unknown");
                println!("   • {} - {}", g.title, blocker);
            }
        }
        
        Ok(())
    }
}
```

---

# SPEC-10: MCP SERVER

## 10.1 Tool Consolidation

```
CONSOLIDATED MCP TOOLS (12 facades):
════════════════════════════════════

planning_goal        → create, list, show, activate, progress, complete, 
                       abandon, pause, resume, block, unblock, decompose,
                       link, tree, feelings, physics, dream, reincarnate

planning_decision    → create, option, crystallize, show, shadows, chain,
                       archaeology, prophecy, counterfactual, regret,
                       recrystallize

planning_commitment  → create, list, show, fulfill, break, renegotiate,
                       entangle, inventory, due_soon, at_risk

planning_progress    → momentum, gravity, blockers, echoes, forecast,
                       velocity, trend

planning_singularity → collapse, position, path, tensions, themes,
                       center, vision

planning_dream       → goal, collective, interpret, insights, accuracy,
                       history

planning_counterfactual → project, compare, learn, timeline

planning_chain       → trace, cascade, roots, leaves, visualize

planning_consensus   → start, round, synthesize, vote, status

planning_federate    → create, join, sync, handoff, status, members

planning_metamorphosis → detect, approve, history, predict, stage

planning_workspace   → create, switch, list, compare, merge, delete

12 tools × ~12 operations = ~144 operations
```

## 10.2 MCP Server Implementation

```rust
//! src/mcp/server.rs

use mcp_rs::{McpServer, Tool, Resource, Prompt};

pub struct PlanningMcpServer {
    engine: PlanningEngine,
    session_id: Option<String>,
}

impl PlanningMcpServer {
    pub fn new(engine: PlanningEngine) -> Self {
        Self {
            engine,
            session_id: None,
        }
    }
    
    /// Build tool definitions
    pub fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::new("planning_goal")
                .description("Living goal management - create, track, and evolve goals with full lifecycle support")
                .param("operation", "string", "Operation: create|list|show|activate|progress|complete|abandon|pause|resume|block|unblock|decompose|link|tree|feelings|physics|dream|reincarnate", true)
                .param("id", "string", "Goal ID (for operations on existing goals)", false)
                .param("title", "string", "Goal title (for create)", false)
                .param("intention", "string", "Original intention (for create)", false)
                .param("description", "string", "Goal description", false)
                .param("priority", "string", "Priority: critical|high|medium|low|someday", false)
                .param("deadline", "string", "Deadline (ISO 8601 or natural language)", false)
                .param("parent", "string", "Parent goal ID", false)
                .param("percentage", "number", "Progress percentage 0-100 (for progress)", false)
                .param("note", "string", "Note for progress/completion", false)
                .param("reason", "string", "Reason for abandon/pause", false)
                .param("blocker", "string", "Blocker description", false)
                .param("blocker_id", "string", "Blocker ID to resolve", false)
                .param("resolution", "string", "How blocker was resolved", false)
                .param("sub_goals", "array", "Sub-goal titles for decomposition", false)
                .param("relationship", "string", "Relationship type: alliance|rivalry|romance|dependency", false)
                .param("goal_b", "string", "Second goal ID for link", false)
                .param("filter", "object", "Filter for list operation", false),
            
            Tool::new("planning_decision")
                .description("Decision crystallization - make choices that solidify reality, with shadow path preservation")
                .param("operation", "string", "Operation: create|option|crystallize|show|shadows|chain|archaeology|prophecy|regret|recrystallize", true)
                .param("id", "string", "Decision ID", false)
                .param("question", "string", "Decision question (for create)", false)
                .param("context", "string", "Decision context", false)
                .param("goals", "array", "Related goal IDs", false)
                .param("name", "string", "Option name (for option)", false)
                .param("pros", "array", "Option pros", false)
                .param("cons", "array", "Option cons", false)
                .param("chosen", "string", "Chosen path ID/name (for crystallize)", false)
                .param("reasoning", "string", "Reasoning for choice", false)
                .param("artifact", "string", "Artifact for archaeology", false)
                .param("path_id", "string", "Path ID for counterfactual", false),
            
            Tool::new("planning_commitment")
                .description("Weighted commitment management - promises with accountability and physics")
                .param("operation", "string", "Operation: create|list|show|fulfill|break|renegotiate|entangle|inventory|due_soon|at_risk", true)
                .param("id", "string", "Commitment ID", false)
                .param("promise", "string", "Promise description (for create)", false)
                .param("stakeholder", "string", "Stakeholder name", false)
                .param("due", "string", "Due date (ISO 8601)", false)
                .param("goal", "string", "Related goal ID", false)
                .param("how_delivered", "string", "How commitment was fulfilled", false)
                .param("new_promise", "string", "New promise terms (for renegotiate)", false)
                .param("reason", "string", "Reason for break/renegotiate", false)
                .param("commitment_b", "string", "Second commitment for entangle", false)
                .param("entanglement_type", "string", "Type: sequential|parallel|inverse|resonant", false)
                .param("within_days", "number", "Days for due_soon filter", false),
            
            Tool::new("planning_progress")
                .description("Progress physics - momentum, gravity, blocker prophecy, completion echoes")
                .param("operation", "string", "Operation: momentum|gravity|blockers|echoes|forecast|velocity|trend", true)
                .param("goal_id", "string", "Goal ID (for specific goal analysis)", false)
                .param("days", "number", "Days for forecast", false),
            
            Tool::new("planning_singularity")
                .description("Intention singularity - unified field of all intentions")
                .param("operation", "string", "Operation: collapse|position|path|tensions|themes|center|vision", true)
                .param("goal_id", "string", "Goal ID for position", false),
            
            Tool::new("planning_dream")
                .description("Goal dreaming - simulate completion, discover insights")
                .param("operation", "string", "Operation: goal|collective|interpret|insights|accuracy|history", true)
                .param("goal_id", "string", "Goal ID to dream", false)
                .param("dream_id", "string", "Dream ID for analysis", false),
            
            Tool::new("planning_counterfactual")
                .description("Counterfactual projection - see what would have happened")
                .param("operation", "string", "Operation: project|compare|learn|timeline", true)
                .param("decision_id", "string", "Decision ID", false)
                .param("path_id", "string", "Shadow path ID to project", false),
            
            Tool::new("planning_chain")
                .description("Decision chain analysis - trace causality")
                .param("operation", "string", "Operation: trace|cascade|roots|leaves|visualize", true)
                .param("decision_id", "string", "Decision ID", false),
            
            Tool::new("planning_consensus")
                .description("Multi-stakeholder decision consensus")
                .param("operation", "string", "Operation: start|round|synthesize|vote|status", true)
                .param("decision_id", "string", "Decision ID", false)
                .param("stakeholders", "array", "Stakeholder list", false)
                .param("position", "string", "Position statement", false),
            
            Tool::new("planning_federate")
                .description("Goal federation across agents")
                .param("operation", "string", "Operation: create|join|sync|handoff|status|members", true)
                .param("goal_id", "string", "Goal ID", false)
                .param("federation_id", "string", "Federation ID", false)
                .param("agent_id", "string", "Agent ID", false),
            
            Tool::new("planning_metamorphosis")
                .description("Goal metamorphosis tracking")
                .param("operation", "string", "Operation: detect|approve|history|predict|stage", true)
                .param("goal_id", "string", "Goal ID", false),
            
            Tool::new("planning_workspace")
                .description("Workspace management")
                .param("operation", "string", "Operation: create|switch|list|compare|merge|delete", true)
                .param("name", "string", "Workspace name", false)
                .param("path", "string", "Workspace path", false),
        ]
    }
    
    /// Handle tool call
    pub async fn handle_tool(
        &mut self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let operation = params.get("operation")
            .and_then(|v| v.as_str())
            .ok_or(Error::MissingOperation)?;
        
        match name {
            "planning_goal" => self.handle_goal(operation, params).await,
            "planning_decision" => self.handle_decision(operation, params).await,
            "planning_commitment" => self.handle_commitment(operation, params).await,
            "planning_progress" => self.handle_progress(operation, params).await,
            "planning_singularity" => self.handle_singularity(operation, params).await,
            "planning_dream" => self.handle_dream(operation, params).await,
            "planning_counterfactual" => self.handle_counterfactual(operation, params).await,
            "planning_chain" => self.handle_chain(operation, params).await,
            "planning_consensus" => self.handle_consensus(operation, params).await,
            "planning_federate" => self.handle_federate(operation, params).await,
            "planning_metamorphosis" => self.handle_metamorphosis(operation, params).await,
            "planning_workspace" => self.handle_workspace(operation, params).await,
            _ => Err(Error::UnknownTool(name.to_string())),
        }
    }
}
```

## 10.3 Tool Handlers

```rust
impl PlanningMcpServer {
    async fn handle_goal(
        &mut self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match operation {
            "create" => {
                let title = get_string(&params, "title")?;
                let intention = get_string_opt(&params, "intention")
                    .unwrap_or_else(|| title.clone());
                
                let request = CreateGoalRequest {
                    title,
                    description: get_string_opt(&params, "description").unwrap_or_default(),
                    intention,
                    priority: get_string_opt(&params, "priority")
                        .and_then(|p| parse_priority(&p)),
                    deadline: get_string_opt(&params, "deadline")
                        .and_then(|d| parse_deadline(&d)),
                    parent: get_string_opt(&params, "parent")
                        .and_then(|p| parse_goal_id(&p).ok()),
                    ..Default::default()
                };
                
                let goal = self.engine.create_goal(request)?;
                Ok(serde_json::to_value(goal)?)
            }
            
            "list" => {
                let filter = if let Some(f) = params.get("filter") {
                    serde_json::from_value(f.clone())?
                } else {
                    GoalFilter::default()
                };
                
                let goals: Vec<_> = self.engine.list_goals(filter)
                    .into_iter()
                    .cloned()
                    .collect();
                
                Ok(serde_json::to_value(goals)?)
            }
            
            "show" => {
                let id = get_goal_id(&params)?;
                let goal = self.engine.get_goal(id)
                    .ok_or(Error::GoalNotFound(id))?;
                
                Ok(serde_json::to_value(goal)?)
            }
            
            "activate" => {
                let id = get_goal_id(&params)?;
                let goal = self.engine.activate_goal(id)?;
                Ok(serde_json::to_value(goal)?)
            }
            
            "progress" => {
                let id = get_goal_id(&params)?;
                let percentage = get_number(&params, "percentage")? / 100.0;
                let note = get_string_opt(&params, "note");
                
                let goal = self.engine.progress_goal(id, percentage, note)?;
                Ok(serde_json::to_value(goal)?)
            }
            
            "complete" => {
                let id = get_goal_id(&params)?;
                let note = get_string_opt(&params, "note");
                
                let goal = self.engine.complete_goal(id, note)?;
                Ok(serde_json::to_value(goal)?)
            }
            
            "abandon" => {
                let id = get_goal_id(&params)?;
                let reason = get_string(&params, "reason")?;
                
                let goal = self.engine.abandon_goal(id, reason)?;
                Ok(serde_json::to_value(goal)?)
            }
            
            "dream" => {
                let id = get_goal_id(&params)?;
                let dream = self.engine.dream_goal(id)?;
                Ok(serde_json::to_value(dream)?)
            }
            
            "feelings" => {
                let id = get_goal_id(&params)?;
                let goal = self.engine.get_goal(id)
                    .ok_or(Error::GoalNotFound(id))?;
                
                Ok(serde_json::to_value(&goal.feelings)?)
            }
            
            "physics" => {
                let id = get_goal_id(&params)?;
                let goal = self.engine.get_goal(id)
                    .ok_or(Error::GoalNotFound(id))?;
                
                Ok(serde_json::to_value(&goal.physics)?)
            }
            
            "tree" => {
                let root_id = get_string_opt(&params, "id")
                    .and_then(|s| parse_goal_id(&s).ok());
                
                if let Some(id) = root_id {
                    let tree = self.engine.get_goal_tree(id);
                    Ok(serde_json::to_value(tree)?)
                } else {
                    // Return all root goals with their trees
                    let roots = self.engine.get_root_goals();
                    Ok(serde_json::to_value(roots)?)
                }
            }
            
            _ => Err(Error::UnknownOperation(operation.to_string())),
        }
    }
    
    async fn handle_progress(
        &mut self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match operation {
            "blockers" => {
                let prophecies = self.engine.scan_blocker_prophecy();
                Ok(serde_json::to_value(prophecies)?)
            }
            
            "echoes" => {
                let echoes = self.engine.listen_progress_echoes();
                Ok(serde_json::to_value(echoes)?)
            }
            
            "momentum" => {
                if let Some(id) = get_string_opt(&params, "goal_id") {
                    let goal_id = parse_goal_id(&id)?;
                    let goal = self.engine.get_goal(goal_id)
                        .ok_or(Error::GoalNotFound(goal_id))?;
                    
                    Ok(serde_json::json!({
                        "goal_id": goal_id,
                        "momentum": goal.physics.momentum,
                        "trend": if goal.physics.momentum > 0.5 { "up" } else { "down" }
                    }))
                } else {
                    // All active goals momentum
                    let report: Vec<_> = self.engine.get_active_goals()
                        .iter()
                        .map(|g| serde_json::json!({
                            "goal_id": g.id,
                            "title": g.title,
                            "momentum": g.physics.momentum
                        }))
                        .collect();
                    
                    Ok(serde_json::to_value(report)?)
                }
            }
            
            "gravity" => {
                let field: Vec<_> = self.engine.get_active_goals()
                    .iter()
                    .map(|g| serde_json::json!({
                        "goal_id": g.id,
                        "title": g.title,
                        "gravity": g.physics.gravity,
                        "pull": if g.physics.gravity > 0.7 { "strong" } else if g.physics.gravity > 0.3 { "moderate" } else { "weak" }
                    }))
                    .collect();
                
                Ok(serde_json::to_value(field)?)
            }
            
            _ => Err(Error::UnknownOperation(operation.to_string())),
        }
    }
    
    async fn handle_singularity(
        &mut self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match operation {
            "collapse" => {
                let singularity = self.engine.get_intention_singularity();
                Ok(serde_json::to_value(singularity)?)
            }
            
            "position" => {
                let id = get_goal_id(&params)?;
                let singularity = self.engine.get_intention_singularity();
                
                let position = singularity.goal_positions.get(&id)
                    .ok_or(Error::GoalNotFound(id))?;
                
                Ok(serde_json::to_value(position)?)
            }
            
            "path" => {
                let singularity = self.engine.get_intention_singularity();
                Ok(serde_json::to_value(&singularity.golden_path)?)
            }
            
            "tensions" => {
                let singularity = self.engine.get_intention_singularity();
                Ok(serde_json::to_value(&singularity.tension_lines)?)
            }
            
            _ => Err(Error::UnknownOperation(operation.to_string())),
        }
    }
}
```

## 10.4 Resources and Prompts

```rust
impl PlanningMcpServer {
    /// Build resource definitions
    pub fn resources(&self) -> Vec<Resource> {
        vec![
            Resource::new("planning://goals")
                .description("All goals")
                .mime_type("application/json"),
            
            Resource::new("planning://goals/{id}")
                .description("Specific goal by ID")
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
        ]
    }
    
    /// Build prompt definitions
    pub fn prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt::new("planning_review")
                .description("Generate a planning review")
                .arg("period", "string", "Review period: daily|weekly|monthly"),
            
            Prompt::new("goal_decomposition")
                .description("Help decompose a goal into sub-goals")
                .arg("goal_id", "string", "Goal to decompose"),
            
            Prompt::new("decision_analysis")
                .description("Analyze a pending decision")
                .arg("question", "string", "Decision question"),
            
            Prompt::new("commitment_check")
                .description("Check commitment health")
                .arg("stakeholder", "string", "Stakeholder name (optional)"),
        ]
    }
}
```

---

# SPEC-11: SISTER INTEGRATION

## 11.1 Sister Bridge Traits

```rust
//! src/bridges/mod.rs

/// Bridge to Temporal Module
pub trait TimeBridge {
    /// Get deadline context
    fn get_deadline_context(&self, deadline: Timestamp) -> TimeContext;
    
    /// Schedule a goal check
    fn schedule_check(&self, goal_id: GoalId, when: Timestamp) -> Result<()>;
    
    /// Get temporal decay for goal
    fn get_decay(&self, goal: &Goal) -> f64;
    
    /// Calculate urgency based on time
    fn calculate_time_urgency(&self, goal: &Goal) -> f64;
}

/// Bridge to AgenticContract
pub trait ContractBridge {
    /// Check if action is allowed
    fn check_policy(&self, action: &str, context: &PolicyContext) -> PolicyResult;
    
    /// Create commitment contract
    fn create_contract(&self, commitment: &Commitment) -> Result<ContractId>;
    
    /// Check commitment compliance
    fn check_compliance(&self, commitment_id: CommitmentId) -> ComplianceResult;
    
    /// Get approval for decision
    fn request_approval(&self, decision: &Decision) -> Result<ApprovalStatus>;
}

/// Bridge to AgenticMemory
pub trait MemoryBridge {
    /// Store goal as memory
    fn persist_goal(&self, goal: &Goal) -> Result<MemoryId>;
    
    /// Retrieve goal context
    fn get_goal_context(&self, goal_id: GoalId) -> Vec<Memory>;
    
    /// Store decision
    fn persist_decision(&self, decision: &Decision) -> Result<MemoryId>;
    
    /// Search memories for planning context
    fn search_context(&self, query: &str) -> Vec<Memory>;
}

/// Bridge to AgenticIdentity
pub trait IdentityBridge {
    /// Sign a decision receipt
    fn sign_decision(&self, decision: &Decision) -> Result<Receipt>;
    
    /// Sign a commitment
    fn sign_commitment(&self, commitment: &Commitment) -> Result<Receipt>;
    
    /// Verify agent identity for federation
    fn verify_agent(&self, agent_id: &str) -> Result<AgentIdentity>;
    
    /// Get accountability chain
    fn get_accountability_chain(&self, goal_id: GoalId) -> Vec<Receipt>;
}

/// Bridge to AgenticCognition
pub trait CognitionBridge {
    /// Get user model for prioritization
    fn get_user_model(&self) -> UserModel;
    
    /// Predict user preference
    fn predict_preference(&self, options: &[DecisionPath]) -> Vec<f64>;
    
    /// Analyze decision patterns
    fn analyze_patterns(&self, decisions: &[Decision]) -> PatternAnalysis;
}

/// Bridge to AgenticVision (for evidence)
pub trait VisionBridge {
    /// Capture goal evidence
    fn capture_evidence(&self, goal_id: GoalId) -> Result<EvidenceId>;
    
    /// Link evidence to progress
    fn link_evidence(&self, goal_id: GoalId, evidence_id: EvidenceId) -> Result<()>;
}
```

## 11.2 Temporal Integration

```rust
//! src/bridges/time.rs

use temporal_bridge::{TemporalEngine, Deadline, Schedule};

pub struct TimeIntegration {
    time: TemporalEngine,
}

impl TimeBridge for TimeIntegration {
    fn get_deadline_context(&self, deadline: Timestamp) -> TimeContext {
        let now = Timestamp::now();
        let remaining = deadline.0 - now.0;
        let days_remaining = remaining as f64 / (86400.0 * 1e9);
        
        TimeContext {
            deadline,
            days_remaining,
            is_overdue: days_remaining < 0.0,
            urgency_level: match days_remaining {
                d if d < 0.0 => UrgencyLevel::Overdue,
                d if d < 1.0 => UrgencyLevel::Critical,
                d if d < 3.0 => UrgencyLevel::High,
                d if d < 7.0 => UrgencyLevel::Medium,
                _ => UrgencyLevel::Low,
            },
        }
    }
    
    fn schedule_check(&self, goal_id: GoalId, when: Timestamp) -> Result<()> {
        self.time.schedule(Schedule {
            id: Uuid::new_v4(),
            name: format!("Goal check: {:?}", goal_id),
            trigger: when,
            action: ScheduleAction::GoalCheck { goal_id },
            recurring: None,
        })
    }
    
    fn calculate_time_urgency(&self, goal: &Goal) -> f64 {
        if let Some(deadline) = goal.deadline {
            let context = self.get_deadline_context(deadline);
            
            match context.urgency_level {
                UrgencyLevel::Overdue => 1.0,
                UrgencyLevel::Critical => 0.95,
                UrgencyLevel::High => 0.8,
                UrgencyLevel::Medium => 0.5,
                UrgencyLevel::Low => 0.2,
            }
        } else {
            0.1  // No deadline = low urgency
        }
    }
}
```

## 11.3 Contract Integration

```rust
//! src/bridges/contract.rs

use agentic_contract::{ContractEngine, Policy, RiskLimit};

pub struct ContractIntegration {
    contract: ContractEngine,
}

impl ContractBridge for ContractIntegration {
    fn check_policy(&self, action: &str, context: &PolicyContext) -> PolicyResult {
        // Check if action is allowed by policies
        let policies = self.contract.get_active_policies();
        
        for policy in policies {
            if policy.applies_to(action) {
                if !policy.allows(context) {
                    return PolicyResult::Denied {
                        policy: policy.name.clone(),
                        reason: policy.denial_reason(context),
                    };
                }
            }
        }
        
        PolicyResult::Allowed
    }
    
    fn create_contract(&self, commitment: &Commitment) -> Result<ContractId> {
        let contract = self.contract.create_commitment_contract(
            &commitment.promise.description,
            &commitment.made_to.name,
            commitment.due,
            commitment.weight,
        )?;
        
        Ok(contract.id)
    }
    
    fn request_approval(&self, decision: &Decision) -> Result<ApprovalStatus> {
        // Check if decision requires approval
        let risk = self.assess_decision_risk(decision);
        
        if risk > 0.7 {
            // High risk, requires approval
            let approval = self.contract.request_approval(ApprovalRequest {
                subject: format!("Decision: {}", decision.question.question),
                risk_level: risk,
                context: decision.question.context.clone(),
            })?;
            
            Ok(approval.status)
        } else {
            Ok(ApprovalStatus::AutoApproved)
        }
    }
}
```

## 11.4 Memory Integration

```rust
//! src/bridges/memory.rs

use agentic_memory::{MemoryEngine, MemoryEvent, EventType};

pub struct MemoryIntegration {
    memory: MemoryEngine,
}

impl MemoryBridge for MemoryIntegration {
    fn persist_goal(&self, goal: &Goal) -> Result<MemoryId> {
        let event = MemoryEvent {
            event_type: EventType::Intention,
            content: format!("Goal: {} - {}", goal.title, goal.soul.intention),
            timestamp: goal.created_at,
            metadata: Some(serde_json::json!({
                "goal_id": goal.id,
                "status": goal.status,
                "progress": goal.progress.percentage,
            })),
            ..Default::default()
        };
        
        self.memory.write(event)
    }
    
    fn get_goal_context(&self, goal_id: GoalId) -> Vec<Memory> {
        // Search for memories related to this goal
        self.memory.search(&format!("goal:{:?}", goal_id))
            .unwrap_or_default()
    }
    
    fn persist_decision(&self, decision: &Decision) -> Result<MemoryId> {
        let content = if let Some(chosen) = &decision.chosen {
            format!("Decision: {} → Chose: {}", 
                decision.question.question, 
                chosen.name)
        } else {
            format!("Decision pending: {}", decision.question.question)
        };
        
        let event = MemoryEvent {
            event_type: EventType::Decision,
            content,
            timestamp: decision.crystallized_at.unwrap_or_else(Timestamp::now),
            metadata: Some(serde_json::json!({
                "decision_id": decision.id,
                "status": decision.status,
                "shadows": decision.shadows.len(),
            })),
            ..Default::default()
        };
        
        self.memory.write(event)
    }
    
    fn search_context(&self, query: &str) -> Vec<Memory> {
        self.memory.search(query).unwrap_or_default()
    }
}
```

## 11.5 Hydra Bridge

```rust
//! src/bridges/hydra.rs

use crate::HydraAdapter;

/// Planning adapter for Hydra orchestration
pub struct PlanningHydraAdapter {
    engine: PlanningEngine,
}

impl HydraAdapter for PlanningHydraAdapter {
    fn id(&self) -> &str {
        "agentic-planning"
    }
    
    fn adapter_type(&self) -> AdapterType {
        AdapterType::Sister
    }
    
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::GoalManagement,
            Capability::DecisionSupport,
            Capability::CommitmentTracking,
            Capability::ProgressPhysics,
            Capability::IntentionSingularity,
        ]
    }
    
    fn health(&self) -> HealthStatus {
        HealthStatus::Healthy {
            goals: self.engine.goal_count(),
            decisions: self.engine.decision_count(),
            commitments: self.engine.commitment_count(),
        }
    }
    
    fn handle(&mut self, request: AdapterRequest) -> Result<AdapterResponse> {
        match request {
            AdapterRequest::GetActiveGoals => {
                let goals = self.engine.get_active_goals();
                Ok(AdapterResponse::Goals(goals.into_iter().cloned().collect()))
            }
            
            AdapterRequest::GetBlockers => {
                let prophecies = self.engine.scan_blocker_prophecy();
                Ok(AdapterResponse::Blockers(prophecies))
            }
            
            AdapterRequest::GetSingularity => {
                let singularity = self.engine.get_intention_singularity();
                Ok(AdapterResponse::Singularity(singularity))
            }
            
            AdapterRequest::RecordProgress { goal_id, percentage, note } => {
                let goal = self.engine.progress_goal(goal_id, percentage, note)?;
                Ok(AdapterResponse::Goal(goal))
            }
            
            _ => Err(Error::UnsupportedRequest),
        }
    }
}
```

---

# SPEC-12: TESTS

## 12.1 Test Structure

```
tests/
├── unit/
│   ├── goal_test.rs
│   ├── decision_test.rs
│   ├── commitment_test.rs
│   ├── progress_test.rs
│   ├── indexes_test.rs
│   └── validation_test.rs
├── integration/
│   ├── lifecycle_test.rs
│   ├── singularity_test.rs
│   ├── federation_test.rs
│   └── bridge_test.rs
├── stress/
│   ├── large_graph_test.rs
│   ├── concurrent_test.rs
│   └── persistence_test.rs
└── scenarios/
    └── phase_scenarios.rs
```

## 12.2 Test Scenarios (16 Required)

```rust
//! tests/scenarios/phase_scenarios.rs

/// SCENARIO 1: Goal Creation and Activation
#[test]
fn scenario_01_goal_lifecycle() {
    let mut engine = PlanningEngine::in_memory();
    
    // Create goal
    let goal = engine.create_goal(CreateGoalRequest {
        title: "Build REST API".to_string(),
        intention: "Create a production REST API".to_string(),
        priority: Some(Priority::High),
        ..Default::default()
    }).unwrap();
    
    assert_eq!(goal.status, GoalStatus::Draft);
    assert_eq!(goal.progress.percentage, 0.0);
    
    // Activate
    let goal = engine.activate_goal(goal.id).unwrap();
    assert_eq!(goal.status, GoalStatus::Active);
    assert!(goal.activated_at.is_some());
    
    // Progress
    let goal = engine.progress_goal(goal.id, 0.5, Some("Halfway done".to_string())).unwrap();
    assert_eq!(goal.progress.percentage, 0.5);
    
    // Complete
    let goal = engine.complete_goal(goal.id, None).unwrap();
    assert_eq!(goal.status, GoalStatus::Completed);
}

/// SCENARIO 2: Decision Crystallization
#[test]
fn scenario_02_decision_crystallization() {
    let mut engine = PlanningEngine::in_memory();
    
    // Create decision
    let decision = engine.create_decision(CreateDecisionRequest {
        question: "Which database to use?".to_string(),
        ..Default::default()
    }).unwrap();
    
    // Add options
    engine.add_option(decision.id, DecisionPath {
        id: PathId(Uuid::new_v4()),
        name: "PostgreSQL".to_string(),
        description: "Relational database".to_string(),
        pros: vec!["ACID".to_string()],
        cons: vec!["Setup complexity".to_string()],
        ..Default::default()
    }).unwrap();
    
    engine.add_option(decision.id, DecisionPath {
        id: PathId(Uuid::new_v4()),
        name: "MongoDB".to_string(),
        description: "Document database".to_string(),
        pros: vec!["Flexible schema".to_string()],
        cons: vec!["No joins".to_string()],
        ..Default::default()
    }).unwrap();
    
    let decision = engine.get_decision(decision.id).unwrap();
    assert_eq!(decision.shadows.len(), 2);
    
    // Crystallize
    let postgres_id = decision.shadows[0].path.id;
    let decision = engine.crystallize(
        decision.id,
        postgres_id,
        DecisionReasoning {
            rationale: "Need ACID compliance".to_string(),
            ..Default::default()
        },
    ).unwrap();
    
    assert_eq!(decision.status, DecisionStatus::Crystallized);
    assert_eq!(decision.chosen.as_ref().unwrap().name, "PostgreSQL");
    assert_eq!(decision.shadows.len(), 1);  // MongoDB is now a shadow
}

/// SCENARIO 3: Commitment Weight and Fulfillment
#[test]
fn scenario_03_commitment_lifecycle() {
    let mut engine = PlanningEngine::in_memory();
    
    let commitment = engine.create_commitment(CreateCommitmentRequest {
        promise: Promise {
            description: "Deliver MVP by end of month".to_string(),
            deliverables: vec!["Working API".to_string()],
            conditions: vec![],
        },
        stakeholder: Stakeholder {
            id: StakeholderId(Uuid::new_v4()),
            name: "Product Manager".to_string(),
            role: "Stakeholder".to_string(),
            importance: 0.9,
        },
        due: Some(Timestamp::now()),
        ..Default::default()
    }).unwrap();
    
    assert!(commitment.weight > 0.0);
    assert_eq!(commitment.status, CommitmentStatus::Active);
    
    // Fulfill
    let commitment = engine.fulfill_commitment(
        commitment.id,
        "Delivered on time".to_string(),
    ).unwrap();
    
    assert_eq!(commitment.status, CommitmentStatus::Fulfilled);
    assert!(commitment.fulfillment.as_ref().unwrap().energy_released > 0.0);
}

/// SCENARIO 4: Goal Hierarchy and Decomposition
#[test]
fn scenario_04_goal_hierarchy() {
    let mut engine = PlanningEngine::in_memory();
    
    // Create parent goal
    let parent = engine.create_goal(CreateGoalRequest {
        title: "Build Application".to_string(),
        intention: "Complete application".to_string(),
        ..Default::default()
    }).unwrap();
    
    // Decompose
    let children = engine.decompose_goal(parent.id, vec![
        CreateGoalRequest {
            title: "Build Backend".to_string(),
            intention: "API development".to_string(),
            ..Default::default()
        },
        CreateGoalRequest {
            title: "Build Frontend".to_string(),
            intention: "UI development".to_string(),
            ..Default::default()
        },
    ]).unwrap();
    
    assert_eq!(children.len(), 2);
    
    let tree = engine.get_goal_tree(parent.id).unwrap();
    assert_eq!(tree.nodes.len(), 3);  // Parent + 2 children
}

/// SCENARIO 5: Blocker Detection and Resolution
#[test]
fn scenario_05_blocker_lifecycle() {
    let mut engine = PlanningEngine::in_memory();
    
    let goal = engine.create_goal(CreateGoalRequest {
        title: "Deploy to Production".to_string(),
        intention: "Ship it".to_string(),
        ..Default::default()
    }).unwrap();
    
    engine.activate_goal(goal.id).unwrap();
    
    // Add blocker
    let blocker = Blocker {
        id: Uuid::new_v4(),
        blocker_type: BlockerType::ApprovalPending { 
            approver: StakeholderId(Uuid::new_v4()) 
        },
        description: "Waiting for security review".to_string(),
        severity: 0.8,
        identified_at: Timestamp::now(),
        resolved_at: None,
        resolution: None,
    };
    
    let goal = engine.block_goal(goal.id, blocker.clone()).unwrap();
    assert_eq!(goal.status, GoalStatus::Blocked);
    
    // Resolve
    let goal = engine.unblock_goal(
        goal.id,
        blocker.id,
        "Security review passed".to_string(),
    ).unwrap();
    
    assert_eq!(goal.status, GoalStatus::Active);
}

/// SCENARIO 6: Goal Feelings Update
#[test]
fn scenario_06_goal_feelings() {
    let mut engine = PlanningEngine::in_memory();
    
    let goal = engine.create_goal(CreateGoalRequest {
        title: "Important Task".to_string(),
        intention: "Complete urgently".to_string(),
        deadline: Some(Timestamp(Timestamp::now().0 + 86400_000_000_000)),  // Tomorrow
        priority: Some(Priority::Critical),
        ..Default::default()
    }).unwrap();
    
    engine.activate_goal(goal.id).unwrap();
    
    let goal = engine.get_goal(goal.id).unwrap();
    
    // Should have high urgency due to deadline
    assert!(goal.feelings.urgency > 0.5);
    
    // Should have low neglect (just created)
    assert!(goal.feelings.neglect < 0.1);
}

/// SCENARIO 7: Decision Chain Tracing
#[test]
fn scenario_07_decision_chain() {
    let mut engine = PlanningEngine::in_memory();
    
    // Root decision
    let d1 = engine.create_decision(CreateDecisionRequest {
        question: "Language choice?".to_string(),
        ..Default::default()
    }).unwrap();
    
    // Child decision caused by first
    let d2 = engine.create_decision(CreateDecisionRequest {
        question: "Framework choice?".to_string(),
        caused_by: Some(d1.id),
        ..Default::default()
    }).unwrap();
    
    let chain = engine.get_decision_chain(d2.id).unwrap();
    assert_eq!(chain.root, d1.id);
    assert!(chain.descendants.contains(&d2.id));
}

/// SCENARIO 8: Intention Singularity
#[test]
fn scenario_08_intention_singularity() {
    let mut engine = PlanningEngine::in_memory();
    
    // Create multiple active goals
    for i in 0..5 {
        let goal = engine.create_goal(CreateGoalRequest {
            title: format!("Goal {}", i),
            intention: format!("Intention {}", i),
            ..Default::default()
        }).unwrap();
        engine.activate_goal(goal.id).unwrap();
    }
    
    let singularity = engine.get_intention_singularity();
    
    assert_eq!(singularity.goal_positions.len(), 5);
    assert!(!singularity.golden_path.is_empty());
}

/// SCENARIO 9: Goal Dreaming
#[test]
fn scenario_09_goal_dreaming() {
    let mut engine = PlanningEngine::in_memory();
    
    let goal = engine.create_goal(CreateGoalRequest {
        title: "Build Feature".to_string(),
        intention: "New feature".to_string(),
        ..Default::default()
    }).unwrap();
    
    engine.activate_goal(goal.id).unwrap();
    
    let dream = engine.dream_goal(goal.id).unwrap();
    
    assert!(dream.confidence > 0.0);
    assert!(!dream.scenario.vision.is_empty());
}

/// SCENARIO 10: Commitment Entanglement
#[test]
fn scenario_10_commitment_entanglement() {
    let mut engine = PlanningEngine::in_memory();
    
    let c1 = engine.create_commitment(CreateCommitmentRequest {
        promise: Promise {
            description: "Deliver backend".to_string(),
            ..Default::default()
        },
        stakeholder: Stakeholder {
            id: StakeholderId(Uuid::new_v4()),
            name: "PM".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
    
    let c2 = engine.create_commitment(CreateCommitmentRequest {
        promise: Promise {
            description: "Deliver frontend".to_string(),
            ..Default::default()
        },
        stakeholder: Stakeholder {
            id: StakeholderId(Uuid::new_v4()),
            name: "PM".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
    
    // Entangle
    engine.entangle_commitments(
        c1.id,
        c2.id,
        EntanglementType::Parallel,
        0.8,
    ).unwrap();
    
    let c1 = engine.get_commitment(c1.id).unwrap();
    assert_eq!(c1.entanglements.len(), 1);
}

/// SCENARIO 11: Progress Momentum
#[test]
fn scenario_11_progress_momentum() {
    let mut engine = PlanningEngine::in_memory();
    
    let goal = engine.create_goal(CreateGoalRequest {
        title: "Momentum Test".to_string(),
        intention: "Test".to_string(),
        ..Default::default()
    }).unwrap();
    
    engine.activate_goal(goal.id).unwrap();
    
    // Make progress multiple times
    for i in 1..=5 {
        engine.progress_goal(
            goal.id,
            i as f64 * 0.1,
            None,
        ).unwrap();
    }
    
    let goal = engine.get_goal(goal.id).unwrap();
    assert!(goal.physics.momentum > 0.0);
}

/// SCENARIO 12: Goal Reincarnation
#[test]
fn scenario_12_goal_reincarnation() {
    let mut engine = PlanningEngine::in_memory();
    
    // Create and abandon
    let goal = engine.create_goal(CreateGoalRequest {
        title: "Learn Rust".to_string(),
        intention: "Master Rust programming".to_string(),
        ..Default::default()
    }).unwrap();
    
    engine.activate_goal(goal.id).unwrap();
    engine.progress_goal(goal.id, 0.3, None).unwrap();
    engine.abandon_goal(goal.id, "Too busy".to_string()).unwrap();
    
    // Reincarnate
    let reborn = engine.reincarnate_goal(goal.id, ReincarnationUpdates {
        title: Some("Learn Rust (Take 2)".to_string()),
        lessons_learned: Some(vec!["Need more dedicated time".to_string()]),
        ..Default::default()
    }).unwrap();
    
    assert_eq!(reborn.status, GoalStatus::Reborn);
    assert!(reborn.previous_life.is_some());
}

/// SCENARIO 13: Persistence and Recovery
#[test]
fn scenario_13_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.aplan");
    
    // Create and save
    {
        let mut engine = PlanningEngine::open(&path).unwrap();
        
        engine.create_goal(CreateGoalRequest {
            title: "Persistent Goal".to_string(),
            intention: "Should survive".to_string(),
            ..Default::default()
        }).unwrap();
        
        engine.save().unwrap();
    }
    
    // Reload and verify
    {
        let engine = PlanningEngine::open(&path).unwrap();
        let goals: Vec<_> = engine.list_goals(GoalFilter::default());
        
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].title, "Persistent Goal");
    }
}

/// SCENARIO 14: Concurrent Access
#[test]
fn scenario_14_concurrent() {
    use std::sync::Arc;
    use std::thread;
    
    let engine = Arc::new(std::sync::Mutex::new(PlanningEngine::in_memory()));
    
    let handles: Vec<_> = (0..10).map(|i| {
        let engine = Arc::clone(&engine);
        thread::spawn(move || {
            let mut engine = engine.lock().unwrap();
            engine.create_goal(CreateGoalRequest {
                title: format!("Concurrent Goal {}", i),
                intention: "Test".to_string(),
                ..Default::default()
            }).unwrap();
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let engine = engine.lock().unwrap();
    let goals: Vec<_> = engine.list_goals(GoalFilter::default());
    assert_eq!(goals.len(), 10);
}

/// SCENARIO 15: Large Graph Performance
#[test]
fn scenario_15_large_graph() {
    let mut engine = PlanningEngine::in_memory();
    
    // Create 1000 goals
    let start = std::time::Instant::now();
    
    for i in 0..1000 {
        engine.create_goal(CreateGoalRequest {
            title: format!("Goal {}", i),
            intention: format!("Intention {}", i),
            ..Default::default()
        }).unwrap();
    }
    
    let create_time = start.elapsed();
    assert!(create_time.as_millis() < 5000, "Creation took too long");
    
    // Query active goals
    let start = std::time::Instant::now();
    let _ = engine.list_goals(GoalFilter::default());
    let query_time = start.elapsed();
    
    assert!(query_time.as_millis() < 100, "Query took too long");
}

/// SCENARIO 16: MCP Tool Integration
#[test]
fn scenario_16_mcp_tools() {
    let engine = PlanningEngine::in_memory();
    let server = PlanningMcpServer::new(engine);
    
    let tools = server.tools();
    assert_eq!(tools.len(), 12);  // 12 consolidated tools
    
    // Verify tool names
    let tool_names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"planning_goal"));
    assert!(tool_names.contains(&"planning_decision"));
    assert!(tool_names.contains(&"planning_commitment"));
    assert!(tool_names.contains(&"planning_progress"));
    assert!(tool_names.contains(&"planning_singularity"));
}
```

---

## Part 3 Complete

**Covered:**
- SPEC-09: CLI
- SPEC-10: MCP Server
- SPEC-11: Sister Integration
- SPEC-12: Tests

**Next (Part 4):**
- SPEC-13: Performance
- SPEC-14: Security / Hardening
- SPEC-15: Research Paper
- SPEC-16: Inventions Implementation

---

*Document: AGENTIC-PLANNING-SPEC-PART3.md*
