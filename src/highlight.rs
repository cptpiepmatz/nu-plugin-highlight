use bat::assets::HighlightingAssets;
use nu_plugin::LabeledError;
use nu_protocol::{Span, Spanned, Value};

pub struct Highlighter {
    highlighting_assets: HighlightingAssets
}

impl Highlighter {
    pub fn new() -> Self {
        Highlighter {
            highlighting_assets: HighlightingAssets::from_binary()
        }
    }

    pub fn list_themes(&self) -> Value {
        let ha = &self.highlighting_assets;
        let default_theme_id = HighlightingAssets::default_theme();
        let vals = ha
            .themes()
            .map(|t_id| {
                let theme = ha.get_theme(t_id);
                Value::Record {
                    cols: vec![
                        "id".to_owned(),
                        "name".to_owned(),
                        "author".to_owned(),
                        "default".to_owned(),
                    ],
                    vals: vec![
                        Value::string(t_id, Span::unknown()),
                        Value::string(theme.name.clone().unwrap_or(String::new()), Span::unknown()),
                        Value::string(
                            theme.author.clone().unwrap_or(String::new()),
                            Span::unknown()
                        ),
                        Value::boolean(t_id == default_theme_id, Span::unknown()),
                    ],
                    span: Span::unknown()
                }
            })
            .collect();
        Value::list(vals, Span::unknown())
    }
}

pub fn highlight_do_something(
    param: Option<Spanned<String>>,
    val: &str,
    value_span: Span
) -> Result<Value, LabeledError> {
    let a_val = match param {
        Some(p) => format!("Hello, {}! with value: {}", p.item, val),
        None => format!("Hello, Default! with value: {}", val)
    };
    Ok(Value::String {
        val: a_val,
        span: value_span
    })
}
