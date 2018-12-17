extern crate futures;
extern crate opentracing_rs;
extern crate tokio;

use std::time::Duration;

use futures::lazy;

use opentracing_rs::{
    jaeger::{Tracer as JaegerTracer, TransportProtocol},
    SpanBuilder, Tracer,
};

fn main() {
    tokio::run(lazy(move || {
        let mut tracer = JaegerTracer::builder()
            .probabilistic_sampler(0.50)
            .udp_remote_reporter(
                "jaeger_example",
                "127.0.0.1:6831".parse().unwrap(),
                TransportProtocol::ThriftCompact,
                Duration::from_millis(500),
            )
            .build_and_serve();
        {
            let mut tracer = tracer.clone();
            tokio::spawn(lazy(move || {
                let span = tracer.span("hello 1").start();

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

                Ok(())
            }));
        }

        tokio::spawn(lazy(move || {
            let span = tracer.span("hello 2").start();

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
            Ok(())
        }));

        Ok(())
    }));
}
