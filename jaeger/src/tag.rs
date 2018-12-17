pub const JAEGER_CLIENT_VERSION_TAG_KEY: &str = "jaeger.version";
pub const JAEGER_CLIENT_VERSION: &str = concat!("opentracing-rs-", env!("CARGO_PKG_VERSION"));

pub const SAMPLER_TYPE_TAG_KEY: &str = "sampler.type";
pub const SAMPLER_PARAM_TAG_KEY: &str = "sampler.param";
pub const SAMPLER_TYPE_CONST: &str = "const";

pub const SAMPLER_TYPE_REMOTE: &str = "remote";
pub const SAMPLER_TYPE_PROBABILISTIC: &str = "probabilistic";
