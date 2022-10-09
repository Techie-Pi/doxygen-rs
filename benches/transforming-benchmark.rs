use std::fs;
use doxygen_rs::{transform, transform_bindgen};
use criterion::{
    criterion_group,
    criterion_main,
    Criterion,
};

const CTRU_SYS_BINDINGS: &str = include_str!("../assets/tests/ctru-sys-bindings.rs");

fn transform_bindgen_benchmark(c: &mut Criterion) {
    c.bench_function(
        "bindgen transform",
        |b| b.iter(|| transform(CTRU_SYS_BINDINGS))
    );
}

criterion_group!(benches, transform_bindgen_benchmark);
criterion_main!(benches);