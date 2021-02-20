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

    let span = debug_span!("span b");
    let _g = span.enter();

    tokio::time::delay_for(std::time::Duration::from_millis(10)).await;
    debug!("in span b");

    std::mem::drop(_g);
    std::mem::drop(span);

    tokio::time::delay_for(std::time::Duration::from_secs(10))
        .instrument(trace_span!("test_span"))
        .instrument(trace_span!("test_span2"))
        .await;

    let span = debug_span!("span c");
    let _g = span.enter();
    debug!("in span c");
}

#[tokio::test(threaded_scheduler)]
async fn latency() {
    observability::test_run_latency().ok();
    tokio::spawn(async {
        loop {
            observability::tick_latency();
            tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        }
    });
    let span = debug_span!("span a");
    let _g = span.enter();

    debug!(msg = "in span a");

    let span = debug_span!("span b");
    let _g = span.enter();

    tokio::time::delay_for(std::time::Duration::from_millis(10)).await;
    debug!("in span b");

    std::mem::drop(_g);
    std::mem::drop(span);

    tokio::time::delay_for(std::time::Duration::from_secs(10))
        .instrument(trace_span!("test_span"))
        .instrument(trace_span!("test_span2"))
        .await;

    let span = debug_span!("span c");
    let _g = span.enter();
    debug!("in span c");
}
