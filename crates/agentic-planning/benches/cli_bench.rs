use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn cli_arg_parse(c: &mut Criterion) {
    c.bench_function("cli_arg_parse", |b| {
        b.iter(|| { let args = vec!["binary", "--help"]; black_box(args); })
    });
}

fn cli_config_load(c: &mut Criterion) {
    c.bench_function("cli_config_load", |b| {
        b.iter(|| { let config = std::collections::HashMap::<String, String>::new(); black_box(config); })
    });
}

criterion_group!(benches, cli_arg_parse, cli_config_load);
criterion_main!(benches);
