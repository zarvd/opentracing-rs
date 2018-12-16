use opentracing_rs_core::{Tag, TagValue};

use crate::{thrift_gen::jaeger, Process, Span, SpanBatch};

impl From<Tag> for jaeger::Tag {
    fn from(tag: Tag) -> Self {
        let (name, value) = tag.split();
        match value {
            TagValue::Bool(v) => Self::new(name, jaeger::TagType::BOOL, None, None, v, None, None),
            TagValue::String(v) => {
                Self::new(name, jaeger::TagType::STRING, v, None, None, None, None)
            }
            TagValue::Number(v) => Self::new(
                name,
                jaeger::TagType::DOUBLE,
                None,
                None,
                None,
                v as i64,
                None,
            ),
        }
    }
}

impl From<Span> for jaeger::Span {
    fn from(span: Span) -> Self {
        let ctx = span.context();
        let state = ctx.state();

        let start_time = {
            let ts = span
                .start_time()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            (ts.as_secs() * 1_000_000 + (ts.subsec_nanos() as u64 / 1000)) as i64
        };

        let duration = {
            let duration = span.duration();

            (duration.as_secs() * 1_000_000 + (duration.subsec_nanos() as u64 / 1000)) as i64
        };

        let flags = 0;
        jaeger::Span::new(
            state.trace_id.low as i64,
            state.trace_id.high as i64,
            state.span_id as i64,
            state.parent_span_id.unwrap_or_default() as i64,
            span.operation_name().to_owned(),
            None,
            flags,
            start_time,
            duration,
            None,
            None,
            None,
        )
    }
}

impl From<Process> for jaeger::Process {
    fn from(process: Process) -> Self {
        let tags = {
            let mut tags = Vec::with_capacity(process.tags.len());

            for t in process.tags {
                tags.push(From::from(t));
            }
            tags
        };

        jaeger::Process {
            service_name: process.service_name,
            tags: Some(tags),
        }
    }
}

impl From<SpanBatch> for jaeger::Batch {
    fn from(batch: SpanBatch) -> Self {
        let spans = {
            let mut spans = Vec::with_capacity(batch.spans.len());

            for s in batch.spans {
                spans.push(From::from(s));
            }
            spans
        };

        jaeger::Batch::new(From::from(batch.process), spans)
    }
}
