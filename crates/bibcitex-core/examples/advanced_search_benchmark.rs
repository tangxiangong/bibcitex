//! Release-only, non-UI search measurements on one deterministic data set.
//! Run with `cargo run --release -p bibcitex-core --example advanced_search_benchmark`.
use bibcitex_core::core::{
    search::{SearchField, SearchIndex, advanced::AdvancedIndex, query},
    utils::read_bibliography,
};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
fn measure(mut search: impl FnMut() -> usize) -> (Duration, Duration, usize) {
    let mut samples = Vec::new();
    let mut count = 0;
    for _ in 0..10 {
        let start = Instant::now();
        count = black_box(search());
        samples.push(start.elapsed());
    }
    samples.sort();
    (samples[5], samples[9], count)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "records,parse_ms,ordinary_index_ms,advanced_index_ms,total_index_mib,mode,query,p50_ms,p95_ms,matches"
    );
    for count in [1000, 10000, 50000] {
        let bibliography:String=(0..count).map(|i|format!("@article{{paper{i:05},title={{Graph analysis with neural networks and machine-learning methods {i}}},author={{Smith, Alice and Gödel, Bob}},journal={{Journal of Computing}},year={{2026}},note={{Synthetic search fixture}}}}\n")).collect();
        let start = Instant::now();
        let refs = read_bibliography(biblatex::Bibliography::parse(&bibliography)?);
        let parse = start.elapsed();
        let start = Instant::now();
        let ordinary = SearchIndex::new(&refs);
        let ordinary_build = start.elapsed();
        let start = Instant::now();
        let advanced = AdvancedIndex::new(&refs);
        let advanced_build = start.elapsed();
        let mib = (ordinary.allocated_bytes() + advanced.allocated_bytes()) as f64 / 1048576.0;
        for text in ["graph neural", "netwroks", "nonexistent"] {
            let (p50, p95, n) = measure(|| ordinary.search(text, SearchField::All).len());
            println!(
                "{count},{:.2},{:.2},{:.2},{mib:.2},ordinary,{text},{:.2},{:.2},{n}",
                parse.as_secs_f64() * 1000.0,
                ordinary_build.as_secs_f64() * 1000.0,
                advanced_build.as_secs_f64() * 1000.0,
                p50.as_secs_f64() * 1000.0,
                p95.as_secs_f64() * 1000.0
            );
        }
        for text in [
            "graph neural",
            "\"machine-learning\"",
            "(graph OR network) (author:Smith OR title:absent)",
            "-absent",
            "year:2020..2026 type:article",
            "tag:read -tag:archived",
            "netwroks",
        ] {
            let doc = query::analyze(text)
                .document
                .ok_or("Invalid benchmark query")?;
            let (p50, p95, n) = measure(|| {
                advanced
                    .search(
                        &doc,
                        "all",
                        |_| true,
                        |i, name, _| name == "read" && i % 3 == 0,
                    )
                    .unwrap()
                    .len()
            });
            let query_csv = text.replace('"', "\"\"");
            println!(
                "{count},{:.2},{:.2},{:.2},{mib:.2},advanced,\"{query_csv}\",{:.2},{:.2},{n}",
                parse.as_secs_f64() * 1000.0,
                ordinary_build.as_secs_f64() * 1000.0,
                advanced_build.as_secs_f64() * 1000.0,
                p50.as_secs_f64() * 1000.0,
                p95.as_secs_f64() * 1000.0
            );
        }
        for leaves in [16, 128, 1024] {
            let text = vec!["graph"; leaves].join(" OR ");
            let start = Instant::now();
            let doc = query::analyze(&text)
                .document
                .ok_or("Invalid limit fixture")?;
            let parse_query = start.elapsed();
            let start = Instant::now();
            let rows = advanced
                .search(&doc, "all", |i| i < 1000, |_, _, _| false)
                .map_err(|_| "Search failed")?;
            eprintln!(
                "limit records={count} leaves={leaves} parse_ms={:.2} search_1000_ms={:.2} matches={}",
                parse_query.as_secs_f64() * 1000.0,
                start.elapsed().as_secs_f64() * 1000.0,
                rows.len()
            );
        }
    }
    Ok(())
}
