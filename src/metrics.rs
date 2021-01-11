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

        pub static TEST_METRICS: [std::sync::atomic::AtomicU64; 0usize $(+ $crate::__replace_expr!($metric 1usize))+] = [$($crate::__replace_expr!($metric std::sync::atomic::AtomicU64::new(0))),+];

        impl $name {
            pub fn count(metric: Self, n: u64) {
                if $crate::metrics::__METRICS_ON.load(std::sync::atomic::Ordering::Relaxed) {
                    let r = Self::count_silent(metric, n);
                    tracing::debug!(?metric, count = r);
                }
            }
            pub fn count_silent(metric: Self, n: u64) -> u64 {
                if $crate::metrics::__METRICS_ON.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut last = TEST_METRICS.get(metric as usize).expect("Used metric that doesn't exist").fetch_add(n, std::sync::atomic::Ordering::SeqCst);
                    last += n;
                    last
                } else {
                    0
                }
            }
        }
        // pub struct TestMetric;
        // impl TestMetric {
        //     $(const $metric: usize

        // }
        // const $name: Vec<std::sync::atomic::AtomicU64> = vec![$($metric),+];

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
