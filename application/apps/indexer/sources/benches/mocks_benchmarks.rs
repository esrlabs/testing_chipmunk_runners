mod bench_utls;

use std::time::Duration;

use bench_utls::run_producer;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mocks::{mock_parser::MockParser, mock_source::MockByteSource};
use sources::producer::MessageProducer;

mod mocks;

/// Runs Benchmarks replicating the producer loop within Chipmunk sessions, using mocks for
/// [`parsers::Parser`] and [`sources::ByteSource`] to ensure that the measurements is for the
/// producer loop only.
///
/// NOTE: This benchmark suffers unfortunately from a lot of noise because we are running it with
/// asynchronous runtime. This test is configured to reduce this amount of noise as possible,
/// However it would be better to run it multiple time for double checking.
fn mocks_benchmark(c: &mut Criterion) {
    let max_parse_calls = 50000;

    c.bench_with_input(
        BenchmarkId::new("mocks_producer", max_parse_calls),
        &(max_parse_calls),
        |bencher, &max| {
            bencher
                // It's important to spawn a new runtime on each run to ensure to reduce the
                // potential noise produced from one runtime created at the start of all benchmarks
                // only.
                .to_async(tokio::runtime::Runtime::new().unwrap())
                .iter_batched(
                    || {
                        // Exclude initiation time from benchmarks.
                        let parser = MockParser::new(max);
                        let byte_source = MockByteSource::new();
                        let producer = MessageProducer::new(parser, byte_source, black_box(None));

                        producer
                    },
                    |producer| run_producer(producer),
                    criterion::BatchSize::SmallInput,
                )
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        // Warm up time is very important here because multiple async runtimes will be spawn in
        // that time which make the next ones to spawn more stable.
        .warm_up_time(Duration::from_secs(10))
        // Measurement time and sample sized to role out noise in the measurements as possible.
        .measurement_time(Duration::from_secs(20))
        .sample_size(200)
        // These two values help to reduce the noise level in the results.
        .significance_level(0.01)
        .noise_threshold(0.03);
    targets = mocks_benchmark
}

criterion_main!(benches);