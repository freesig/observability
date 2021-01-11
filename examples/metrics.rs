use observability::metrics;
use tracing::*;
use std::error::Error;

metrics!(MyMetric, CounterA, CounterB);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    observability::test_run().ok();
    observability::metrics::init();
    let span = debug_span!("span a");
    let _g = span.enter();

    debug!(metric = "my_counter", count = 32);
    MyMetric::count(MyMetric::CounterA, 30);
    MyMetric::count(MyMetric::CounterA, 40);
    MyMetric::count(MyMetric::CounterB, 40);

    MyMetric::print();
    let mut td = std::env::temp_dir();
    td.push("metrics_csv");
    std::fs::create_dir(&td).unwrap();
    td.push("metrics.csv");
    MyMetric::save_csv(&td);

    Ok(())
}