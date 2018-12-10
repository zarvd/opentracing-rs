extern crate futures;
extern crate opentracing_rs;
extern crate tokio;

use futures::lazy;

use opentracing_rs::{
    jaeger::{ConstSampler, LoggingReporter, Tracer as JaegerTracer},
    Tracer,
};

fn main() {
    let sampler = ConstSampler::new(true);
    let reporter = LoggingReporter::new();

    let mut tracer = JaegerTracer::new("jaeger-example", sampler, reporter);

    tokio::run(lazy(move || {
        tokio::spawn(tracer.serve());
        {
            let span = tracer.span("hello").start();

            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        Ok(())
    }));
}
