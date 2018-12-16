use futures::{sync::mpsc, Future, Stream};

use crate::{Reporter, Sampler, Span, SpanBuilder, SpanState};

use opentracing_rs_core::Tag;

#[derive(Clone)]
pub struct Process {
    pub(crate) service_name: String,
    pub(crate) tags: Vec<Tag>,
}

impl Process {
    pub fn new<N>(service_name: N) -> Self
    where
        N: Into<String>,
    {
        let tags = vec![Tag::new(
            crate::tag::JAEGER_CLIENT_VERSION_TAG_KEY,
            crate::tag::JAEGER_CLIENT_VERSION,
        )];

        Self::with_tags(service_name, tags)
    }

    pub fn with_tags<N>(service_name: N, tags: Vec<Tag>) -> Self
    where
        N: Into<String>,
    {
        Self {
            service_name: service_name.into(),
            tags,
        }
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }
}

pub struct Tracer<S, R>
where
    S: Sampler,
    R: Reporter,
{
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
    pub fn new(sampler: S, reporter: R) -> Self {
        let (sender, receiver) = mpsc::unbounded();

        Self {
            sampler,
            sender,
            reporter: Some(reporter),
            receiver: Some(receiver),
        }
    }

    pub fn serve(&mut self) -> impl Future<Item = (), Error = ()> {
        let mut reporter = self.reporter.take().unwrap();

        self.receiver
            .take()
            .unwrap()
            .for_each(move |span| {
                reporter.report(span);
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
