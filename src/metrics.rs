//! # Metrics
//! WIP metrics helper for counting values
//! and sending tracing events.
//! This is designed to be fast so everything
//! is on the stack.
//! This means you need to keep the metric sets small (<100 metrics per set).
//! If you need more then make a new set.
use std::sync::atomic::AtomicBool;

#[allow(missing_docs)]
#[doc(hidden)]
pub static __METRICS_ON: AtomicBool = AtomicBool::new(false);

/// Enable all metrics for your program
pub fn init() {
    __METRICS_ON.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Create a metrics set.
/// Takes the name of the metric set followed by
/// a list of metric names.
#[macro_export]
macro_rules! metrics {
    ($name:ident, $($metric:ident),+) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($metric),+
        }

        mod metrics_inner {
            pub(super) const NUM: usize = 0usize $(+ $crate::__replace_expr!($metric 1usize))+;
            pub(super) static METRICS: [std::sync::atomic::AtomicU64; NUM] = [$($crate::__replace_expr!($metric std::sync::atomic::AtomicU64::new(0))),+];
            pub(super) const NAMES: [&'static str; NUM] = [$(stringify!($metric)),+];
        }
        impl $name {
            pub fn count<N: std::convert::TryInto<u64, Error = std::num::TryFromIntError>>(metric: Self, n: N) {
                if $crate::metrics::__METRICS_ON.load(std::sync::atomic::Ordering::Relaxed) {
                    let n = n.try_into().expect("Failed to convert metric to u64");
                    let r = Self::count_silent(metric, n);
                    $crate::tracing::debug!(?metric, count = r);
                }
            }
            pub fn count_silent(metric: Self, n: u64) -> u64 {
                if $crate::metrics::__METRICS_ON.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut last = metrics_inner::METRICS[metric as usize].fetch_add(n, std::sync::atomic::Ordering::Relaxed);
                    last += n;
                    last
                } else {
                    0
                }
            }
            pub fn print() {
                if $crate::metrics::__METRICS_ON.load(std::sync::atomic::Ordering::Relaxed) {
                    for (i, count) in metrics_inner::METRICS.iter().enumerate() {
                        let metric = metrics_inner::NAMES[i];
                        let count = count.load(std::sync::atomic::Ordering::Relaxed);
                        $crate::tracing::debug!(%metric, count);
                    }
                }
            }
            pub fn save_csv(path: &std::path::Path) {
                if $crate::metrics::__METRICS_ON.load(std::sync::atomic::Ordering::Relaxed) {
                    use std::fmt::Write;
                    let mut keys = String::new();
                    let mut values = String::new();
                    for (i, count) in metrics_inner::METRICS.iter().enumerate() {
                        let metric = metrics_inner::NAMES[i];
                        let count = count.load(std::sync::atomic::Ordering::Relaxed);
                        write!(keys, "{},", metric).expect("Failed to write metrics");
                        write!(values, "{},", count).expect("Failed to write metrics");
                    }
                    std::fs::write(path, format!("{}\n{}\n", keys, values)).expect("Failed to write metrics to csv");
                    $crate::tracing::info!(metrics = "Saved csv to", ?path);
                }
            }
        }

    };
}
#[macro_export]
#[allow(missing_docs)]
#[doc(hidden)]
macro_rules! __replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}
