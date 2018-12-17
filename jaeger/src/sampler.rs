use opentracing_rs_core::Tag;

use crate::{tag, TraceId};

pub trait Sampler: Send + Sync {
    fn is_sampled(&self, trace_id: &TraceId, operation: &str) -> (bool, &[Tag]);
}

pub struct ConstSampler {
    decision: bool,
    tags: Vec<Tag>,
}

impl ConstSampler {
    pub fn new(sample: bool) -> Self {
        let mut tags = Vec::with_capacity(2);
        tags.push(Tag::new(tag::SAMPLER_TYPE_TAG_KEY, tag::SAMPLER_TYPE_CONST));

        Self {
            decision: sample,
            tags,
        }
    }
}

impl Sampler for ConstSampler {
    fn is_sampled(&self, _trace_id: &TraceId, _operation: &str) -> (bool, &[Tag]) {
        (self.decision, &self.tags)
    }
}

pub struct ProbabilisticSampler {
    sampling_rate: f64,
    sampling_boundary: u64,
    tags: Vec<Tag>,
}

impl ProbabilisticSampler {
    pub fn new(sampling_rate: f64) -> Self {
        if sampling_rate < 0.0 || sampling_rate > 1.0 {
            panic!(
                "Sampling Rate must be between 0.0 and 1.0, received {}",
                sampling_rate
            );
        }

        let tags = vec![
            Tag::new(tag::SAMPLER_TYPE_TAG_KEY, tag::SAMPLER_TYPE_PROBABILISTIC),
            Tag::new(tag::SAMPLER_PARAM_TAG_KEY, sampling_rate),
        ];

        let sampling_boundary = (std::u64::MAX as f64 * sampling_rate) as u64;
        Self {
            sampling_boundary,
            sampling_rate,
            tags,
        }
    }

    pub fn sampling_rate(&self) -> f64 {
        self.sampling_rate
    }
}

impl Sampler for ProbabilisticSampler {
    fn is_sampled(&self, trace_id: &TraceId, _operation: &str) -> (bool, &[Tag]) {
        (self.sampling_boundary > trace_id.low, &self.tags)
    }
}
