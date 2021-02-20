use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use tracing::span::Attributes;
use tracing::Metadata;
use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

pub struct TimingLayer {
    timings: std::sync::RwLock<HashMap<tracing::span::Id, Timing>>,
    latency: std::sync::RwLock<Vec<Latency>>,
}

#[derive(Debug, Clone)]
pub struct Timing {
    start: Instant,
    enter: Option<Instant>,
    busy: Duration,
    metadata: &'static Metadata<'static>,
    parents: String,
    parent: Option<tracing::span::Id>,
    closing: bool,
    p_count: usize,
}

#[derive(Debug, Clone)]
pub struct Latency {
    id: tracing::span::Id,
    metadata: &'static Metadata<'static>,
    latency: Duration,
}

impl Timing {
    fn new(metadata: &'static Metadata<'static>) -> Self {
        Self {
            start: Instant::now(),
            enter: None,
            busy: Duration::default(),
            metadata,
            parents: String::new(),
            parent: None,
            closing: false,
            p_count: 0,
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
            latency: std::sync::RwLock::new(Vec::new()),
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
        // let parents = ctx
        //     .scope()
        //     .map(|p| format!("{}:", p.name()))
        //     .collect::<Vec<_>>();
        // let parents = ctx
        //     .span(&id)
        //     .map(|s| {
        //         s.parents()
        //             .map(|p| format!("{}:", p.name()))
        //             .collect::<Vec<_>>()
        //     })
        //     .unwrap_or_default();

        // let parents = parents.into_iter().rev().collect::<String>();
        // let parents = ctx.current_span().metadata().map(|m|m.name().to_string()).unwrap_or_default();

        if let Ok(mut t) = self.timings.write() {
            if let Some(t) = t.get_mut(id) {
                // t.parents = parents;
                t.enter = Some(Instant::now());
            }
        }
    }

    fn on_exit(&self, id: &tracing::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        // let parents = ctx.current_span().metadata().map(|m|m.name().to_string()).unwrap_or_default();
        let parent = ctx.current_span().id().cloned();
        if let Ok(mut t) = self.timings.write() {
            if let Some(p) = &parent {
                if let Some(p) = t.get_mut(p) {
                    p.p_count += 1;
                }
            }
            if let Some(t) = t.get_mut(id) {
                t.parent = parent;
                if let Some(enter) = t.enter.take() {
                    t.busy += enter.elapsed();
                }
            }
        }
    }

    fn on_close(&self, id: tracing::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Ok(mut t) = self.timings.write() {
            let mut close_p = false;
            let (p, p_count) = t
                .get(&id)
                .map(|t| (t.parent.clone(), Some(t.p_count)))
                .unwrap_or((None, None));
            if let Some(p) = p {
                if let Some(p) = t.get_mut(&p) {
                    if let Some(s) = p.p_count.checked_sub(1) {
                        p.p_count = s;
                        close_p = s == 0 && p.closing;
                    }
                }
                if close_p {
                    if let Some(t) = t.remove(&p) {
                        let latency = t.start.elapsed();
                        if latency.as_millis() > 1 {
                            if let Ok(mut l) = self.latency.write() {
                                l.push(Latency {
                                    id: p.clone(),
                                    metadata: t.metadata,
                                    latency,
                                });
                            }
                        }
                    }
                }
            }
            if let Some(p_count) = p_count {
                if p_count == 0 {
                    if let Some(t) = t.remove(&id) {
                        let latency = t.start.elapsed();
                        if latency.as_millis() > 1 {
                            if let Ok(mut l) = self.latency.write() {
                                l.push(Latency {
                                    id: id.clone(),
                                    metadata: t.metadata,
                                    latency,
                                });
                            }
                        }
                    }
                } else {
                    t.get_mut(&id).map(|t| t.closing = true);
                }
            }
        }
    }
}

/// Publish deadlock statistics
pub fn tick_deadlock_catcher() {
    tracing::dispatcher::get_default(|dispatch| match dispatch.downcast_ref::<TimingLayer>() {
        Some(timing) => {
            let timings = {
                timing.timings.read().map(|t| {
                    (
                        t.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect::<Vec<_>>(),
                        t.clone(),
                    )
                })
            };
            let latencies = {
                timing.latency.write().map(|mut l| {
                    let lat = l.clone();
                    l.clear();
                    lat
                })
            };
            if let Ok(mut latencies) = latencies {
                latencies.sort_unstable_by_key(|l| l.latency);
                println!("--- Latencies ---");
                for Latency {
                    id: _,
                    metadata,
                    latency,
                // } in latencies.into_iter().rev().take(100)
                } in latencies.into_iter().rev()
                {
                    println!(
                        "{} {:?} {}:{}",
                        metadata.name(),
                        latency,
                        metadata.file().unwrap_or_default(),
                        metadata.line().unwrap_or_default(),
                    );
                }
            }
            if let Ok((t, map)) = timings {
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
                for (_id, timing, count) in t.into_iter().rev().take(200) {
                    let mut parents = Vec::new();
                    let mut parent = timing.parent.clone();
                    while let Some(p) = parent.take() {
                        if let Some(tim) = map.get(&p) {
                            parents.push(format!("{}: ", tim.metadata.name()));
                            parent = tim.parent.clone();
                        }
                    }
                    // let parents = dispatch
                    //     .downcast_ref::<Registry>()
                    //     .and_then(|r| {
                    //         r.span(&id).map(|s| {
                    //             s.parents()
                    //                 .map(|p| format!("{}:", p.name()))
                    //                 .collect::<Vec<_>>()
                    //         })
                    //     })
                    //     .unwrap_or_default();

                    let parents = parents.into_iter().rev().collect::<String>();
                    let file = match (timing.metadata.file(), timing.metadata.line()) {
                        (Some(f), Some(l)) => format!("{}:{}", f, l),
                        _ => String::new(),
                    };
                    println!(
                        "{} x {} - {}: idle: {:?}, busy {:?}, ice: {:?}, flame: {:?} {}",
                        count,
                        // timing.parents,
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
