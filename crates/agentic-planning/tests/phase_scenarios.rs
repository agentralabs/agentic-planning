//! SPEC-12 phase scenarios (canonical 16-scenario suite).

use agentic_planning::{
    Blocker, BlockerType, CommitmentStatus, CreateCommitmentRequest, CreateDecisionRequest,
    CreateGoalRequest, DecisionPath, DecisionReasoning, DecisionStatus, EntanglementType,
    GoalFilter, GoalStatus, PathId, PlanningEngine, Priority, Promise, ReincarnationUpdates,
    Stakeholder, StakeholderId, Timestamp,
};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

#[test]
fn scenario_01_goal_lifecycle() {
    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Build REST API".to_string(),
            intention: "Create a production REST API".to_string(),
            priority: Some(Priority::High),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(goal.status, GoalStatus::Draft);
    assert_eq!(goal.progress.percentage, 0.0);

    let goal = engine.activate_goal(goal.id).unwrap();
    assert_eq!(goal.status, GoalStatus::Active);
    assert!(goal.activated_at.is_some());

    let goal = engine
        .progress_goal(goal.id, 0.5, Some("Halfway done".to_string()))
        .unwrap();
    assert_eq!(goal.progress.percentage, 0.5);

    let goal = engine.complete_goal(goal.id, None).unwrap();
    assert_eq!(goal.status, GoalStatus::Completed);
}

#[test]
fn scenario_02_decision_crystallization() {
    let mut engine = PlanningEngine::in_memory();
    let decision = engine
        .create_decision(CreateDecisionRequest {
            question: "Which database to use?".to_string(),
            ..Default::default()
        })
        .unwrap();

    engine
        .add_option(
            decision.id,
            DecisionPath {
                id: PathId(Uuid::new_v4()),
                name: "PostgreSQL".to_string(),
                description: "Relational database".to_string(),
                pros: vec!["ACID".to_string()],
                cons: vec!["Setup complexity".to_string()],
                ..Default::default()
            },
        )
        .unwrap();

    engine
        .add_option(
            decision.id,
            DecisionPath {
                id: PathId(Uuid::new_v4()),
                name: "MongoDB".to_string(),
                description: "Document database".to_string(),
                pros: vec!["Flexible schema".to_string()],
                cons: vec!["No joins".to_string()],
                ..Default::default()
            },
        )
        .unwrap();

    let decision = engine.get_decision(decision.id).unwrap();
    assert_eq!(decision.shadows.len(), 2);

    let postgres_id = decision.shadows[0].path.id;
    let decision = engine
        .crystallize(
            decision.id,
            postgres_id,
            DecisionReasoning {
                rationale: "Need ACID compliance".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(decision.status, DecisionStatus::Crystallized);
    assert_eq!(decision.chosen.as_ref().unwrap().name, "PostgreSQL");
    assert_eq!(decision.shadows.len(), 1);
}

#[test]
fn scenario_03_commitment_lifecycle() {
    let mut engine = PlanningEngine::in_memory();
    let commitment = engine
        .create_commitment(CreateCommitmentRequest {
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
            due: Some(Timestamp::days_from_now(14.0)),
            ..Default::default()
        })
        .unwrap();

    assert!(commitment.weight > 0.0);
    assert_eq!(commitment.status, CommitmentStatus::Active);

    let commitment = engine
        .fulfill_commitment(commitment.id, "Delivered on time".to_string())
        .unwrap();
    assert_eq!(commitment.status, CommitmentStatus::Fulfilled);
    assert!(commitment.fulfillment.as_ref().unwrap().energy_released > 0.0);
}

#[test]
fn scenario_04_goal_hierarchy() {
    let mut engine = PlanningEngine::in_memory();
    let parent = engine
        .create_goal(CreateGoalRequest {
            title: "Build Application".to_string(),
            intention: "Complete application".to_string(),
            ..Default::default()
        })
        .unwrap();

    let children = engine
        .decompose_goal(
            parent.id,
            vec![
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
            ],
        )
        .unwrap();

    assert_eq!(children.len(), 2);
    let tree = engine.get_goal_tree(parent.id).unwrap();
    assert_eq!(tree.nodes.len(), 3);
}

#[test]
fn scenario_05_blocker_lifecycle() {
    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Deploy to Production".to_string(),
            intention: "Ship it".to_string(),
            ..Default::default()
        })
        .unwrap();
    engine.activate_goal(goal.id).unwrap();

    let blocker = Blocker {
        id: Uuid::new_v4(),
        blocker_type: BlockerType::ApprovalPending {
            approver: StakeholderId(Uuid::new_v4()),
        },
        description: "Waiting for security review".to_string(),
        severity: 0.8,
        identified_at: Timestamp::now(),
        resolved_at: None,
        resolution: None,
    };

    let goal = engine.block_goal(goal.id, blocker.clone()).unwrap();
    assert_eq!(goal.status, GoalStatus::Blocked);

    let goal = engine
        .unblock_goal(goal.id, blocker.id, "Security review passed".to_string())
        .unwrap();
    assert_eq!(goal.status, GoalStatus::Active);
}

#[test]
fn scenario_06_goal_feelings() {
    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Important Task".to_string(),
            intention: "Complete urgently".to_string(),
            deadline: Some(Timestamp::days_from_now(1.0)),
            priority: Some(Priority::Critical),
            ..Default::default()
        })
        .unwrap();
    engine.activate_goal(goal.id).unwrap();
    let goal = engine.get_goal(goal.id).unwrap();
    assert!(goal.feelings.urgency > 0.5);
    assert!(goal.feelings.neglect <= 0.1);
}

#[test]
fn scenario_07_decision_chain() {
    let mut engine = PlanningEngine::in_memory();
    let d1 = engine
        .create_decision(CreateDecisionRequest {
            question: "Language choice?".to_string(),
            ..Default::default()
        })
        .unwrap();
    let d2 = engine
        .create_decision(CreateDecisionRequest {
            question: "Framework choice?".to_string(),
            caused_by: Some(d1.id),
            ..Default::default()
        })
        .unwrap();
    let chain = engine.get_decision_chain(d2.id).unwrap();
    assert_eq!(chain.root, d1.id);
    assert!(chain.descendants.contains(&d2.id));
}

#[test]
fn scenario_08_intention_singularity() {
    let mut engine = PlanningEngine::in_memory();
    for i in 0..5 {
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: format!("Goal {i}"),
                intention: format!("Intention {i}"),
                ..Default::default()
            })
            .unwrap();
        engine.activate_goal(goal.id).unwrap();
    }

    let singularity = engine.get_intention_singularity();
    assert_eq!(singularity.goal_positions.len(), 5);
    assert!(!singularity.golden_path.is_empty());
}

#[test]
fn scenario_09_goal_dreaming() {
    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Build Feature".to_string(),
            intention: "New feature".to_string(),
            ..Default::default()
        })
        .unwrap();
    engine.activate_goal(goal.id).unwrap();
    let dream = engine.dream_goal(goal.id).unwrap();
    assert!(dream.confidence > 0.0);
    assert!(!dream.scenario.vision.is_empty());
}

#[test]
fn scenario_10_commitment_entanglement() {
    let mut engine = PlanningEngine::in_memory();
    let c1 = engine
        .create_commitment(CreateCommitmentRequest {
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
        })
        .unwrap();
    let c2 = engine
        .create_commitment(CreateCommitmentRequest {
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
        })
        .unwrap();

    engine
        .entangle_commitments(c1.id, c2.id, EntanglementType::Parallel, 0.8)
        .unwrap();
    let c1 = engine.get_commitment(c1.id).unwrap();
    assert_eq!(c1.entanglements.len(), 1);
}

#[test]
fn scenario_11_progress_momentum() {
    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Momentum Test".to_string(),
            intention: "Test".to_string(),
            ..Default::default()
        })
        .unwrap();
    engine.activate_goal(goal.id).unwrap();

    for i in 1..=5 {
        engine.progress_goal(goal.id, i as f64 * 0.1, None).unwrap();
    }
    let goal = engine.get_goal(goal.id).unwrap();
    assert!(goal.physics.momentum > 0.0);
}

#[test]
fn scenario_12_goal_reincarnation() {
    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Learn Rust".to_string(),
            intention: "Master Rust programming".to_string(),
            ..Default::default()
        })
        .unwrap();

    engine.activate_goal(goal.id).unwrap();
    engine.progress_goal(goal.id, 0.3, None).unwrap();
    engine
        .abandon_goal(goal.id, "Too busy".to_string())
        .unwrap();

    let reborn = engine
        .reincarnate_goal(
            goal.id,
            ReincarnationUpdates {
                title: Some("Learn Rust (Take 2)".to_string()),
                lessons_learned: Some(vec!["Need more dedicated time".to_string()]),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(reborn.status, GoalStatus::Reborn);
    assert!(reborn.previous_life.is_some());
}

#[test]
fn scenario_13_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.aplan");

    {
        let mut engine = PlanningEngine::open(&path).unwrap();
        engine
            .create_goal(CreateGoalRequest {
                title: "Persistent Goal".to_string(),
                intention: "Should survive".to_string(),
                ..Default::default()
            })
            .unwrap();
        engine.save().unwrap();
    }

    {
        let engine = PlanningEngine::open(&path).unwrap();
        let goals = engine.list_goals(GoalFilter::default());
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].title, "Persistent Goal");
    }
}

#[test]
fn scenario_14_concurrent() {
    let engine = Arc::new(Mutex::new(PlanningEngine::in_memory()));
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let engine = Arc::clone(&engine);
            thread::spawn(move || {
                let mut engine = engine.lock().unwrap();
                engine
                    .create_goal(CreateGoalRequest {
                        title: format!("Concurrent Goal {i}"),
                        intention: "Test".to_string(),
                        ..Default::default()
                    })
                    .unwrap();
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
    let engine = engine.lock().unwrap();
    let goals = engine.list_goals(GoalFilter::default());
    assert_eq!(goals.len(), 10);
}

#[test]
fn scenario_15_large_graph() {
    let mut engine = PlanningEngine::in_memory();

    let start = std::time::Instant::now();
    for i in 0..1000 {
        engine
            .create_goal(CreateGoalRequest {
                title: format!("Goal {i}"),
                intention: format!("Intention {i}"),
                ..Default::default()
            })
            .unwrap();
    }
    let create_time = start.elapsed();
    assert!(create_time.as_millis() < 5000, "Creation took too long");

    let start = std::time::Instant::now();
    let _ = engine.list_goals(GoalFilter::default());
    let query_time = start.elapsed();
    assert!(query_time.as_millis() < 100, "Query took too long");
}

#[test]
fn scenario_16_federation_lifecycle() {
    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Federated Goal".to_string(),
            intention: "Coordinate across agents".to_string(),
            ..Default::default()
        })
        .unwrap();
    engine.activate_goal(goal.id).unwrap();

    let federation = engine
        .create_federation(goal.id, "agent-alpha".to_string(), None)
        .unwrap();
    assert_eq!(federation.members.len(), 1);
    assert_eq!(federation.members[0].agent_id, "agent-alpha");

    let federation = engine
        .join_federation(federation.id, "agent-beta".to_string())
        .unwrap();
    assert_eq!(federation.members.len(), 2);

    let federation = engine.sync_federation(federation.id).unwrap();
    assert!(matches!(
        federation.sync_status,
        agentic_planning::SyncStatus::Synced
    ));

    let federation = engine
        .handoff_federation(federation.id, "agent-beta".to_string())
        .unwrap();
    assert_eq!(federation.coordinator.as_deref(), Some("agent-beta"));

    let members = engine.get_federation_members(federation.id).unwrap();
    assert_eq!(members.len(), 2);
}
