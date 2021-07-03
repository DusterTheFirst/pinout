use std::borrow::Cow;

use lexpr::Value;

pub type Text<'t> = Cow<'t, str>;

pub trait IntoText<'a>: lexpr::Index {
    fn into_text_internal(&'a self, join: bool) -> Text<'a>;
    fn text(&'a self) -> Text<'a> {
        self.into_text_internal(false)
    }
    fn text_join(&'a self) -> Text<'a> {
        self.into_text_internal(true)
    }
}

impl<'a> IntoText<'a> for Value {
    fn into_text_internal(&'a self, join: bool) -> Text<'a> {
        match self {
            Value::Nil | Value::Null => Cow::Borrowed(""),
            Value::Bool(b) => Cow::Owned(b.to_string()),
            Value::Number(n) => Cow::Owned(n.to_string()),
            Value::Char(c) => Cow::Owned(c.to_string()),
            Value::String(s) => Cow::Borrowed(&s),
            Value::Symbol(s) => Cow::Borrowed(&s),
            Value::Keyword(k) => Cow::Borrowed(&k),
            Value::Bytes(b) => String::from_utf8_lossy(&b),
            Value::Cons(c) if join => {
                let (left, right) = c.as_pair();

                if right.is_nil() || right.is_null() {
                    return left.into_text_internal(join);
                }

                let mut joined = left.into_text_internal(join).to_string();
                joined.push_str(&right.into_text_internal(join));

                Cow::Owned(joined)
            }
            Value::Vector(v) if join => {
                Cow::Owned(v.iter().map(|v| v.into_text_internal(join)).collect::<String>())
            }
            _ => unimplemented!(
                "expected single value, found `{}`. Use `text_join` if these should be one",
                self.to_string()
            ),
        }
    }
}
