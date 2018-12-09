extern crate opentracing_rs;

use opentracing_rs::{
    jaeger::{ConstSampler, LoggingReporter, Tracer as JaegerTracer},
    Tracer,
};

fn main() {
    let sampler = ConstSampler::new(true);
    let reporter = LoggingReporter::new();

    let mut tracer = JaegerTracer::new("jaeger-example", sampler, reporter);
    {
        let span = tracer.start_span("hello");
    }
}
