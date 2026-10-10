//! Run with `cargo run --locked --release -p bibcitex-core --example search_benchmark`.
//! Synthetic, non-GUI measurements; not a timing assertion or a device acceptance test.
use bibcitex_core::core::{
    search::{SearchField, SearchIndex},
    utils::read_bibliography,
};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

fn measure(mut search: impl FnMut() -> Vec<usize>) -> (Duration, usize) {
    let mut samples = Vec::new();
    let mut count = 0;
    for _ in 0..30 {
        let start = Instant::now();
        count = black_box(search()).len();
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    (samples[28], count)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("records,parse_ms,index_ms,index_MiB,query,auto_P95_ms,single_worker_P95_ms,matches");
    let serial = rayon::ThreadPoolBuilder::new().num_threads(1).build()?;
    for count in [1_000, 10_000, 50_000] {
        let bibliography: String = (0..count).map(|i| format!(
            "@article{{paper{i:05},title={{Graph analysis with neural networks and machine-learning methods {i}}},author={{Smith, Alice and Gödel, Bob}},journal={{Journal of Computing}},year={{2026}},note={{Synthetic ordinary search performance fixture}}}}\n"
        )).collect();
        let parse_start = Instant::now();
        let source = read_bibliography(biblatex::Bibliography::parse(&bibliography)?);
        let parse = parse_start.elapsed();
        let start = Instant::now();
        let index = SearchIndex::new(&source);
        let build = start.elapsed();
        for query in [
            "graph neural",
            "Smith learning",
            "netwroks",
            "nonexistent",
            "paper00001",
        ] {
            black_box(index.search(query, SearchField::All));
            let (auto, matches) = measure(|| index.search(query, SearchField::All));
            let (single, _) = serial.install(|| measure(|| index.search(query, SearchField::All)));
            println!(
                "{count},{:.2},{:.2},{:.2},{query},{:.2},{:.2},{matches}",
                parse.as_secs_f64() * 1000.0,
                build.as_secs_f64() * 1000.0,
                index.allocated_bytes() as f64 / 1048576.0,
                auto.as_secs_f64() * 1000.0,
                single.as_secs_f64() * 1000.0
            );
        }
    }
    Ok(())
}
