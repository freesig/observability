use tracing_timing::TimingSubscriber;

/// Publish deadlock statistics
pub fn tick_latency() {
    tracing::dispatcher::get_default(|dispatch| {
        if let Some(latency) = dispatch.downcast_ref::<TimingSubscriber>() {
            latency.force_synchronize();
            latency.with_histograms(|hs| {
                for (span_group, hs) in hs {
                    for (event_group, h) in hs {
                        // make sure we see the latest samples:
                        // h.refresh();
                        // print the median:
                        println!(
                            "{} -> {}: mean: {:.1}µs, p50: {}µs, p90: {}µs, p99: {}µs, p999: {}µs, max: {}µs",
                            span_group,
                            event_group,
                            h.mean() / 1000.0,
                            h.value_at_quantile(0.5) / 1_000,
                            h.value_at_quantile(0.9) / 1_000,
                            h.value_at_quantile(0.99) / 1_000,
                            h.value_at_quantile(0.999) / 1_000,
                            h.max() / 1_000,
                        );
                        println!(
                            "{} -> {}: mean: {:.1}ms, p50: {}ms, p90: {}ms, p99: {}ms, p999: {}ms, max: {}ms",
                            span_group,
                            event_group,
                            h.mean() / 1_000_000.0,
                            h.value_at_quantile(0.5) / 1_000_000,
                            h.value_at_quantile(0.9) / 1_000_000,
                            h.value_at_quantile(0.99) / 1_000_000,
                            h.value_at_quantile(0.999) / 1_000_000,
                            h.max() / 1_000_000,
                        );
                    }
                }
            });
        }
    });
}
