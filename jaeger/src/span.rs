use std::time::SystemTime;

use futures::sync::mpsc;

use opentracing_rs_core::{BaggageItem, Tag};

pub type Span = opentracing_rs_core::Span<SpanState>;
pub type SpanReference = opentracing_rs_core::SpanReference<SpanState>;

#[derive(Debug, Clone)]
pub struct TraceId {
    pub(crate) low: u64,
    pub(crate) high: u64,
}

impl TraceId {
    pub fn new() -> Self {
        Self {
            low: rand::random(),
            high: rand::random(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpanState {
    trace_id: TraceId,
    span_id: u64,
}

impl SpanState {
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: rand::random(),
        }
    }

    pub fn from_parent(parent: Self) -> Self {
        Self {
            trace_id: parent.trace_id,
            span_id: rand::random(),
        }
    }
}

impl Default for SpanState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SpanBuilder {
    sender: mpsc::UnboundedSender<Span>,
    operation_name: String,
    start_time: Option<SystemTime>,
    tags: Vec<Tag>,
    references: Vec<SpanReference>,
    baggage_items: Vec<BaggageItem>,
}

impl SpanBuilder {
    pub fn new<O>(operation_name: O, sender: mpsc::UnboundedSender<Span>) -> Self
    where
        O: Into<String>,
    {
        let operation_name = operation_name.into();
        let baggage_items = Vec::new();
        let tags = Vec::new();
        let references = Vec::new();
        Self {
            sender,
            operation_name,
            baggage_items,
            tags,
            references,
            start_time: None,
        }
    }
}

impl opentracing_rs_core::SpanBuilder<SpanState> for SpanBuilder {
    fn start_time(mut self, time: SystemTime) -> Self {
        self.start_time = Some(time);
        self
    }

    fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    fn child_of(mut self, span: &Span) -> Self {
        self.baggage_items
            .extend(span.context().baggage_items().clone());
        self.references
            .push(opentracing_rs_core::SpanReference::ChildOf(
                span.context().state().clone(),
            ));
        self
    }

    fn start(self) -> Span {
        let mut state = None;

        for reference in &self.references {
            match reference {
                opentracing_rs_core::SpanReference::ChildOf(parent) => {
                    state = Some(SpanState::from_parent(parent.clone()))
                }
                _ => {}
            }
        }

        Span::new(
            self.sender,
            self.operation_name,
            self.start_time.unwrap_or_else(SystemTime::now),
            self.tags,
            self.references,
            state.unwrap_or_else(SpanState::default),
            self.baggage_items,
        )
    }
}
