//! Benchmark tests for the AgenticPlanning research paper.
//! These produce validated performance data to be cited in the paper.
//!
//! Run with: cargo test --release -p agentic-planning --test paper_benchmarks -- --nocapture

use agentic_planning::{
    CreateCommitmentRequest, CreateDecisionRequest, CreateGoalRequest, DecisionPath,
    DecisionReasoning, GoalFilter, PathId, PlanningEngine, Priority, Promise, Stakeholder,
    StakeholderId,
};
use std::collections::HashMap;
use std::time::Instant;
use tempfile::tempdir;

/// Benchmark: File save performance at varying entity counts.
#[test]
fn paper_bench_save_scaling() {
    println!("\n=== PAPER BENCHMARK: Save Scaling ===");
    println!(
        "{:<10} {:<14} {:<14} {:<12} {:<12}",
        "Goals", "Save (ms)", "Load (ms)", "File (KB)", "B/goal"
    );

    for count in [10u32, 100, 500, 1000, 5000] {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bench.aplan");
        let mut engine = PlanningEngine::create(&path).unwrap();

        for i in 0..count {
            let _ = engine.create_goal(CreateGoalRequest {
                title: format!("Goal {i}"),
                intention: format!("Benchmark intention for goal number {i}"),
                priority: Some(Priority::Medium),
                ..Default::default()
            });
        }

        // Warm-up save
        engine.save().unwrap();

        // Measure save (3 runs, take median)
        // To force re-save, we add a trivial goal then save
        let mut save_times = Vec::new();
        for run in 0..3u32 {
            let _ = engine.create_goal(CreateGoalRequest {
                title: format!("_save_trigger_{run}"),
                intention: "trigger dirty".into(),
                ..Default::default()
            });
            let start = Instant::now();
            engine.save().unwrap();
            save_times.push(start.elapsed());
        }
        save_times.sort();
        let save_ms = save_times[1].as_secs_f64() * 1000.0;

        let file_size = std::fs::metadata(&path).unwrap().len();

        // Measure load (3 runs, take median)
        let mut load_times = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            let _ = PlanningEngine::load(&path).unwrap();
            load_times.push(start.elapsed());
        }
        load_times.sort();
        let load_ms = load_times[1].as_secs_f64() * 1000.0;

        println!(
            "{:<10} {:<14.3} {:<14.3} {:<12.1} {:<12}",
            count,
            save_ms,
            load_ms,
            file_size as f64 / 1024.0,
            file_size / count as u64
        );
    }
}

/// Benchmark: Core entity creation latency.
#[test]
fn paper_bench_creation_latency() {
    println!("\n=== PAPER BENCHMARK: Entity Creation Latency ===");

    // Goal creation
    let iterations = 10_000;
    let mut engine = PlanningEngine::in_memory();
    let start = Instant::now();
    for i in 0..iterations {
        let _ = engine.create_goal(CreateGoalRequest {
            title: format!("Goal {i}"),
            intention: "Latency measurement".into(),
            ..Default::default()
        });
    }
    let goal_time = start.elapsed();
    println!(
        "Goal creation:       {:.3} us/op  ({} ops in {:.1} ms)",
        goal_time.as_secs_f64() * 1_000_000.0 / iterations as f64,
        iterations,
        goal_time.as_secs_f64() * 1000.0
    );

    // Decision creation
    let mut engine2 = PlanningEngine::in_memory();
    let goal = engine2
        .create_goal(CreateGoalRequest {
            title: "Anchor".into(),
            intention: "Anchor goal".into(),
            ..Default::default()
        })
        .unwrap();
    let start = Instant::now();
    for i in 0..iterations {
        let _ = engine2.create_decision(CreateDecisionRequest {
            question: format!("Decision question {i}?"),
            goals: Some(vec![goal.id]),
            ..Default::default()
        });
    }
    let decision_time = start.elapsed();
    println!(
        "Decision creation:   {:.3} us/op  ({} ops in {:.1} ms)",
        decision_time.as_secs_f64() * 1_000_000.0 / iterations as f64,
        iterations,
        decision_time.as_secs_f64() * 1000.0
    );

    // Commitment creation
    let mut engine3 = PlanningEngine::in_memory();
    let goal3 = engine3
        .create_goal(CreateGoalRequest {
            title: "Anchor".into(),
            intention: "Anchor".into(),
            ..Default::default()
        })
        .unwrap();
    let start = Instant::now();
    for i in 0..iterations {
        let _ = engine3.create_commitment(CreateCommitmentRequest {
            promise: Promise {
                description: format!("Promise {i}"),
                deliverables: vec!["item".into()],
                conditions: vec![],
            },
            stakeholder: Stakeholder {
                id: StakeholderId(uuid::Uuid::new_v4()),
                name: "Test Stakeholder".into(),
                role: "colleague".into(),
                importance: 0.8,
            },
            goal: Some(goal3.id),
            ..Default::default()
        });
    }
    let commitment_time = start.elapsed();
    println!(
        "Commitment creation: {:.3} us/op  ({} ops in {:.1} ms)",
        commitment_time.as_secs_f64() * 1_000_000.0 / iterations as f64,
        iterations,
        commitment_time.as_secs_f64() * 1000.0
    );
}

/// Benchmark: Query performance on populated engine.
#[test]
fn paper_bench_query_performance() {
    println!("\n=== PAPER BENCHMARK: Query Performance ===");

    let mut engine = PlanningEngine::in_memory();
    // Populate with 1000 goals, some activated
    for i in 0..1000 {
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: format!("Goal {i}"),
                intention: format!("Intention {i}"),
                priority: Some(if i % 3 == 0 {
                    Priority::High
                } else {
                    Priority::Medium
                }),
                ..Default::default()
            })
            .unwrap();
        if i % 2 == 0 {
            let _ = engine.activate_goal(goal.id);
        }
    }

    // list_goals (unfiltered)
    let iterations = 10_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = engine.list_goals(GoalFilter::default());
    }
    let list_time = start.elapsed();
    println!(
        "list_goals (1000 goals):     {:.3} us/query ({} queries)",
        list_time.as_secs_f64() * 1_000_000.0 / iterations as f64,
        iterations
    );

    // get_active_goals
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = engine.get_active_goals();
    }
    let active_time = start.elapsed();
    println!(
        "get_active_goals (500 active): {:.3} us/query ({} queries)",
        active_time.as_secs_f64() * 1_000_000.0 / iterations as f64,
        iterations
    );

    // get_intention_singularity
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = engine.get_intention_singularity();
    }
    let singularity_time = start.elapsed();
    println!(
        "intention_singularity:       {:.3} us/op (1000 ops)",
        singularity_time.as_secs_f64() * 1_000_000.0 / 1000.0
    );

    // get_momentum_report
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = engine.get_momentum_report();
    }
    let momentum_time = start.elapsed();
    println!(
        "momentum_report:             {:.3} us/op (1000 ops)",
        momentum_time.as_secs_f64() * 1_000_000.0 / 1000.0
    );

    // get_gravity_field
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = engine.get_gravity_field();
    }
    let gravity_time = start.elapsed();
    println!(
        "gravity_field:               {:.3} us/op (1000 ops)",
        gravity_time.as_secs_f64() * 1_000_000.0 / 1000.0
    );
}

/// Benchmark: Dream generation performance.
#[test]
fn paper_bench_dream_generation() {
    println!("\n=== PAPER BENCHMARK: Dream Generation ===");

    let mut engine = PlanningEngine::in_memory();
    let mut goal_ids = Vec::new();
    for i in 0..100 {
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: format!("Dreamable Goal {i}"),
                intention: format!("Understand the path forward for {i}"),
                priority: Some(Priority::High),
                ..Default::default()
            })
            .unwrap();
        let _ = engine.activate_goal(goal.id);
        goal_ids.push(goal.id);
    }

    let start = Instant::now();
    for id in &goal_ids {
        let _ = engine.dream_goal(*id);
    }
    let dream_time = start.elapsed();
    println!(
        "Dream generation: {:.3} us/dream (100 dreams in {:.3} ms)",
        dream_time.as_secs_f64() * 1_000_000.0 / 100.0,
        dream_time.as_secs_f64() * 1000.0
    );
}

/// Benchmark: Decision crystallization performance.
#[test]
fn paper_bench_crystallization() {
    println!("\n=== PAPER BENCHMARK: Decision Crystallization ===");

    let mut engine = PlanningEngine::in_memory();
    let goal = engine
        .create_goal(CreateGoalRequest {
            title: "Crystal anchor".into(),
            intention: "Anchor".into(),
            ..Default::default()
        })
        .unwrap();

    let mut decision_ids = Vec::new();
    let mut path_ids = Vec::new();
    for i in 0..100 {
        let dec = engine
            .create_decision(CreateDecisionRequest {
                question: format!("Should we proceed with option {i}?"),
                goals: Some(vec![goal.id]),
                ..Default::default()
            })
            .unwrap();
        // Add an option (path) via add_option
        let path = DecisionPath {
            id: PathId(uuid::Uuid::new_v4()),
            name: "Chosen path".into(),
            description: "The selected option".into(),
            pros: vec!["pro1".into(), "pro2".into()],
            cons: vec!["con1".into()],
            estimated_effort: Some(0.5),
            estimated_risk: Some(0.3),
        };
        let pid = path.id;
        let _ = engine.add_option(dec.id, path);
        decision_ids.push(dec.id);
        path_ids.push(pid);
    }

    let start = Instant::now();
    let mut crystallized = 0;
    for (id, path_id) in decision_ids.iter().zip(path_ids.iter()) {
        let reasoning = DecisionReasoning {
            rationale: "Benchmark rationale".into(),
            factors_considered: vec!["performance".into()],
            weights: HashMap::new(),
            confidence: 0.9,
        };
        if engine.crystallize(*id, *path_id, reasoning).is_ok() {
            crystallized += 1;
        }
    }
    let crystal_time = start.elapsed();
    println!(
        "Crystallization: {:.3} us/op ({} crystallized in {:.3} ms)",
        crystal_time.as_secs_f64() * 1_000_000.0 / crystallized as f64,
        crystallized,
        crystal_time.as_secs_f64() * 1000.0
    );
}

/// Benchmark: BLAKE3 checksum computation on file save.
#[test]
fn paper_bench_checksum() {
    println!("\n=== PAPER BENCHMARK: BLAKE3 Checksum Overhead ===");

    for count in [100u32, 500, 1000] {
        let dir = tempdir().unwrap();
        let path = dir.path().join("checksum.aplan");
        let mut engine = PlanningEngine::create(&path).unwrap();
        for i in 0..count {
            let _ = engine.create_goal(CreateGoalRequest {
                title: format!("Goal {i}"),
                intention: "checksum bench".into(),
                ..Default::default()
            });
        }
        engine.save().unwrap();
        let file_size = std::fs::metadata(&path).unwrap().len();

        // Measure save (includes serialization + checksum)
        // Force dirty by adding a goal each time
        let mut times = Vec::new();
        for run in 0..5u32 {
            let _ = engine.create_goal(CreateGoalRequest {
                title: format!("_cksum_trigger_{run}"),
                intention: "trigger".into(),
                ..Default::default()
            });
            let start = Instant::now();
            engine.save().unwrap();
            times.push(start.elapsed());
        }
        times.sort();
        let median_ms = times[2].as_secs_f64() * 1000.0;

        println!(
            "goals={:<5}  save={:.3}ms  size={:.1}KB  bytes/goal={}",
            count,
            median_ms,
            file_size as f64 / 1024.0,
            file_size / count as u64
        );
    }
}
