use std::borrow::Cow;

use crate::SpanBuilder;

pub trait Tracer<S> {
    fn span<O>(&mut self, operation_name: O) -> SpanBuilder<S>
    where
        O: Into<Cow<'static, str>>;
}
