use anyhow::Result;
use gungraun::prelude::*;
use gungraun::{Callgrind, Dhat, DhatMetric, EventKind, OutputFormat};

fn create_index_command(engine: &str) -> gungraun::Command {
    gungraun::Command::new(env!("CARGO_BIN_EXE_nano_search"))
        .arg(format!("--engines={engine}"))
        .arg("--dataset=cisi")
        .arg("--parent-index-dir=/tmp")
        .arg("index")
        .arg("--threads=1")
        .build()
}

#[binary_benchmark]
#[bench::index(args = ("tantivy"))]
fn bench_index_tantivy(engine: &str) -> gungraun::Command {
    create_index_command(engine)
}

#[binary_benchmark]
#[bench::index(args = ("nano"))]
fn bench_index_nano(engine: &str) -> gungraun::Command {
    create_index_command(engine)
}

binary_benchmark_group!(
    name = bench_index_group,
    // cross-compare benches with same ID (e.g. nano vs tantivy).
    // this is the main reason to create a group of several bench functions,
    // instead of single one with different arguments
    compare_by_id = true,
    benchmarks = [bench_index_tantivy, bench_index_nano]
);

#[allow(clippy::expect_used)]
fn setup_eval(engine: &str) {
    std::process::Command::new(env!("CARGO_BIN_EXE_nano_search"))
        .arg(format!("--engines={engine}"))
        .arg("--dataset=cisi")
        .arg("--parent-index-dir=/tmp")
        .arg("index")
        .arg("--threads=1")
        .status()
        .expect("should create index");
}

fn create_eval_command(engine: &str) -> gungraun::Command {
    gungraun::Command::new(env!("CARGO_BIN_EXE_nano_search"))
        .arg(format!("--engines={engine}"))
        .arg("--dataset=cisi")
        .arg("--parent-index-dir=/tmp")
        .arg("eval")
        .build()
}

#[binary_benchmark]
#[bench::eval(args = ("tantivy"), setup = setup_eval)]
fn bench_eval_tantivy(engine: &str) -> gungraun::Command {
    create_eval_command(engine)
}

#[binary_benchmark]
#[bench::eval(args = ("nano"), setup = setup_eval)]
fn bench_eval_nano(engine: &str) -> gungraun::Command {
    create_eval_command(engine)
}

binary_benchmark_group!(
    name = bench_eval_group,
    compare_by_id = true,
    benchmarks = [bench_eval_tantivy, bench_eval_nano]
);

main!(
    config = BinaryBenchmarkConfig::default()
        // collect and show CPU metrics
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)]))
        // collect and show RAM metrics
        .tool(Dhat::default().soft_limits([(DhatMetric::AtTGmaxBytes, 5.0)]))
        .output_format(
            OutputFormat::default()
                // do not highlight small variation of results as a change.
                // e.g. variation can happen due to non-derministic std hash
                // function, multi-thread channels and disk IO
                .tolerance(1.0)
        ),
    binary_benchmark_groups = [bench_index_group, bench_eval_group]
);
