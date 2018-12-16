Opentracing Rust Client
====

[![MIT licensed][mit-badge]][mit-url]
[![Travis Build Status][travis-badge]][travis-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE-MIT
[travis-badge]: https://travis-ci.org/ccc13/opentracing-rs.svg?branch=master
[travis-url]: https://travis-ci.org/ccc13/opentracing-rs

Features:
----
- support tokio and futures

Quickstart
----

TBD

Example
----

```rust
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
```

License
----

This project is licensed under the [MIT license](LICENSE).
