use std::time::SystemTime;

use futures::sync::mpsc;
use futures::{Future, Sink};

use crate::Tag;

#[derive(Debug)]
enum SpanReference<S> {
    ChildOf(S),
    FollowsFrom(S),
}

#[derive(Debug)]
pub struct Span<S>
where
    S: 'static + Sync + Send,
{
    inner: Option<Inner<S>>,
}

impl<S> Span<S>
where
    S: 'static + Send + Sync,
{
    fn new<O>(
        sender: mpsc::UnboundedSender<Span<S>>,
        operation_name: O,
        start_time: SystemTime,
        tags: Vec<Tag>,
        references: Vec<SpanReference<S>>,
        state: S,
        baggage_items: Vec<BaggageItem>,
    ) -> Self
    where
        O: Into<String>,
    {
        let context = SpanContext::new(state, baggage_items);
        let operation_name = operation_name.into();
        let finish_time = None;
        let inner = Some(Inner {
            sender,
            operation_name,
            start_time,
            finish_time,
            tags,
            references,
            context,
        });

        Self { inner }
    }

    pub fn context(&self) -> &SpanContext<S> {
        &self.inner.as_ref().unwrap().context
    }

    pub fn set_operation_name<O>(&mut self, op_name: O)
    where
        O: Into<String>,
    {
        if let Some(inner) = self.inner.as_mut() {
            inner.operation_name = op_name.into();
        }
    }

    pub fn operation_name(&self) -> &str {
        self.inner.as_ref().unwrap().operation_name.as_ref()
    }

    pub fn set_tag(&mut self, tag: Tag) {
        if let Some(inner) = self.inner.as_mut() {
            inner.tags.push(tag);
        }
    }

    pub fn is_finished(&self) -> bool {
        self.inner.as_ref().unwrap().finish_time.is_some()
    }

    pub fn finish(&mut self) {
        if self.inner.is_none() || self.is_finished() {
            return;
        }
        let inner = self.inner.as_mut().unwrap();
        inner.finish_time = Some(SystemTime::now());
        let sender = inner.sender.clone();
        tokio::spawn(
            sender
                .send(Span {
                    inner: self.inner.take(),
                })
                .map(|_| ())
                .map_err(|_| ()),
        );
    }
}

impl<S> Drop for Span<S>
where
    S: 'static + Send + Sync,
{
    fn drop(&mut self) {
        self.finish()
    }
}

#[derive(Debug)]
struct Inner<S>
where
    S: 'static + Send + Sync,
{
    sender: mpsc::UnboundedSender<Span<S>>,
    operation_name: String,
    start_time: SystemTime,
    finish_time: Option<SystemTime>,
    tags: Vec<Tag>,
    references: Vec<SpanReference<S>>,
    context: SpanContext<S>,
}

#[derive(Debug)]
pub struct SpanContext<S> {
    state: S,
    baggage_items: Vec<BaggageItem>,
}

impl<S> SpanContext<S> {
    pub fn new(state: S, baggage_items: Vec<BaggageItem>) -> Self {
        Self {
            state,
            baggage_items,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BaggageItem {
    key: String,
    value: String,
}

impl BaggageItem {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_owned(),
            value: value.to_owned(),
        }
    }
}

pub struct SpanBuilder<S>
where
    S: 'static + Send + Sync,
{
    sender: mpsc::UnboundedSender<Span<S>>,
    operation_name: String,
    start_time: Option<SystemTime>,
    tags: Vec<Tag>,
    state: S,
    references: Vec<SpanReference<S>>,
    baggage_items: Vec<BaggageItem>,
}

impl<S> SpanBuilder<S>
where
    S: 'static + Send + Sync,
{
    pub fn new<O>(operation_name: O, state: S, sender: mpsc::UnboundedSender<Span<S>>) -> Self
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
            state,
            references,
            start_time: None,
        }
    }

    pub fn start_time(mut self, time: SystemTime) -> Self {
        self.start_time = Some(time);
        self
    }

    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn child_of(mut self, span: &Span<S>) -> Self
    where
        S: Clone,
    {
        self.baggage_items
            .extend(span.inner.as_ref().unwrap().context.baggage_items.clone());
        self.references
            .push(SpanReference::ChildOf(span.context().state.clone()));
        self
    }

    pub fn start(self) -> Span<S> {
        Span::new(
            self.sender,
            self.operation_name,
            self.start_time.unwrap_or_else(SystemTime::now),
            self.tags,
            self.references,
            self.state,
            self.baggage_items,
        )
    }
}
