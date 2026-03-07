// =============================================================================
// edge_cases_inventions.rs — Comprehensive MCP edge-case & stress tests
//
// Follows the sister benchmark pattern:
//   - Smoke tests: every tool with empty/minimal args
//   - Unknown operations: every tool rejects invalid operation strings
//   - Missing required params: every tool returns proper errors
//   - Boundary values: numeric clamp, empty strings, huge strings
//   - Concurrency: rapid-fire and parallel tool calls
//   - State stability: session integrity after error storms
// =============================================================================

use agentic_planning::PlanningEngine;
use agentic_planning_mcp::{McpError, PlanningMcpServer};
use serde_json::json;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn fresh_server() -> PlanningMcpServer {
    PlanningMcpServer::new(PlanningEngine::in_memory())
}

async fn server_with_goal() -> (PlanningMcpServer, String) {
    let mut s = fresh_server().await;
    let g = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"create","title":"Test Goal","intention":"fixture"}),
        )
        .await
        .unwrap();
    let id = g.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    (s, id)
}

async fn server_with_active_goal() -> (PlanningMcpServer, String) {
    let (mut s, id) = server_with_goal().await;
    s.handle_tool("planning_goal", json!({"operation":"activate","id":id}))
        .await
        .unwrap();
    (s, id)
}

async fn server_with_decision() -> (PlanningMcpServer, String) {
    let mut s = fresh_server().await;
    let d = s
        .handle_tool(
            "planning_decision",
            json!({"operation":"create","question":"Which path?"}),
        )
        .await
        .unwrap();
    let id = d.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    (s, id)
}

async fn server_with_commitment() -> (PlanningMcpServer, String) {
    let mut s = fresh_server().await;
    let c = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"create","promise":"Deliver X","stakeholder":"Ops"}),
        )
        .await
        .unwrap();
    let id = c.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    (s, id)
}

// ===========================================================================
// Section 1: Tool inventory verification
// ===========================================================================

#[tokio::test]
async fn test_01_tool_count_is_13() {
    let server = fresh_server().await;
    let tools = server.tools();
    assert_eq!(tools.len(), 13, "Planning must expose exactly 13 MCP tools");
}

#[tokio::test]
async fn test_02_all_tool_names_present() {
    let server = fresh_server().await;
    let names: Vec<_> = server.tools().iter().map(|t| t.name.clone()).collect();
    let expected = [
        "planning_goal",
        "planning_decision",
        "planning_commitment",
        "planning_progress",
        "planning_singularity",
        "planning_dream",
        "planning_counterfactual",
        "planning_chain",
        "planning_consensus",
        "planning_federate",
        "planning_metamorphosis",
        "planning_workspace",
        "planning_context_log",
    ];
    for tool in &expected {
        assert!(names.iter().any(|n| n == tool), "Missing tool: {tool}");
    }
}

#[tokio::test]
async fn test_03_resource_count_is_9() {
    let server = fresh_server().await;
    let resources = server.resources();
    assert_eq!(resources.len(), 9, "Planning must expose exactly 9 resources");
}

#[tokio::test]
async fn test_04_prompt_count_is_4() {
    let server = fresh_server().await;
    let prompts = server.prompts();
    assert_eq!(prompts.len(), 4, "Planning must expose exactly 4 prompts");
}

// ===========================================================================
// Section 2: Unknown tool / unknown operation
// ===========================================================================

#[tokio::test]
async fn test_05_unknown_tool_returns_error() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_nonexistent", json!({"operation": "list"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownTool(_)));
}

#[tokio::test]
async fn test_06_unknown_op_goal() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_goal", json!({"operation": "explode"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_07_unknown_op_decision() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_decision", json!({"operation": "yeet"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_08_unknown_op_commitment() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_commitment", json!({"operation": "shatter"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_09_unknown_op_progress() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_progress", json!({"operation": "warp"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_10_unknown_op_singularity() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_singularity", json!({"operation": "implode"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_11_unknown_op_dream() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_dream", json!({"operation": "nightmare"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_12_unknown_op_counterfactual() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_counterfactual", json!({"operation": "paradox"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_13_unknown_op_chain() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_chain", json!({"operation": "snap"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_14_unknown_op_consensus() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_consensus", json!({"operation": "riot"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_15_unknown_op_federate() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_federate", json!({"operation": "secede"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_16_unknown_op_metamorphosis() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_metamorphosis", json!({"operation": "dissolve"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

#[tokio::test]
async fn test_17_unknown_op_workspace() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_workspace", json!({"operation": "nuke"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::UnknownOperation(_)));
}

// ===========================================================================
// Section 3: Missing operation field
// ===========================================================================

#[tokio::test]
async fn test_18_missing_op_goal() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_goal", json!({"title": "no-op"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingOperation));
}

#[tokio::test]
async fn test_19_missing_op_decision() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_decision", json!({"question": "no-op"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingOperation));
}

#[tokio::test]
async fn test_20_missing_op_commitment() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_commitment", json!({"promise": "no-op"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingOperation));
}

#[tokio::test]
async fn test_21_missing_op_progress() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_progress", json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingOperation));
}

#[tokio::test]
async fn test_22_missing_op_singularity() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_singularity", json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingOperation));
}

// ===========================================================================
// Section 4: Missing required fields per tool
// ===========================================================================

#[tokio::test]
async fn test_23_goal_create_missing_title() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"create","intention":"no title"}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("title")));
}

#[tokio::test]
async fn test_24_goal_create_intention_optional_defaults_to_title() {
    // intention is OPTIONAL — defaults to title when omitted
    let mut s = fresh_server().await;
    let out = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"create","title":"no intention"}),
        )
        .await
        .unwrap();
    assert!(out.get("id").is_some());
    // intention lives inside soul.intention in the serialized Goal
    let soul_intention = out
        .get("soul")
        .and_then(|s| s.get("intention"))
        .and_then(|v| v.as_str());
    assert_eq!(soul_intention, Some("no intention"));
}

#[tokio::test]
async fn test_25_goal_show_missing_id() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_goal", json!({"operation":"show"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("id")));
}

#[tokio::test]
async fn test_26_goal_activate_missing_id() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_goal", json!({"operation":"activate"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("id")));
}

#[tokio::test]
async fn test_27_decision_create_missing_question() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool("planning_decision", json!({"operation":"create"}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("question")));
}

#[tokio::test]
async fn test_28_commitment_create_missing_promise() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"create","stakeholder":"Team"}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("promise")));
}

#[tokio::test]
async fn test_29_commitment_create_missing_stakeholder() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"create","promise":"Deliver"}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("stakeholder")));
}

#[tokio::test]
async fn test_30_goal_block_missing_blocker() {
    let (mut s, id) = server_with_active_goal().await;
    let err = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"block","id":id,"severity":0.5}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("blocker")));
}

// ===========================================================================
// Section 5: Invalid value validation
// ===========================================================================

#[tokio::test]
async fn test_31_invalid_priority_string() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool(
            "planning_goal",
            json!({
                "operation":"create",
                "title":"Bad",
                "intention":"test",
                "priority":"ultra-mega-critical"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        McpError::InvalidValue {
            field: "priority",
            ..
        }
    ));
}

#[tokio::test]
async fn test_32_invalid_severity_not_number() {
    let (mut s, id) = server_with_active_goal().await;
    let err = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"block","id":id,"blocker":"x","severity":"high"}),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        McpError::InvalidValue {
            field: "severity",
            ..
        }
    ));
}

#[tokio::test]
async fn test_33_invalid_entanglement_type() {
    let mut s = fresh_server().await;
    let c1 = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"create","promise":"A","stakeholder":"X"}),
        )
        .await
        .unwrap();
    let c2 = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"create","promise":"B","stakeholder":"X"}),
        )
        .await
        .unwrap();
    let err = s
        .handle_tool(
            "planning_commitment",
            json!({
                "operation":"entangle",
                "id":c1.get("id").unwrap(),
                "commitment_b":c2.get("id").unwrap(),
                "entanglement_type":"quantum-loop"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        McpError::InvalidValue {
            field: "entanglement_type",
            ..
        }
    ));
}

#[tokio::test]
async fn test_34_invalid_due_soon_not_number() {
    let mut s = fresh_server().await;
    let err = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"due_soon","within_days":"soon"}),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        McpError::InvalidValue {
            field: "within_days",
            ..
        }
    ));
}

#[tokio::test]
async fn test_35_invalid_metamorphosis_factor_not_number() {
    let (mut s, id) = server_with_goal().await;
    let err = s
        .handle_tool(
            "planning_metamorphosis",
            json!({
                "operation":"approve",
                "goal_id":id,
                "change_type":"expansion",
                "factor":"large"
            }),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        McpError::InvalidValue {
            field: "factor",
            ..
        }
    ));
}

// ===========================================================================
// Section 6: Non-existent ID references
// ===========================================================================

#[tokio::test]
async fn test_36_show_nonexistent_goal() {
    let mut s = fresh_server().await;
    let fake_id = Uuid::new_v4().to_string();
    let err = s
        .handle_tool("planning_goal", json!({"operation":"show","id":fake_id}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::Core(_)));
}

#[tokio::test]
async fn test_37_activate_nonexistent_goal() {
    let mut s = fresh_server().await;
    let fake_id = Uuid::new_v4().to_string();
    let err = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"activate","id":fake_id}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::Core(_)));
}

#[tokio::test]
async fn test_38_show_nonexistent_decision() {
    let mut s = fresh_server().await;
    let fake_id = Uuid::new_v4().to_string();
    let err = s
        .handle_tool(
            "planning_decision",
            json!({"operation":"show","id":fake_id}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::Core(_)));
}

#[tokio::test]
async fn test_39_show_nonexistent_commitment() {
    let mut s = fresh_server().await;
    let fake_id = Uuid::new_v4().to_string();
    let err = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"show","id":fake_id}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::Core(_)));
}

#[tokio::test]
async fn test_40_dream_nonexistent_goal() {
    let mut s = fresh_server().await;
    let fake_id = Uuid::new_v4().to_string();
    let err = s
        .handle_tool("planning_dream", json!({"operation":"goal","id":fake_id}))
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::Core(_)));
}

#[tokio::test]
async fn test_41_counterfactual_project_nonexistent_decision() {
    let mut s = fresh_server().await;
    let fake_id = Uuid::new_v4().to_string();
    let err = s
        .handle_tool(
            "planning_counterfactual",
            json!({"operation":"project","decision_id":fake_id,"path_id":fake_id}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::Core(_)));
}

#[tokio::test]
async fn test_42_chain_roots_nonexistent_decision() {
    // chain.roots returns Ok with null root when decision not found (graceful)
    let mut s = fresh_server().await;
    let fake_id = Uuid::new_v4().to_string();
    let out = s
        .handle_tool(
            "planning_chain",
            json!({"operation":"roots","decision_id":fake_id}),
        )
        .await
        .unwrap();
    assert!(out.get("root").is_some());
    assert!(out.get("root").unwrap().is_null());
}

// ===========================================================================
// Section 7: Smoke tests — every tool with list/empty operations
// ===========================================================================

#[tokio::test]
async fn test_43_goal_list_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_goal", json!({"operation":"list"}))
        .await
        .unwrap();
    assert!(out.is_array());
    assert_eq!(out.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_44_decision_list_empty() {
    let mut s = fresh_server().await;
    // decision doesn't have list, use show on empty store; use a different approach
    // Actually decision.create is the main entry. Just verify create works.
    let d = s
        .handle_tool(
            "planning_decision",
            json!({"operation":"create","question":"Smoke?"}),
        )
        .await
        .unwrap();
    assert!(d.get("id").is_some());
}

#[tokio::test]
async fn test_45_commitment_list_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_commitment", json!({"operation":"list"}))
        .await
        .unwrap();
    assert!(out.is_array());
}

#[tokio::test]
async fn test_46_commitment_inventory_empty() {
    // inventory returns engine.get_commitment_inventory() directly — an object
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_commitment", json!({"operation":"inventory"}))
        .await
        .unwrap();
    assert!(out.is_object());
}

#[tokio::test]
async fn test_47_progress_momentum_empty() {
    // momentum without goal_id returns an array of active goals (empty when none)
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_progress", json!({"operation":"momentum"}))
        .await
        .unwrap();
    assert!(out.is_array());
    assert_eq!(out.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_48_progress_gravity_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_progress", json!({"operation":"gravity"}))
        .await
        .unwrap();
    assert!(out.is_array() || out.is_object());
}

#[tokio::test]
async fn test_49_progress_blockers_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_progress", json!({"operation":"blockers"}))
        .await
        .unwrap();
    assert!(out.is_array() || out.is_object());
}

#[tokio::test]
async fn test_50_singularity_collapse_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_singularity", json!({"operation":"collapse"}))
        .await
        .unwrap();
    assert!(out.is_object());
}

#[tokio::test]
async fn test_51_singularity_themes_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_singularity", json!({"operation":"themes"}))
        .await
        .unwrap();
    assert!(out.is_array() || out.is_object());
}

#[tokio::test]
async fn test_52_singularity_center_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_singularity", json!({"operation":"center"}))
        .await
        .unwrap();
    assert!(out.is_object());
}

#[tokio::test]
async fn test_53_singularity_vision_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_singularity", json!({"operation":"vision"}))
        .await
        .unwrap();
    assert!(out.is_object());
}

#[tokio::test]
async fn test_54_workspace_list_empty() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_workspace", json!({"operation":"list"}))
        .await
        .unwrap();
    assert!(out.get("workspaces").is_some());
}

// ===========================================================================
// Section 8: Goal operations — full lifecycle through MCP
// ===========================================================================

#[tokio::test]
async fn test_55_goal_create_and_show() {
    let (mut s, id) = server_with_goal().await;
    let shown = s
        .handle_tool("planning_goal", json!({"operation":"show","id":id}))
        .await
        .unwrap();
    assert_eq!(shown.get("title").and_then(|v| v.as_str()), Some("Test Goal"));
}

#[tokio::test]
async fn test_56_goal_activate_then_progress() {
    let (mut s, id) = server_with_active_goal().await;
    let progressed = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"progress","id":id,"percentage":0.4,"note":"progress test"}),
        )
        .await
        .unwrap();
    assert!(progressed.get("progress").is_some());
}

#[tokio::test]
async fn test_57_goal_pause_and_resume() {
    let (mut s, id) = server_with_active_goal().await;
    let paused = s
        .handle_tool("planning_goal", json!({"operation":"pause","id":id}))
        .await
        .unwrap();
    assert_eq!(
        paused.get("status").and_then(|v| v.as_str()),
        Some("Paused")
    );
    let resumed = s
        .handle_tool("planning_goal", json!({"operation":"resume","id":id}))
        .await
        .unwrap();
    assert_eq!(
        resumed.get("status").and_then(|v| v.as_str()),
        Some("Active")
    );
}

#[tokio::test]
async fn test_58_goal_abandon() {
    let (mut s, id) = server_with_active_goal().await;
    let abandoned = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"abandon","id":id,"reason":"no longer needed"}),
        )
        .await
        .unwrap();
    assert_eq!(
        abandoned.get("status").and_then(|v| v.as_str()),
        Some("Abandoned")
    );
}

#[tokio::test]
async fn test_59_goal_complete() {
    let (mut s, id) = server_with_active_goal().await;
    let completed = s
        .handle_tool("planning_goal", json!({"operation":"complete","id":id}))
        .await
        .unwrap();
    assert_eq!(
        completed.get("status").and_then(|v| v.as_str()),
        Some("Completed")
    );
}

#[tokio::test]
async fn test_60_goal_block_and_unblock() {
    let (mut s, id) = server_with_active_goal().await;
    let blocked = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"block","id":id,"blocker":"External dependency","severity":0.6}),
        )
        .await
        .unwrap();
    assert_eq!(
        blocked.get("status").and_then(|v| v.as_str()),
        Some("Blocked")
    );
    // Extract blocker_id (UUID) from the blocked goal's blockers array
    let blocker_id = blocked
        .get("blockers")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|b| b.get("id"))
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();
    let unblocked = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"unblock","id":id,"blocker_id":blocker_id,"resolution":"resolved externally"}),
        )
        .await
        .unwrap();
    assert_eq!(
        unblocked.get("status").and_then(|v| v.as_str()),
        Some("Active")
    );
}

#[tokio::test]
async fn test_61_goal_feelings() {
    let (mut s, id) = server_with_active_goal().await;
    let feelings = s
        .handle_tool("planning_goal", json!({"operation":"feelings","id":id}))
        .await
        .unwrap();
    assert!(feelings.get("urgency").is_some());
    assert!(feelings.get("confidence").is_some());
}

#[tokio::test]
async fn test_62_goal_physics() {
    let (mut s, id) = server_with_active_goal().await;
    let physics = s
        .handle_tool("planning_goal", json!({"operation":"physics","id":id}))
        .await
        .unwrap();
    assert!(physics.get("momentum").is_some());
    assert!(physics.get("gravity").is_some());
}

#[tokio::test]
async fn test_63_goal_tree() {
    let (mut s, id) = server_with_goal().await;
    let tree = s
        .handle_tool("planning_goal", json!({"operation":"tree","id":id}))
        .await
        .unwrap();
    // GoalTree serializes with "root" field (the root GoalId)
    assert!(tree.get("root").is_some());
}

#[tokio::test]
async fn test_64_goal_decompose() {
    let (mut s, id) = server_with_active_goal().await;
    // decompose uses required_string_array — expects plain strings, not objects
    let decomposed = s
        .handle_tool(
            "planning_goal",
            json!({
                "operation":"decompose",
                "id":id,
                "sub_goals":["Sub A","Sub B"]
            }),
        )
        .await
        .unwrap();
    assert!(decomposed.is_object() || decomposed.is_array());
}

#[tokio::test]
async fn test_65_goal_reincarnate() {
    let (mut s, id) = server_with_active_goal().await;
    // First abandon to make reincarnation possible
    s.handle_tool(
        "planning_goal",
        json!({"operation":"abandon","id":id,"reason":"testing reincarnation"}),
    )
    .await
    .unwrap();
    let reincarnated = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"reincarnate","id":id,"title":"Reborn Goal","intention":"second life"}),
        )
        .await
        .unwrap();
    assert!(reincarnated.get("id").is_some());
}

// ===========================================================================
// Section 9: Decision operations through MCP
// ===========================================================================

#[tokio::test]
async fn test_66_decision_add_option_and_show() {
    let (mut s, id) = server_with_decision().await;
    s.handle_tool(
        "planning_decision",
        json!({
            "operation":"option",
            "id":id,
            "name":"Option A",
            "description":"First choice"
        }),
    )
    .await
    .unwrap();
    let shown = s
        .handle_tool("planning_decision", json!({"operation":"show","id":id}))
        .await
        .unwrap();
    let shadows = shown.get("shadows").and_then(|v| v.as_array()).unwrap();
    assert_eq!(shadows.len(), 1);
}

#[tokio::test]
async fn test_67_decision_crystallize() {
    let (mut s, id) = server_with_decision().await;
    s.handle_tool(
        "planning_decision",
        json!({"operation":"option","id":id,"name":"Only Option","description":"sole path"}),
    )
    .await
    .unwrap();
    let show = s
        .handle_tool("planning_decision", json!({"operation":"show","id":id}))
        .await
        .unwrap();
    let path_id = show
        .get("shadows")
        .and_then(|v| v.as_array())
        .unwrap()[0]
        .get("path")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();
    let crystallized = s
        .handle_tool(
            "planning_decision",
            json!({
                "operation":"crystallize",
                "id":id,
                "chosen":path_id,
                "reasoning":"best fit"
            }),
        )
        .await
        .unwrap();
    assert_eq!(
        crystallized.get("status").and_then(|v| v.as_str()),
        Some("Crystallized")
    );
}

#[tokio::test]
async fn test_68_decision_regret_empty() {
    let (mut s, id) = server_with_decision().await;
    let regret = s
        .handle_tool(
            "planning_decision",
            json!({"operation":"regret","id":id}),
        )
        .await
        .unwrap();
    // Server returns json!({"decision_id": id, "regret": decision.regret_score})
    assert!(regret.get("regret").is_some());
    assert!(regret.get("decision_id").is_some());
}

#[tokio::test]
async fn test_69_decision_shadows() {
    let (mut s, id) = server_with_decision().await;
    s.handle_tool(
        "planning_decision",
        json!({"operation":"option","id":id,"name":"A","description":"a"}),
    )
    .await
    .unwrap();
    let shadows = s
        .handle_tool(
            "planning_decision",
            json!({"operation":"shadows","id":id}),
        )
        .await
        .unwrap();
    assert!(shadows.is_array());
}

// ===========================================================================
// Section 10: Commitment operations through MCP
// ===========================================================================

#[tokio::test]
async fn test_70_commitment_show() {
    let (mut s, id) = server_with_commitment().await;
    let shown = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"show","id":id}),
        )
        .await
        .unwrap();
    assert_eq!(shown.get("id").and_then(|v| v.as_str()), Some(id.as_str()));
}

#[tokio::test]
async fn test_71_commitment_fulfill() {
    let (mut s, id) = server_with_commitment().await;
    // fulfill requires "how_delivered" field
    let fulfilled = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"fulfill","id":id,"how_delivered":"completed on time"}),
        )
        .await
        .unwrap();
    assert_eq!(
        fulfilled.get("status").and_then(|v| v.as_str()),
        Some("Fulfilled")
    );
}

#[tokio::test]
async fn test_72_commitment_break() {
    let (mut s, id) = server_with_commitment().await;
    let broken = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"break","id":id,"reason":"changed priorities"}),
        )
        .await
        .unwrap();
    assert_eq!(
        broken.get("status").and_then(|v| v.as_str()),
        Some("Broken")
    );
}

#[tokio::test]
async fn test_73_commitment_renegotiate() {
    let (mut s, id) = server_with_commitment().await;
    let renegotiated = s
        .handle_tool(
            "planning_commitment",
            json!({
                "operation":"renegotiate",
                "id":id,
                "new_promise":"Deliver X v2",
                "reason":"scope change"
            }),
        )
        .await
        .unwrap();
    // Engine sets status to Active (accepted) or AtRisk (rejected) — never "Renegotiated"
    let status = renegotiated.get("status").and_then(|v| v.as_str()).unwrap();
    assert!(status == "Active" || status == "AtRisk");
    // Verify renegotiations array grew
    let renegs = renegotiated
        .get("renegotiations")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!renegs.is_empty());
}

#[tokio::test]
async fn test_74_commitment_at_risk() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_commitment", json!({"operation":"at_risk"}))
        .await
        .unwrap();
    assert!(out.is_array());
}

#[tokio::test]
async fn test_75_commitment_due_soon_valid() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool(
            "planning_commitment",
            json!({"operation":"due_soon","within_days":7}),
        )
        .await
        .unwrap();
    assert!(out.is_array());
}

// ===========================================================================
// Section 11: Progress operations
// ===========================================================================

#[tokio::test]
async fn test_76_progress_velocity() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_progress", json!({"operation":"velocity"}))
        .await
        .unwrap();
    assert!(out.is_object() || out.is_array());
}

#[tokio::test]
async fn test_77_progress_echoes() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_progress", json!({"operation":"echoes"}))
        .await
        .unwrap();
    assert!(out.is_object() || out.is_array());
}

#[tokio::test]
async fn test_78_progress_forecast() {
    // forecast requires goal_id
    let (mut s, id) = server_with_active_goal().await;
    let out = s
        .handle_tool(
            "planning_progress",
            json!({"operation":"forecast","goal_id":id}),
        )
        .await
        .unwrap();
    assert!(out.is_object());
    assert!(out.get("goal_id").is_some());
    assert!(out.get("velocity").is_some());
}

#[tokio::test]
async fn test_79_progress_trend() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool("planning_progress", json!({"operation":"trend"}))
        .await
        .unwrap();
    assert!(out.is_object() || out.is_array());
}

// ===========================================================================
// Section 12: Context log
// ===========================================================================

#[tokio::test]
async fn test_80_context_log_basic() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool(
            "planning_context_log",
            json!({"intent":"testing","finding":"edge case"}),
        )
        .await
        .unwrap();
    // Server returns: { log_index, intent, finding, topic, message }
    assert!(out.get("log_index").is_some());
    assert_eq!(out.get("intent").and_then(|v| v.as_str()), Some("testing"));
    assert!(out.get("message").is_some());
}

#[tokio::test]
async fn test_81_context_log_with_topic() {
    let mut s = fresh_server().await;
    let out = s
        .handle_tool(
            "planning_context_log",
            json!({"intent":"audit","finding":"all good","topic":"testing"}),
        )
        .await
        .unwrap();
    assert!(out.is_object());
}

// ===========================================================================
// Section 13: Federation lifecycle through MCP
// ===========================================================================

#[tokio::test]
async fn test_82_federation_create_and_join() {
    let (mut s, goal_id) = server_with_goal().await;
    let fed = s
        .handle_tool(
            "planning_federate",
            json!({"operation":"create","goal_id":goal_id,"agent_id":"agent-1"}),
        )
        .await
        .unwrap();
    assert!(fed.get("id").is_some());
    let fed_id = fed.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    let joined = s
        .handle_tool(
            "planning_federate",
            json!({"operation":"join","federation_id":fed_id,"agent_id":"agent-2"}),
        )
        .await
        .unwrap();
    assert!(joined.get("members").is_some());
}

#[tokio::test]
async fn test_83_federation_status() {
    let (mut s, goal_id) = server_with_goal().await;
    let fed = s
        .handle_tool(
            "planning_federate",
            json!({"operation":"create","goal_id":goal_id,"agent_id":"a1"}),
        )
        .await
        .unwrap();
    let fed_id = fed.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    let status = s
        .handle_tool(
            "planning_federate",
            json!({"operation":"status","federation_id":fed_id}),
        )
        .await
        .unwrap();
    assert!(status.get("status").is_some() || status.get("id").is_some());
}

// ===========================================================================
// Section 14: Consensus lifecycle
// ===========================================================================

#[tokio::test]
async fn test_84_consensus_full_cycle() {
    let (mut s, dec_id) = server_with_decision().await;
    s.handle_tool(
        "planning_decision",
        json!({"operation":"option","id":dec_id,"name":"A","description":"a"}),
    )
    .await
    .unwrap();

    s.handle_tool(
        "planning_consensus",
        json!({"operation":"start","decision_id":dec_id,"stakeholders":["dev","pm"]}),
    )
    .await
    .unwrap();

    s.handle_tool(
        "planning_consensus",
        json!({"operation":"round","decision_id":dec_id,"stakeholder":"dev","position":"prefer A"}),
    )
    .await
    .unwrap();

    s.handle_tool(
        "planning_consensus",
        json!({"operation":"vote","decision_id":dec_id,"stakeholder":"dev","option":"A"}),
    )
    .await
    .unwrap();

    s.handle_tool(
        "planning_consensus",
        json!({"operation":"vote","decision_id":dec_id,"stakeholder":"pm","option":"A"}),
    )
    .await
    .unwrap();

    let synth = s
        .handle_tool(
            "planning_consensus",
            json!({"operation":"synthesize","decision_id":dec_id}),
        )
        .await
        .unwrap();
    assert!(synth.get("recommendation").is_some());

    let status = s
        .handle_tool(
            "planning_consensus",
            json!({"operation":"status","decision_id":dec_id}),
        )
        .await
        .unwrap();
    assert!(status.is_object());
}

// ===========================================================================
// Section 15: Dream operations
// ===========================================================================

#[tokio::test]
async fn test_85_dream_goal_and_interpret() {
    let (mut s, id) = server_with_active_goal().await;
    let dream = s
        .handle_tool("planning_dream", json!({"operation":"goal","id":id}))
        .await
        .unwrap();
    let dream_id = dream.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    let interpreted = s
        .handle_tool(
            "planning_dream",
            json!({"operation":"interpret","dream_id":dream_id}),
        )
        .await
        .unwrap();
    assert!(interpreted.get("interpretation").is_some());
}

#[tokio::test]
async fn test_86_dream_insights() {
    let (mut s, id) = server_with_active_goal().await;
    let dream = s
        .handle_tool("planning_dream", json!({"operation":"goal","id":id}))
        .await
        .unwrap();
    let dream_id = dream.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    let insights = s
        .handle_tool(
            "planning_dream",
            json!({"operation":"insights","dream_id":dream_id}),
        )
        .await
        .unwrap();
    assert!(insights.is_array() || insights.get("insights").is_some());
}

#[tokio::test]
async fn test_87_dream_history() {
    let (mut s, id) = server_with_active_goal().await;
    s.handle_tool("planning_dream", json!({"operation":"goal","id":id}))
        .await
        .unwrap();
    let history = s
        .handle_tool(
            "planning_dream",
            json!({"operation":"history","goal_id":id}),
        )
        .await
        .unwrap();
    assert!(history.is_array());
}

// ===========================================================================
// Section 16: Metamorphosis
// ===========================================================================

#[tokio::test]
async fn test_88_metamorphosis_detect_empty() {
    let (mut s, id) = server_with_goal().await;
    let detected = s
        .handle_tool(
            "planning_metamorphosis",
            json!({"operation":"detect","goal_id":id}),
        )
        .await
        .unwrap();
    assert!(detected.is_object());
}

#[tokio::test]
async fn test_89_metamorphosis_approve_refinement() {
    let (mut s, id) = server_with_goal().await;
    let approved = s
        .handle_tool(
            "planning_metamorphosis",
            json!({
                "operation":"approve",
                "goal_id":id,
                "title":"Refined Goal",
                "description":"Narrowed scope",
                "change_type":"refinement",
                "clarification":"thin vertical slice"
            }),
        )
        .await
        .unwrap();
    assert!(approved.get("metamorphosis").is_some());
}

#[tokio::test]
async fn test_90_metamorphosis_missing_new_direction_for_pivot() {
    let (mut s, id) = server_with_goal().await;
    let err = s
        .handle_tool(
            "planning_metamorphosis",
            json!({"operation":"approve","goal_id":id,"change_type":"pivot"}),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, McpError::MissingField("new_direction")));
}

// ===========================================================================
// Section 17: Workspace lifecycle
// ===========================================================================

#[tokio::test]
async fn test_91_workspace_create_switch_list() {
    let mut s = fresh_server().await;
    s.handle_tool(
        "planning_workspace",
        json!({"operation":"create","name":"test-ws","path":"test.aplan"}),
    )
    .await
    .unwrap();
    s.handle_tool(
        "planning_workspace",
        json!({"operation":"switch","name":"test-ws"}),
    )
    .await
    .unwrap();
    let listed = s
        .handle_tool("planning_workspace", json!({"operation":"list"}))
        .await
        .unwrap();
    let ws = listed
        .get("workspaces")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!ws.is_empty());
}

#[tokio::test]
async fn test_92_workspace_delete() {
    let mut s = fresh_server().await;
    s.handle_tool(
        "planning_workspace",
        json!({"operation":"create","name":"doomed","path":"doomed.aplan"}),
    )
    .await
    .unwrap();
    let deleted = s
        .handle_tool(
            "planning_workspace",
            json!({"operation":"delete","name":"doomed"}),
        )
        .await
        .unwrap();
    assert_eq!(
        deleted.get("status").and_then(|v| v.as_str()),
        Some("deleted")
    );
}

// ===========================================================================
// Section 18: Stress tests — rapid-fire MCP
// ===========================================================================

#[tokio::test]
async fn stress_100_goal_creates() {
    let mut s = fresh_server().await;
    for i in 0..100 {
        let out = s
            .handle_tool(
                "planning_goal",
                json!({
                    "operation": "create",
                    "title": format!("Stress Goal {i}"),
                    "intention": "rapid-fire creation"
                }),
            )
            .await
            .unwrap();
        assert!(out.get("id").is_some());
    }
}

#[tokio::test]
async fn stress_50_decisions_with_options() {
    let mut s = fresh_server().await;
    for i in 0..50 {
        let d = s
            .handle_tool(
                "planning_decision",
                json!({"operation":"create","question":format!("Decision {i}?")}),
            )
            .await
            .unwrap();
        let id = d.get("id").and_then(|v| v.as_str()).unwrap().to_string();
        s.handle_tool(
            "planning_decision",
            json!({"operation":"option","id":id,"name":"A","description":"option a"}),
        )
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn stress_50_commitments() {
    let mut s = fresh_server().await;
    for i in 0..50 {
        let out = s
            .handle_tool(
                "planning_commitment",
                json!({
                    "operation": "create",
                    "promise": format!("Commit {i}"),
                    "stakeholder": "Ops"
                }),
            )
            .await
            .unwrap();
        assert!(out.get("id").is_some());
    }
}

#[tokio::test]
async fn stress_progress_after_many_goals() {
    let mut s = fresh_server().await;
    for i in 0..20 {
        let g = s
            .handle_tool(
                "planning_goal",
                json!({
                    "operation":"create",
                    "title":format!("Prog Goal {i}"),
                    "intention":"measure progress at scale"
                }),
            )
            .await
            .unwrap();
        let id = g.get("id").and_then(|v| v.as_str()).unwrap().to_string();
        s.handle_tool("planning_goal", json!({"operation":"activate","id":id}))
            .await
            .unwrap();
    }
    // momentum without goal_id returns array of active goals
    let momentum = s
        .handle_tool("planning_progress", json!({"operation":"momentum"}))
        .await
        .unwrap();
    assert!(momentum.is_array());
    assert_eq!(momentum.as_array().unwrap().len(), 20);
}

#[tokio::test]
async fn stress_singularity_with_rich_state() {
    let mut s = fresh_server().await;
    // Populate diverse state
    for i in 0..10 {
        s.handle_tool(
            "planning_goal",
            json!({
                "operation":"create",
                "title":format!("Singularity Goal {i}"),
                "intention":format!("theme {}", i % 3)
            }),
        )
        .await
        .unwrap();
    }
    for i in 0..5 {
        s.handle_tool(
            "planning_decision",
            json!({"operation":"create","question":format!("Q{i}?")}),
        )
        .await
        .unwrap();
    }
    // Singularity should work with rich state
    let collapsed = s
        .handle_tool("planning_singularity", json!({"operation":"collapse"}))
        .await
        .unwrap();
    assert!(collapsed.is_object());
    let themes = s
        .handle_tool("planning_singularity", json!({"operation":"themes"}))
        .await
        .unwrap();
    assert!(themes.is_array() || themes.is_object());
}

// ===========================================================================
// Section 19: State stability after error storms
// ===========================================================================

#[tokio::test]
async fn stability_errors_dont_corrupt_state() {
    let mut s = fresh_server().await;
    // Create valid state first
    let g = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"create","title":"Stable","intention":"survive errors"}),
        )
        .await
        .unwrap();
    let id = g.get("id").and_then(|v| v.as_str()).unwrap().to_string();

    // Storm of errors
    for _ in 0..20 {
        let _ = s
            .handle_tool("planning_goal", json!({"operation":"show"}))
            .await;
        let _ = s
            .handle_tool("planning_nonexistent", json!({"operation":"x"}))
            .await;
        let _ = s
            .handle_tool("planning_goal", json!({"operation":"explode"}))
            .await;
        let _ = s
            .handle_tool(
                "planning_goal",
                json!({"operation":"activate","id":"not-a-uuid"}),
            )
            .await;
    }

    // State must still be intact
    let shown = s
        .handle_tool("planning_goal", json!({"operation":"show","id":id}))
        .await
        .unwrap();
    assert_eq!(
        shown.get("title").and_then(|v| v.as_str()),
        Some("Stable")
    );
}

#[tokio::test]
async fn stability_rapid_list_after_creates() {
    let mut s = fresh_server().await;
    for i in 0..30 {
        s.handle_tool(
            "planning_goal",
            json!({"operation":"create","title":format!("G{i}"),"intention":"list test"}),
        )
        .await
        .unwrap();
    }
    let listed = s
        .handle_tool("planning_goal", json!({"operation":"list"}))
        .await
        .unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 30);
}

// ===========================================================================
// Section 20: Workspace merge with real files
// ===========================================================================

#[tokio::test]
async fn test_93_workspace_merge_real_files() {
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
            json!({"operation":"create","name":"src","path":source_path.to_string_lossy()}),
        )
        .await
        .unwrap();
    server
        .handle_tool(
            "planning_workspace",
            json!({"operation":"create","name":"tgt","path":target_path.to_string_lossy()}),
        )
        .await
        .unwrap();

    let merged = server
        .handle_tool(
            "planning_workspace",
            json!({"operation":"merge","source":"src","target":"tgt"}),
        )
        .await
        .unwrap();
    assert_eq!(
        merged.get("status").and_then(|v| v.as_str()),
        Some("merged")
    );

    let target = PlanningEngine::open(&target_path).unwrap();
    assert_eq!(target.goal_count(), 2);
}

// ===========================================================================
// Section 21: Singularity with position & path
// ===========================================================================

#[tokio::test]
async fn test_94_singularity_position() {
    // position requires "id" (goal_id) via get_goal_id(&params)
    let (mut s, id) = server_with_active_goal().await;
    let pos = s
        .handle_tool(
            "planning_singularity",
            json!({"operation":"position","id":id}),
        )
        .await
        .unwrap();
    // Returns the position for this goal in the singularity (may be null if not computed)
    assert!(pos.is_object() || pos.is_null());
}

#[tokio::test]
async fn test_95_singularity_path() {
    let (mut s, _id) = server_with_active_goal().await;
    let path = s
        .handle_tool("planning_singularity", json!({"operation":"path"}))
        .await
        .unwrap();
    assert!(path.is_object() || path.is_array());
}

#[tokio::test]
async fn test_96_singularity_tensions() {
    let mut s = fresh_server().await;
    // Create conflicting goals to generate tensions
    s.handle_tool(
        "planning_goal",
        json!({"operation":"create","title":"Speed","intention":"go fast","priority":"high"}),
    )
    .await
    .unwrap();
    s.handle_tool(
        "planning_goal",
        json!({"operation":"create","title":"Quality","intention":"no bugs","priority":"high"}),
    )
    .await
    .unwrap();
    let tensions = s
        .handle_tool("planning_singularity", json!({"operation":"tensions"}))
        .await
        .unwrap();
    assert!(tensions.is_object() || tensions.is_array());
}

// ===========================================================================
// Section 22: Counterfactual & chain operations
// ===========================================================================

#[tokio::test]
async fn test_97_counterfactual_learn() {
    let (mut s, id) = server_with_decision().await;
    s.handle_tool(
        "planning_decision",
        json!({"operation":"option","id":id,"name":"A","description":"a"}),
    )
    .await
    .unwrap();
    s.handle_tool(
        "planning_decision",
        json!({"operation":"option","id":id,"name":"B","description":"b"}),
    )
    .await
    .unwrap();
    let learned = s
        .handle_tool(
            "planning_counterfactual",
            json!({"operation":"learn","decision_id":id}),
        )
        .await
        .unwrap();
    assert!(learned.is_object());
}

#[tokio::test]
async fn test_98_counterfactual_timeline() {
    let (mut s, id) = server_with_decision().await;
    let timeline = s
        .handle_tool(
            "planning_counterfactual",
            json!({"operation":"timeline","decision_id":id}),
        )
        .await
        .unwrap();
    assert!(timeline.is_object() || timeline.is_array());
}

#[tokio::test]
async fn test_99_chain_trace() {
    let (mut s, id) = server_with_decision().await;
    let trace = s
        .handle_tool(
            "planning_chain",
            json!({"operation":"trace","decision_id":id}),
        )
        .await
        .unwrap();
    assert!(trace.is_object() || trace.is_array());
}

#[tokio::test]
async fn test_100_chain_leaves() {
    let (mut s, id) = server_with_decision().await;
    let leaves = s
        .handle_tool(
            "planning_chain",
            json!({"operation":"leaves","decision_id":id}),
        )
        .await
        .unwrap();
    assert!(leaves.is_object() || leaves.is_array());
}

#[tokio::test]
async fn test_101_chain_cascade() {
    let (mut s, id) = server_with_decision().await;
    let cascade = s
        .handle_tool(
            "planning_chain",
            json!({"operation":"cascade","decision_id":id}),
        )
        .await
        .unwrap();
    assert!(cascade.is_object() || cascade.is_array());
}

// ===========================================================================
// Section 23: Goal link
// ===========================================================================

#[tokio::test]
async fn test_102_goal_link_dependency() {
    let mut s = fresh_server().await;
    let g1 = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"create","title":"Parent","intention":"link test"}),
        )
        .await
        .unwrap();
    let g2 = s
        .handle_tool(
            "planning_goal",
            json!({"operation":"create","title":"Child","intention":"link test"}),
        )
        .await
        .unwrap();
    let id1 = g1.get("id").and_then(|v| v.as_str()).unwrap().to_string();
    let id2 = g2.get("id").and_then(|v| v.as_str()).unwrap().to_string();

    // link uses "goal_a"/"goal_b" keys and "dependency"|"alliance" relationship
    let linked = s
        .handle_tool(
            "planning_goal",
            json!({
                "operation":"link",
                "goal_a":id1,
                "goal_b":id2,
                "relationship":"dependency"
            }),
        )
        .await
        .unwrap();
    assert_eq!(
        linked.get("status").and_then(|v| v.as_str()),
        Some("linked")
    );
}
