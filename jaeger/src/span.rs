use std::sync::Arc;
use std::time::SystemTime;

use futures::sync::mpsc;

use opentracing_rs_core::{BaggageItem, Tag};

use crate::Sampler;

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
    pub(crate) trace_id: TraceId,
    pub(crate) span_id: u64,
    pub(crate) parent_span_id: Option<u64>,
    pub(crate) is_sampled: bool,
}

impl SpanState {
    pub fn new(trace_id: TraceId, span_id: u64, is_sampled: bool) -> Self {
        Self {
            trace_id,
            span_id,
            is_sampled,
            parent_span_id: None,
        }
    }

    pub fn from_parent(parent: Self) -> Self {
        Self {
            trace_id: parent.trace_id,
            span_id: rand::random(),
            parent_span_id: Some(parent.span_id),
            is_sampled: parent.is_sampled,
        }
    }
}

impl Default for SpanState {
    fn default() -> Self {
        Self::new(TraceId::new(), rand::random(), false)
    }
}

pub struct SpanBuilder {
    sender: mpsc::UnboundedSender<Span>,
    operation_name: String,
    start_time: Option<SystemTime>,
    tags: Vec<Tag>,
    references: Vec<SpanReference>,
    baggage_items: Vec<BaggageItem>,
    sampler: Arc<Sampler>,
}

impl SpanBuilder {
    pub fn new<N>(
        operation_name: N,
        sampler: Arc<Sampler>,
        sender: mpsc::UnboundedSender<Span>,
    ) -> Self
    where
        N: Into<String>,
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
            sampler,
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

    fn start(mut self) -> Span {
        let state = {
            let mut state = None;

            for reference in &self.references {
                match reference {
                    opentracing_rs_core::SpanReference::ChildOf(parent) => {
                        state = Some(SpanState::from_parent(parent.clone()))
                    }
                    _ => {}
                }
            }
            match state {
                Some(state) => state,
                None => {
                    let trace_id = TraceId::new();
                    let span_id = rand::random();
                    let (is_sampled, tags) =
                        self.sampler.is_sampled(&trace_id, &self.operation_name);

                    self.tags.extend_from_slice(tags);

                    let state = SpanState::new(trace_id, span_id, is_sampled);
                    state
                }
            }
        };

        Span::new(
            self.sender,
            self.operation_name,
            self.start_time.unwrap_or_else(SystemTime::now),
            self.tags,
            self.references,
            state,
            self.baggage_items,
        )
    }
}
