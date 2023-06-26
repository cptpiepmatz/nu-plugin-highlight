use bat::assets::HighlightingAssets;
use nu_plugin::LabeledError;
use nu_protocol::{Span, Spanned, Value};
use crate::theme::{ListThemes, ThemeDescription};

pub struct Highlighter {
    highlighting_assets: HighlightingAssets
}

impl Highlighter {
    pub fn new() -> Self {
        Highlighter {
            highlighting_assets: HighlightingAssets::from_binary()
        }
    }

    pub fn list_themes(&self) -> ListThemes {
        let ha = &self.highlighting_assets;
        let default_theme_id = HighlightingAssets::default_theme();
        ListThemes(ha.themes().map(|t_id| {
            let theme = ha.get_theme(t_id);
            ThemeDescription {
                id: t_id.to_owned(),
                name: theme.name.clone(),
                author: theme.author.clone(),
                default: default_theme_id == t_id,
            }
        }).collect())
    }

    //pub fn highlight(&self, input: &str, language: &Into<Option<String>>) -> String {}
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
