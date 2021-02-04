use tracing::*;

#[tokio::test(threaded_scheduler)]
async fn waiting() {
    observability::test_run_dead().ok();
    tokio::spawn(async {
        loop {
            observability::tick_deadlock_catcher();
            tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        }
    });
    let span = debug_span!("span a");
    let _g = span.enter();

    debug!(msg = "in span a");

    tokio::time::delay_for(std::time::Duration::from_secs(10))
        .instrument(trace_span!("test_span"))
        .await;

    let span = debug_span!("span b");
    let _g = span.enter();
    debug!("in span b");

    let span = debug_span!("span c");
    let _g = span.enter();
    debug!("in span c");
}
