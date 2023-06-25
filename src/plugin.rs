use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginSignature, Spanned, SyntaxShape, Value};

use crate::highlight::Highlighter;

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
        for (named, _) in call.named.iter() {
            if named.item.as_str() == "list-themes" {
                return Ok(highlighter.list_themes());
            }
        }

        dbg!(call);
        dbg!(input);

        let param: Option<Spanned<String>> = call.opt(0)?;

        let ret_val = match input {
            Value::String { val, span } => {
                crate::highlight::highlight_do_something(param, val, *span)?
            }
            v => {
                return Err(LabeledError {
                    label: "Expected something from pipeline".into(),
                    msg: format!("requires some input, got {}", v.get_type()),
                    span: Some(call.head)
                });
            }
        };

        Ok(ret_val)
    }
}
