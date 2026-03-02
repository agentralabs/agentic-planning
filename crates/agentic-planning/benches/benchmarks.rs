use agentic_planning::{CreateGoalRequest, GoalFilter, PlanningEngine};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_create_goal(c: &mut Criterion) {
    c.bench_function("create_goal", |b| {
        b.iter(|| {
            let mut engine = PlanningEngine::in_memory();
            let _ = engine.create_goal(CreateGoalRequest {
                title: "Bench Goal".to_string(),
                intention: "Measure creation".to_string(),
                ..Default::default()
            });
        })
    });
}

fn bench_create_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_many_goals");
    for size in [10usize, 100, 1000] {
        group.bench_with_input(BenchmarkId::new("count", size), &size, |b, size| {
            b.iter(|| {
                let mut engine = PlanningEngine::in_memory();
                for i in 0..*size {
                    let _ = engine.create_goal(CreateGoalRequest {
                        title: format!("Goal {i}"),
                        intention: "Bulk create".to_string(),
                        ..Default::default()
                    });
                }
            })
        });
    }
    group.finish();
}

fn bench_list_goals(c: &mut Criterion) {
    let mut engine = PlanningEngine::in_memory();
    for i in 0..1000 {
        let _ = engine.create_goal(CreateGoalRequest {
            title: format!("Goal {i}"),
            intention: "Query workload".to_string(),
            ..Default::default()
        });
    }

    c.bench_function("list_goals", |b| {
        b.iter(|| {
            let _ = engine.list_goals(GoalFilter::default());
        })
    });
}

fn bench_active_goals(c: &mut Criterion) {
    let mut engine = PlanningEngine::in_memory();
    for i in 0..500 {
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: format!("Active Goal {i}"),
                intention: "Active set".to_string(),
                ..Default::default()
            })
            .expect("create goal");
        let _ = engine.activate_goal(goal.id);
    }

    c.bench_function("get_active_goals", |b| {
        b.iter(|| {
            let _ = engine.get_active_goals();
        })
    });
}

fn bench_singularity(c: &mut Criterion) {
    let mut engine = PlanningEngine::in_memory();
    for i in 0..200 {
        let goal = engine
            .create_goal(CreateGoalRequest {
                title: format!("Field Goal {i}"),
                intention: "Unified intention".to_string(),
                ..Default::default()
            })
            .expect("create goal");
        let _ = engine.activate_goal(goal.id);
    }

    c.bench_function("get_intention_singularity", |b| {
        b.iter(|| {
            let _ = engine.get_intention_singularity();
        })
    });
}

criterion_group!(
    benches,
    bench_create_goal,
    bench_create_many,
    bench_list_goals,
    bench_active_goals,
    bench_singularity
);
criterion_main!(benches);
