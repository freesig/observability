use super::*;

/// TODO
pub struct MsgWrap<T> {
    t: T,
    context: Option<Context>,
}

impl<T> MsgWrap<T> {
    /// TODO
    pub fn inner(self) -> T {
        if let Some(context) = self.context {
            tracing::Span::set_current_context(context);
        }
        self.t
    }
    /// TODO
    pub fn without_context(self) -> T {
        self.t
    }
}

impl<T> From<T> for MsgWrap<T> {
    fn from(t: T) -> Self {
        let span = tracing::Span::current();
        let context = if span.is_disabled() {
            None
        } else {
            Some(span.get_context())
        };

        Self { t, context }
    }
}

impl<T> std::fmt::Debug for MsgWrap<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.t))
    }
}
