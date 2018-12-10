use crate::SpanBuilder;

pub trait Tracer<S>
where
    S: Send + Sync,
{
    fn span<O>(&mut self, operation_name: O) -> SpanBuilder<S>
    where
        O: Into<String>;
}
