use criterion::{criterion_group, criterion_main, Criterion};
use shipyard::World;
use tag::{initialize_world, TICK};

fn criterion_benchmark(c: &mut Criterion) {
    let world = initialize_world(1000);
    c.bench_function("tick world", |b| {
        b.iter(|| world.run_workload(TICK).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
