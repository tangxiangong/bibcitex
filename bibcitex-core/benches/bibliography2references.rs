use bibcitex_core::{
    bib::parse,
    utils::{par_read_bibliography, serial_read_bibliography},
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let bibliography = parse("../database.bib").unwrap();
    c.bench_function("read bibliography serially", |b| {
        b.iter(|| serial_read_bibliography(black_box(bibliography.clone())));
    });
    c.bench_function("read bibliography parallelly", |b| {
        b.iter(|| par_read_bibliography(black_box(bibliography.clone())));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
