use nu_plugin::LabeledError;
use nu_protocol::{Span, Spanned, Value};

pub fn highlight_do_something(
    param: Option<Spanned<String>>,
    val: &str,
    value_span: Span,
) -> Result<Value, LabeledError> {
    let a_val = match param {
        Some(p) => format!("Hello, {}! with value: {}", p.item, val),
        None => format!("Hello, Default! with value: {}", val),
    };
    Ok(Value::String {
        val: a_val,
        span: value_span,
    })
}
