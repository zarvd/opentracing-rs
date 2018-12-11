#[derive(Clone, Debug)]
pub struct Tag {
    name: String,
    value: TagValue,
}

impl Tag {
    pub fn new<N, V>(name: N, value: V) -> Self
    where
        N: Into<String>,
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
    String(String),
    Bool(bool),
    Number(f64),
}

impl From<&'static str> for TagValue {
    fn from(f: &'static str) -> Self {
        TagValue::String(f.to_owned())
    }
}

impl From<String> for TagValue {
    fn from(f: String) -> Self {
        TagValue::String(f)
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
