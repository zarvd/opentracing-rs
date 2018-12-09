#[derive(Debug)]
pub struct TraceId {
    pub(crate) low: u64,
    pub(crate) high: u64,
}

impl TraceId {
    pub fn new() -> Self {
        Self { low: 0, high: 0 }
    }
}

pub type Span = opentracing::Span<SpanState>;

#[derive(Debug)]
pub struct SpanState {
    trace_id: TraceId,
    span_id: u64,
}

impl SpanState {
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: 0,
        }
    }
}
