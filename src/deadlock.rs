use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use tracing::span::Attributes;
use tracing::Metadata;
use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

pub struct TimingLayer {
    timings: std::sync::RwLock<HashMap<tracing::span::Id, Timing>>,
}

#[derive(Debug, Clone)]
pub struct Timing {
    start: Instant,
    enter: Option<Instant>,
    busy: Duration,
    metadata: &'static Metadata<'static>,
}

impl Timing {
    fn new(metadata: &'static Metadata<'static>) -> Self {
        Self {
            start: Instant::now(),
            enter: None,
            busy: Duration::default(),
            metadata,
        }
    }

    fn idle(&self) -> Duration {
        let total = self.start.elapsed();
        total.checked_sub(self.busy()).unwrap_or_default()
    }

    fn busy(&self) -> Duration {
        self.enter.map(|e| e.elapsed()).unwrap_or_default() + self.busy
    }

    fn ice(&self) -> Duration {
        self.idle().checked_sub(self.busy()).unwrap_or_default()
    }

    fn flame(&self) -> Duration {
        self.busy().checked_sub(self.idle()).unwrap_or_default()
    }
}

impl TimingLayer {
    pub fn new() -> Self {
        Self {
            timings: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl<S> Layer<S> for TimingLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn new_span(
        &self,
        attrs: &Attributes<'_>,
        id: &tracing::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        {
            if let Ok(mut t) = self.timings.write() {
                t.insert(id.clone(), Timing::new(attrs.metadata()));
            }
        }
    }

    fn on_enter(&self, id: &tracing::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Ok(mut t) = self.timings.write() {
            if let Some(t) = t.get_mut(id) {
                t.enter = Some(Instant::now());
            }
        }
    }

    fn on_exit(&self, id: &tracing::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Ok(mut t) = self.timings.write() {
            if let Some(t) = t.get_mut(id) {
                if let Some(enter) = t.enter.take() {
                    t.busy += enter.elapsed();
                }
            }
        }
    }

    fn on_close(&self, id: tracing::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Ok(mut t) = self.timings.write() {
            t.remove(&id);
        }
    }
}

pub fn tick_deadlock_catcher() {
    tracing::dispatcher::get_default(|dispatch| match dispatch.downcast_ref::<TimingLayer>() {
        Some(timing) => {
            let timings = {
                timing.timings.read().map(|t| {
                    t.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<Vec<_>>()
                })
            };
            if let Ok(t) = timings {
                let mut dedup: HashMap<&str, (tracing::Id, Timing, usize)> =
                    HashMap::with_capacity(t.len());
                for (id, timing) in t {
                    match dedup.get_mut(timing.metadata.name()) {
                        Some((i, span, count)) => {
                            if timing.ice() > span.ice() {
                                *i = id;
                                *span = timing;
                                *count += 1
                            }
                        }
                        None => {
                            dedup.insert(timing.metadata.name(), (id, timing, 1));
                        }
                    }
                }
                let mut t = dedup
                    .into_iter()
                    .map(|(_, (k, v, c))| (k, v, c))
                    .collect::<Vec<_>>();
                t.sort_by_cached_key(|t| t.1.ice());
                println!("--- Slow Spans ---");
                for (id, timing, count) in t.into_iter().rev().take(200) {
                    let parents = match dispatch
                        .downcast_ref::<Registry>()
                        .and_then(|r| r.span(&id))
                    {
                        Some(s) => s
                            .parents()
                            .map(|p| format!("{}:", p.name()))
                            .collect::<Vec<_>>(),
                        None => Vec::with_capacity(0),
                    };
                    let parents = parents.into_iter().rev().collect::<String>();

                    let file = match (timing.metadata.file(), timing.metadata.line()) {
                        (Some(f), Some(l)) => format!("{}:{}", f, l),
                        _ => String::new(),
                    };
                    println!(
                        "{} x {} - {}: idle: {:?}, busy {:?}, ice: {:?}, flame: {:?} {}",
                        count,
                        parents,
                        timing.metadata.name(),
                        timing.idle(),
                        timing.busy(),
                        timing.ice(),
                        timing.flame(),
                        file
                    );
                }
                println!("--- --- ---\n");
            }
        }
        None => dbg!(),
    })
}
