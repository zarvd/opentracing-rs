use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::{sync::mpsc, Future, Stream};

use opentracing_rs_core::Tag;

use crate::{Reporter, Sampler, Span, SpanBuilder, SpanState, TransportProtocol};

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

#[derive(Clone)]
pub struct Tracer {
    sampler: Arc<Sampler>,
    sender: mpsc::UnboundedSender<Span>,
}

impl Tracer {
    pub fn new(
        sampler: Arc<Sampler>,
        mut reporter: Box<Reporter>,
    ) -> (Self, impl Future<Item = (), Error = ()>) {
        let (sender, receiver) = mpsc::unbounded();

        let tracer = Self { sampler, sender };
        let serve = receiver
            .for_each(move |span| {
                reporter.report(span);
                Ok(())
            })
            .map_err(|_| ());

        (tracer, serve)
    }

    pub fn builder() -> TracerBuilder {
        TracerBuilder::default()
    }
}

impl opentracing_rs_core::Tracer for Tracer {
    type SpanState = SpanState;
    type SpanBuilder = SpanBuilder;

    fn span<N>(&mut self, operation_name: N) -> Self::SpanBuilder
    where
        N: Into<String>,
    {
        SpanBuilder::new(operation_name, self.sampler.clone(), self.sender.clone())
    }
}

#[derive(Default)]
pub struct TracerBuilder {
    sampler: Option<Arc<Sampler>>,
    reporter: Option<Box<Reporter>>,
    reporter_serve: Option<Box<Future<Item = (), Error = ()> + Send>>,
}

impl TracerBuilder {
    pub fn const_sampler(mut self, sample: bool) -> Self {
        use crate::ConstSampler;
        self.sampler = Some(Arc::new(ConstSampler::new(sample)));
        self
    }

    pub fn probabilistic_sampler(mut self, sampling_rate: f64) -> Self {
        use crate::ProbabilisticSampler;
        self.sampler = Some(Arc::new(ProbabilisticSampler::new(sampling_rate)));
        self
    }

    pub fn udp_remote_reporter<N>(
        mut self,
        service_name: N,
        socket_addr: SocketAddr,
        protocol: TransportProtocol,
        flush_interval: Duration,
    ) -> Self
    where
        N: Into<String>,
    {
        use crate::{RemoteReporter, UdpTransport};
        let (transport, serve) = UdpTransport::builder()
            .process_service_name(service_name)
            .transport_protocol(protocol)
            .build_and_serve(socket_addr);
        let reporter = Box::new(RemoteReporter::new(transport));
        self.reporter_serve = Some(Box::new(
            serve
                .join(reporter.interval_flush(flush_interval))
                .map(|_| ()),
        ));
        self.reporter = Some(reporter);

        self
    }

    pub fn build_and_serve(self) -> Tracer {
        let (tracer, tracer_serve) = Tracer::new(self.sampler.unwrap(), self.reporter.unwrap());
        tokio::spawn(self.reporter_serve.unwrap());
        tokio::spawn(tracer_serve);

        tracer
    }
}
