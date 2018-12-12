use std::borrow::Cow;

use futures::{sync::mpsc, Future, Stream};

use crate::{Reporter, Sampler, Span, SpanBuilder, SpanState};

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

impl<S, R> opentracing_rs_core::Tracer for Tracer<S, R>
where
    S: Sampler,
    R: Reporter,
{
    type SpanState = SpanState;
    type SpanBuilder = SpanBuilder;

    fn span<N>(&mut self, operation_name: N) -> Self::SpanBuilder
    where
        N: Into<String>,
    {
        SpanBuilder::new(operation_name, self.sender.clone())
    }
}
