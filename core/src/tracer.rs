use crate::SpanBuilder;

pub trait Tracer<S, B>
where
    S: Send + Sync,
    B: SpanBuilder<S>,
{
    fn span<O>(&mut self, operation_name: O) -> B
    where
        O: Into<String>;
}
