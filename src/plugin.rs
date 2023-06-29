use std::env;

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginSignature, Span, Spanned, SyntaxShape, Value};

use crate::highlight::Highlighter;

const THEME_ENV: &str = "NU_PLUGIN_HIGHLIGHT_THEME";

pub struct HighlightPlugin;

impl HighlightPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for HighlightPlugin {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("highlight")
            .usage("View highlight results")
            .optional(
                "language",
                SyntaxShape::String,
                "language or file extension to help language detection"
            )
            .named(
                "theme",
                SyntaxShape::String,
                "theme used for highlighting",
                Some('t')
            )
            .switch("list-themes", "list all possible themes", None)
            .category(Category::Strings)]
    }

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        input: &Value
    ) -> Result<Value, LabeledError> {
        assert_eq!(name, "highlight");
        let highlighter = Highlighter::new();

        // ignore everything else and return the list of themes
        if call.has_flag("list-themes") {
            return Ok(highlighter.list_themes().into());
        }

        // use environment variable if available, override with passed theme
        let mut theme = env::var(THEME_ENV).ok();
        if let Some(theme_value) = call.get_flag_value("theme") {
            match theme_value {
                Value::String { val, span } => match highlighter.is_valid_theme(&val) {
                    true => theme = Some(val),
                    false => {
                        return Err(LabeledError {
                            label: "Unknown theme, use `highlight --list-themes` to list all \
                                    themes"
                                .into(),
                            msg: "unknown theme".into(),
                            span: Some(span)
                        })
                    }
                },

                _ => {
                    return Err(LabeledError {
                        label: "Expected theme value to be a string".into(),
                        msg: format!("expected string, got {}", theme_value.get_type()),
                        span: Some(theme_value.expect_span())
                    })
                }
            }
        }

        // extract language parameter, doesn't need any validation
        let param: Option<Spanned<String>> = call.opt(0)?;
        let language = param.map(|Spanned { item, .. }| item);

        // try to highlight if input is a string
        let ret_val = match input {
            Value::String { val, .. } => {
                Value::string(highlighter.highlight(val, &language, &theme), call.head)
            }
            v => {
                return Err(LabeledError {
                    label: "Expected source code as string from pipeline".into(),
                    msg: format!("expected string, got {}", v.get_type()),
                    span: Some(call.head)
                });
            }
        };

        Ok(ret_val)
    }
}
