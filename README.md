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
    tokio::run(lazy(move || {
        // build and serve tracer
        let mut tracer = JaegerTracer::builder()
            .probabilistic_sampler(0.50)
            .udp_remote_reporter(
                "jaeger_example",
                "127.0.0.1:6831".parse().unwrap(),
                TransportProtocol::ThriftCompact,
                Duration::from_millis(500),
            )
            .build_and_serve();
            
        // clone tracer into event callback
        // start tracing
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

        Ok(())
    }));
}

```

License
----

This project is licensed under the [MIT license](LICENSE).
