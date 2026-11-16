//! Reproducible smoke benchmark, not a general performance guarantee.
//! cargo run --release --example benchmark -- 1000 100
use airust::{Agent, ResponseFormat, TfidfAgent, TrainableAgent, TrainingExample};
use std::{env, hint::black_box, time::Instant};

fn main() {
    let args: Vec<String> = env::args().collect();
    let count = args
        .get(1)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1000)
        .max(1);
    let queries = args
        .get(2)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100)
        .max(1);
    let examples: Vec<TrainingExample> = (0..count)
        .map(|i| TrainingExample {
            input: format!("document topic{} rust programming reference", i % 20),
            output: ResponseFormat::Text(format!("answer-{i}")),
            weight: 1.0,
            metadata: None,
        })
        .collect();
    let mut agent = TfidfAgent::new();
    let started = Instant::now();
    agent.train(&examples);
    let build = started.elapsed();
    let started = Instant::now();
    for _ in 0..queries {
        black_box(agent.predict(black_box("rust programming")));
    }
    let search = started.elapsed();
    let started = Instant::now();
    agent.append(&examples[..1]);
    println!(
        "entries={count} queries={queries} build_ms={:.3} search_ms={:.3} append_ms={:.3}",
        build.as_secs_f64() * 1000.0,
        search.as_secs_f64() * 1000.0,
        started.elapsed().as_secs_f64() * 1000.0
    );
}
