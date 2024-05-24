use criterion::{criterion_group, criterion_main, Criterion};

use crate::stack::abc273_e::WithPersistentStack;
mod stack;

fn benchmark(c: &mut Criterion) {
    use stack::abc273_e::Solver as _;
    c.benchmark_group("ABC273-E").bench_function("stack", |b| {
        let queries = stack::abc273_e::benchmark_case();
        let mut output = Vec::with_capacity(queries.len());
        b.iter(|| {
            output.clear();
            WithPersistentStack.solve(&queries, &mut output);
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
