use std::borrow::Cow;

use futures::{sync::mpsc, Future, Stream};
use opentracing::SpanBuilder;

use crate::{Reporter, Sampler, Span, SpanState};

pub struct Tracer<S, R>
where
    S: Sampler,
    R: Reporter,
{
    service_name: Cow<'static, str>,
    sampler: S,
    reporter: Option<R>,
    sender: mpsc::UnboundedSender<Span>,
    receiver: Option<mpsc::UnboundedReceiver<Span>>,
}

impl<S, R> Tracer<S, R>
where
    S: Sampler,
    R: Reporter,
{
    pub fn new<N>(service_name: N, sampler: S, reporter: R) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        let (sender, receiver) = mpsc::unbounded();

        Self {
            service_name: service_name.into(),
            sampler,
            sender,
            reporter: Some(reporter),
            receiver: Some(receiver),
        }
    }

    pub fn serve(&mut self) -> (impl Future<Item = (), Error = ()>) {
        let mut reporter = self.reporter.take().unwrap();

        self.receiver
            .take()
            .unwrap()
            .for_each(move |span| {
                reporter.report(&span);
                Ok(())
            })
            .map_err(|_| ())
    }
}

impl<S, R> opentracing::Tracer<SpanState> for Tracer<S, R>
where
    S: Sampler,
    R: Reporter,
{
    fn span<N>(&mut self, operation_name: N) -> SpanBuilder<SpanState>
    where
        N: Into<String>,
    {
        let state = SpanState::new();

        SpanBuilder::new(operation_name, state, self.sender.clone())
    }
}
