use tracing::*;

#[test]
fn reload_filter() {
    let mut handle = observability::dyn_test_run().unwrap();

    let span = debug_span!("span");
    span.in_scope(|| debug!("test"));

    handle.reload("debug").unwrap();

    let span = debug_span!("span");
    span.in_scope(|| debug!("test"));
}
