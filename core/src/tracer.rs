pub trait Tracer {
    type SpanState;
    type SpanBuilder;

    fn span<N>(&mut self, operation_name: N) -> Self::SpanBuilder
    where
        N: Into<String>;
}
