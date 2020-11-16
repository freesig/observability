use super::*;
#[derive(Debug, Clone)]
pub struct Context;

#[derive(Debug, Clone)]
pub struct WireContext {
    span_context: WireSpanContext,
    links: Option<WireLinks>,
}

#[derive(Debug, Clone, derive_more::From, derive_more::Into)]
pub struct WireLinks(pub Vec<WireLink>);

#[derive(Debug, Clone)]
pub struct WireLink;

#[derive(Debug, Clone)]
pub struct WireSpanContext;

impl OpenSpanExt for tracing::Span {
    fn get_current_context() -> Context {
        Context
    }
    fn get_context(&self) -> Context {
        Context
    }

    fn set_context(&self, _: Context) {}

    fn set_current_context(_: Context) {}

    fn display_context(&self) -> String {
        String::with_capacity(0)
    }
}

pub fn display_context(_: &Context) -> String {
    String::with_capacity(0)
}
