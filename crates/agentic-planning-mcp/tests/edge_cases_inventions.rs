use agentic_planning::PlanningEngine;
use agentic_planning_mcp::{McpError, PlanningMcpServer};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn scenario_16_mcp_tools() {
    let engine = PlanningEngine::in_memory();
    let server = PlanningMcpServer::new(engine);

    let tools = server.tools();
    assert_eq!(tools.len(), 13);

    let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"planning_goal"));
    assert!(names.contains(&"planning_decision"));
    assert!(names.contains(&"planning_commitment"));
    assert!(names.contains(&"planning_progress"));
    assert!(names.contains(&"planning_singularity"));
    assert!(names.contains(&"planning_context_log"));
}

#[tokio::test]
async fn edge_case_unknown_tool() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    let err = server
        .handle_tool("planning_missing", json!({"operation":"list"}))
        .await
        .unwrap_err();

    assert!(matches!(err, McpError::UnknownTool(_)));
}

#[tokio::test]
async fn edge_case_missing_operation() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    let err = server
        .handle_tool("planning_goal", json!({"title":"x"}))
        .await
        .unwrap_err();

    assert!(matches!(err, McpError::MissingOperation));
}

#[tokio::test]
async fn stress_many_goal_creates() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    for i in 0..100 {
        let out = server
            .handle_tool(
                "planning_goal",
                json!({
                    "operation": "create",
                    "title": format!("MCP Goal {i}"),
                    "intention": "mcp stress"
                }),
            )
            .await
            .unwrap();
        assert!(out.get("id").is_some());
    }
}

#[tokio::test]
async fn scenario_17_mcp_federation_lifecycle() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    let goal = server
        .handle_tool(
            "planning_goal",
            json!({
                "operation": "create",
                "title": "Federation Goal",
                "intention": "cross-agent planning"
            }),
        )
        .await
        .unwrap();
    assert!(goal.get("id").is_some());

    let goal_id = goal.get("id").cloned().unwrap();

    let federation = server
        .handle_tool(
            "planning_federate",
            json!({
                "operation": "create",
                "goal_id": goal_id,
                "agent_id": "agent-alpha"
            }),
        )
        .await
        .unwrap();
    assert!(federation.get("id").is_some());

    let federation_id = federation.get("id").cloned().unwrap();
    let joined = server
        .handle_tool(
            "planning_federate",
            json!({
                "operation": "join",
                "federation_id": federation_id,
                "agent_id": "agent-beta"
            }),
        )
        .await
        .unwrap();
    assert!(joined.get("members").is_some());
}

#[tokio::test]
async fn scenario_18_mcp_workspace_lifecycle() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    let created = server
        .handle_tool(
            "planning_workspace",
            json!({
                "operation": "create",
                "name": "local",
                "path": "local.aplan"
            }),
        )
        .await
        .unwrap();
    assert_eq!(
        created.get("status").and_then(|v| v.as_str()),
        Some("created")
    );

    let switched = server
        .handle_tool(
            "planning_workspace",
            json!({
                "operation": "switch",
                "name": "local"
            }),
        )
        .await
        .unwrap();
    assert_eq!(
        switched.get("status").and_then(|v| v.as_str()),
        Some("switched")
    );

    let listed = server
        .handle_tool("planning_workspace", json!({"operation":"list"}))
        .await
        .unwrap();
    assert!(listed.get("workspaces").is_some());
}

#[tokio::test]
async fn scenario_19_mcp_dream_and_metamorphosis_lifecycle() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    let goal = server
        .handle_tool(
            "planning_goal",
            json!({
                "operation": "create",
                "title": "Metamorphosis Goal",
                "intention": "evolve a constrained goal"
            }),
        )
        .await
        .unwrap();
    let goal_id = goal.get("id").and_then(|v| v.as_str()).unwrap().to_string();

    server
        .handle_tool(
            "planning_goal",
            json!({"operation":"activate","id":goal_id}),
        )
        .await
        .unwrap();

    let dream = server
        .handle_tool("planning_dream", json!({"operation":"goal","id":goal_id}))
        .await
        .unwrap();
    let dream_id = dream
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    let interpreted = server
        .handle_tool(
            "planning_dream",
            json!({"operation":"interpret","dream_id":dream_id}),
        )
        .await
        .unwrap();
    assert!(interpreted.get("interpretation").is_some());

    let history = server
        .handle_tool(
            "planning_dream",
            json!({"operation":"history","goal_id":goal_id}),
        )
        .await
        .unwrap();
    assert!(history.as_array().map(|a| !a.is_empty()).unwrap_or(false));

    let bad_approve = server
        .handle_tool(
            "planning_metamorphosis",
            json!({
                "operation":"approve",
                "goal_id":goal_id,
                "change_type":"pivot"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        bad_approve,
        McpError::MissingField("new_direction")
    ));

    let approved = server
        .handle_tool(
            "planning_metamorphosis",
            json!({
                "operation":"approve",
                "goal_id":goal_id,
                "title":"Refined Scope",
                "description":"Narrow to first milestone",
                "change_type":"refinement",
                "clarification":"reduce scope to a thin vertical slice"
            }),
        )
        .await
        .unwrap();
    assert!(approved.get("metamorphosis").is_some());

    let stage = server
        .handle_tool(
            "planning_metamorphosis",
            json!({"operation":"stage","goal_id":goal_id}),
        )
        .await
        .unwrap();
    assert!(stage.get("title").is_some());
}

#[tokio::test]
async fn scenario_20_mcp_counterfactual_chain_consensus() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    let root = server
        .handle_tool(
            "planning_decision",
            json!({"operation":"create","question":"Which architecture?"}),
        )
        .await
        .unwrap();
    let root_id = root.get("id").and_then(|v| v.as_str()).unwrap().to_string();

    server
        .handle_tool(
            "planning_decision",
            json!({
                "operation":"option",
                "id":root_id,
                "name":"modular-monolith",
                "description":"single deployable"
            }),
        )
        .await
        .unwrap();
    server
        .handle_tool(
            "planning_decision",
            json!({
                "operation":"option",
                "id":root_id,
                "name":"microservices",
                "description":"multiple services"
            }),
        )
        .await
        .unwrap();

    let show = server
        .handle_tool(
            "planning_decision",
            json!({"operation":"show","id":root_id}),
        )
        .await
        .unwrap();
    let shadows = show.get("shadows").and_then(|v| v.as_array()).unwrap();
    let path_a = shadows[0]
        .get("path")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();
    let path_b = shadows[1]
        .get("path")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    let child = server
        .handle_tool(
            "planning_decision",
            json!({
                "operation":"create",
                "question":"Which persistence model?",
                "caused_by":root_id
            }),
        )
        .await
        .unwrap();
    let child_id = child
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    let chain_roots = server
        .handle_tool(
            "planning_chain",
            json!({"operation":"roots","decision_id":child_id}),
        )
        .await
        .unwrap();
    assert_eq!(
        chain_roots.get("root").and_then(|v| v.as_str()),
        Some(root_id.as_str())
    );

    let projected = server
        .handle_tool(
            "planning_counterfactual",
            json!({
                "operation":"project",
                "decision_id":root_id,
                "path_id":path_a
            }),
        )
        .await
        .unwrap();
    assert!(projected.get("projected_at").is_some());
    assert!(projected.get("final_state").is_some());

    let compared = server
        .handle_tool(
            "planning_counterfactual",
            json!({
                "operation":"compare",
                "decision_id":root_id,
                "path_a":path_a,
                "path_b":path_b
            }),
        )
        .await
        .unwrap();
    assert!(compared.get("path_a").is_some());
    assert!(compared.get("path_b").is_some());

    let invalid_path = Uuid::new_v4().to_string();
    let err = server
        .handle_tool(
            "planning_counterfactual",
            json!({
                "operation":"project",
                "decision_id":root_id,
                "path_id":invalid_path
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        McpError::Core(agentic_planning::Error::PathNotFound(_))
    ));

    server
        .handle_tool(
            "planning_consensus",
            json!({
                "operation":"start",
                "decision_id":root_id,
                "stakeholders":["architect","pm"]
            }),
        )
        .await
        .unwrap();
    server
        .handle_tool(
            "planning_consensus",
            json!({
                "operation":"round",
                "decision_id":root_id,
                "stakeholder":"architect",
                "position":"prefer modular-monolith for faster execution"
            }),
        )
        .await
        .unwrap();
    server
        .handle_tool(
            "planning_consensus",
            json!({
                "operation":"vote",
                "decision_id":root_id,
                "stakeholder":"architect",
                "option":"modular-monolith"
            }),
        )
        .await
        .unwrap();
    server
        .handle_tool(
            "planning_consensus",
            json!({
                "operation":"vote",
                "decision_id":root_id,
                "stakeholder":"pm",
                "option":"modular-monolith"
            }),
        )
        .await
        .unwrap();

    let synthesized = server
        .handle_tool(
            "planning_consensus",
            json!({"operation":"synthesize","decision_id":root_id}),
        )
        .await
        .unwrap();
    assert_eq!(
        synthesized.get("recommendation").and_then(|v| v.as_str()),
        Some("modular-monolith")
    );

    let crystallized = server
        .handle_tool(
            "planning_consensus",
            json!({"operation":"crystallize","decision_id":root_id}),
        )
        .await
        .unwrap();
    assert_eq!(
        crystallized.get("status").and_then(|v| v.as_str()),
        Some("Crystallized")
    );
}

#[tokio::test]
async fn scenario_21_mcp_workspace_merge() {
    let temp = tempfile::tempdir().unwrap();
    let source_path = temp.path().join("source.aplan");
    let target_path = temp.path().join("target.aplan");

    {
        let mut source = PlanningEngine::open(&source_path).unwrap();
        source
            .create_goal(agentic_planning::CreateGoalRequest {
                title: "Source Goal".to_string(),
                intention: "from source".to_string(),
                ..Default::default()
            })
            .unwrap();
        source.save().unwrap();
    }

    {
        let mut target = PlanningEngine::open(&target_path).unwrap();
        target
            .create_goal(agentic_planning::CreateGoalRequest {
                title: "Target Goal".to_string(),
                intention: "from target".to_string(),
                ..Default::default()
            })
            .unwrap();
        target.save().unwrap();
    }

    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);
    server
        .handle_tool(
            "planning_workspace",
            json!({
                "operation":"create",
                "name":"source",
                "path":source_path.to_string_lossy()
            }),
        )
        .await
        .unwrap();
    server
        .handle_tool(
            "planning_workspace",
            json!({
                "operation":"create",
                "name":"target",
                "path":target_path.to_string_lossy()
            }),
        )
        .await
        .unwrap();

    let merged = server
        .handle_tool(
            "planning_workspace",
            json!({
                "operation":"merge",
                "source":"source",
                "target":"target"
            }),
        )
        .await
        .unwrap();
    assert_eq!(
        merged.get("status").and_then(|v| v.as_str()),
        Some("merged")
    );
    assert!(merged.get("report").is_some());

    let target = PlanningEngine::open(&target_path).unwrap();
    assert_eq!(target.goal_count(), 2);
}

#[tokio::test]
async fn scenario_22_strict_validation_no_silent_fallback() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    let bad_priority = server
        .handle_tool(
            "planning_goal",
            json!({
                "operation":"create",
                "title":"Bad Priority",
                "intention":"validation",
                "priority":"urgent-plus-plus"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        bad_priority,
        McpError::InvalidValue {
            field: "priority",
            ..
        }
    ));

    let goal = server
        .handle_tool(
            "planning_goal",
            json!({
                "operation":"create",
                "title":"Validation Goal",
                "intention":"test numeric strictness"
            }),
        )
        .await
        .unwrap();
    let goal_id = goal.get("id").and_then(|v| v.as_str()).unwrap().to_string();

    let bad_severity = server
        .handle_tool(
            "planning_goal",
            json!({
                "operation":"block",
                "id":goal_id,
                "blocker":"broken type",
                "severity":"high"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        bad_severity,
        McpError::InvalidValue {
            field: "severity",
            ..
        }
    ));

    let c1 = server
        .handle_tool(
            "planning_commitment",
            json!({
                "operation":"create",
                "promise":"Deliver A",
                "stakeholder":"Ops"
            }),
        )
        .await
        .unwrap();
    let c2 = server
        .handle_tool(
            "planning_commitment",
            json!({
                "operation":"create",
                "promise":"Deliver B",
                "stakeholder":"Ops"
            }),
        )
        .await
        .unwrap();

    let bad_entanglement = server
        .handle_tool(
            "planning_commitment",
            json!({
                "operation":"entangle",
                "id": c1.get("id").unwrap(),
                "commitment_b": c2.get("id").unwrap(),
                "entanglement_type":"quantum-loop"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        bad_entanglement,
        McpError::InvalidValue {
            field: "entanglement_type",
            ..
        }
    ));

    let bad_due_soon = server
        .handle_tool(
            "planning_commitment",
            json!({
                "operation":"due_soon",
                "within_days":"soon"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        bad_due_soon,
        McpError::InvalidValue {
            field: "within_days",
            ..
        }
    ));

    let bad_factor = server
        .handle_tool(
            "planning_metamorphosis",
            json!({
                "operation":"approve",
                "goal_id":goal_id,
                "change_type":"expansion",
                "factor":"large"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        bad_factor,
        McpError::InvalidValue {
            field: "factor",
            ..
        }
    ));
}
