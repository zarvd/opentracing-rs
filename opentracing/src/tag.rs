use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Tag {
    name: Cow<'static, str>,
    value: TagValue,
}

impl Tag {
    pub fn new<N, V>(name: N, value: V) -> Self
    where
        N: Into<Cow<'static, str>>,
        V: Into<TagValue>,
    {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TagValue {
    String(Cow<'static, str>),
    Bool(bool),
    Number(f64),
}

impl From<&'static str> for TagValue {
    fn from(f: &'static str) -> Self {
        TagValue::String(Cow::Borrowed(f))
    }
}

impl From<String> for TagValue {
    fn from(f: String) -> Self {
        TagValue::String(Cow::Owned(f))
    }
}

impl From<bool> for TagValue {
    fn from(f: bool) -> Self {
        TagValue::Bool(f)
    }
}

impl From<i64> for TagValue {
    fn from(f: i64) -> Self {
        TagValue::Number(f as f64)
    }
}

impl From<f64> for TagValue {
    fn from(f: f64) -> Self {
        TagValue::Number(f)
    }
}
