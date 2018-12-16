extern crate futures;
extern crate opentracing_rs;
extern crate tokio;

use std::time::Duration;

use futures::lazy;

use opentracing_rs::{
    jaeger::{
        ConstSampler, LoggingReporter, RemoteReporter, Tracer as JaegerTracer, TransportProtocol,
        UdpTransport,
    },
    SpanBuilder, Tracer,
};

fn main() {
    let sampler = ConstSampler::new(true);
    let (transport, transport_serve) = UdpTransport::builder()
        .process_service_name("jaeger_example")
        .transport_protocol(TransportProtocol::ThriftCompact)
        .build_and_serve("127.0.0.1:6831".parse().unwrap());

    let reporter = RemoteReporter::new(transport);

    let interval_flush = reporter.interval_flush(Duration::from_millis(500));

    let mut tracer = JaegerTracer::new(sampler, reporter);

    tokio::run(lazy(move || {
        tokio::spawn(interval_flush);
        tokio::spawn(transport_serve);
        tokio::spawn(tracer.serve());
        {
            let span = tracer.span("hello").start();

            std::thread::sleep(std::time::Duration::from_secs(1));

            let child_span = tracer.span("testing 1").child_of(&span);
            {
                let _child_span = child_span.start();
                std::thread::sleep(std::time::Duration::from_secs(2));
            }

            let child_span = tracer.span("testing 2").child_of(&span);
            {
                let _child_span = child_span.start();
                std::thread::sleep(std::time::Duration::from_secs(2));
            }

            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        Ok(())
    }));
}
