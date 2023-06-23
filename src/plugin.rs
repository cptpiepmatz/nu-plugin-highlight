use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Spanned, SyntaxShape, Value};

pub struct Highlight;

impl Highlight {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for Highlight {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("highlight")
            .usage("View highlight results")
            .required("path", SyntaxShape::String, "path to highlight input file")
            .category(Category::Experimental)
            .plugin_examples(vec![PluginExample {
                description: "This is the example descripion".into(),
                example: "some pipeline involving highlight".into(),
                result: None,
            }])]
    }

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        assert_eq!(name, "highlight");
        let param: Option<Spanned<String>> = call.opt(0)?;

        let ret_val = match input {
            Value::String { val, span } => {
                crate::highlight::highlight_do_something(param, val, *span)?
            }
            v => {
                return Err(LabeledError {
                    label: "Expected something from pipeline".into(),
                    msg: format!("requires some input, got {}", v.get_type()),
                    span: Some(call.head),
                });
            }
        };

        Ok(ret_val)
    }
}
