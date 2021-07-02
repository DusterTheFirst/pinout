use std::borrow::Cow;

use lexpr::Value;

pub type Text<'t> = Cow<'t, str>;

pub trait IntoText<'a>: lexpr::Index {
    fn text(&'a self) -> Text<'a>;
}

impl<'a> IntoText<'a> for Value {
    fn text(&'a self) -> Text<'a> {
        match self {
            Value::Nil | Value::Null => Cow::Borrowed(""),
            Value::Bool(b) => Cow::Owned(b.to_string()),
            Value::Number(n) => Cow::Owned(n.to_string()),
            Value::Char(c) => Cow::Owned(c.to_string()),
            Value::String(s) => Cow::Borrowed(&s),
            Value::Symbol(s) => Cow::Borrowed(&s),
            Value::Keyword(k) => Cow::Borrowed(&k),
            Value::Bytes(b) => String::from_utf8_lossy(&b),
            Value::Cons(_) | Value::Vector(_) => {
                unimplemented!("expected single value, found `{}`", self.to_string())
            }
        }
    }
}
