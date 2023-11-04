use std::env;

use nu_plugin::{EvaluatedCall, LabeledError, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Spanned, SyntaxShape, Type, Value};

use crate::highlight::Highlighter;

const THEME_ENV: &str = "NU_PLUGIN_HIGHLIGHT_THEME";
const TRUE_COLORS_ENV: &str = "NU_PLUGIN_HIGHLIGHT_TRUE_COLORS";

/// The struct that handles the plugin itself.
pub struct HighlightPlugin;

impl HighlightPlugin {
    /// Creates a new instance of the HighlightPlugin.
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for HighlightPlugin {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("highlight")
            .usage("Syntax highlight source code.")
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
            .category(Category::Strings)
            .search_terms(vec![
                "syntax".into(),
                "highlight".into(),
                "highlighting".into(),
            ])
            .input_output_types(vec![
                (Type::String, Type::String),
                (
                    Type::Any,
                    Type::Table(vec![
                        (String::from("id"), Type::String),
                        (String::from("name"), Type::String),
                        (String::from("author"), Type::String),
                        (String::from("default"), Type::Bool),
                    ])
                ),
            ])
            .plugin_examples(
                (vec![
                    (
                        "Highlight a toml file by its file extension",
                        "open Cargo.toml -r | highlight toml"
                    ),
                    (
                        "Highlight a rust file by programming language",
                        "open src/main.rs | highlight Rust"
                    ),
                    (
                        "Highlight a bash script by inferring the language (needs shebang)",
                        "open example.sh | highlight"
                    ),
                    (
                        "Highlight a toml file with another theme",
                        "open Cargo.toml -r | highlight toml -t ansi"
                    ),
                    ("List all available themes", "highlight --list-themes"),
                ])
                .into_iter()
                .map(|(description, example)| PluginExample {
                    example: example.to_owned(),
                    description: description.to_owned(),
                    result: None
                })
                .collect()
            )]
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
        let theme = call.get_flag_value("theme");
        let theme_span = theme.clone().map(|v| v.span()).unwrap_or(call.head);

        // use theme from environment variable if available, override with passed
        let theme = match (theme, env::var(THEME_ENV).ok()) {
            (Some(Value::String { val, .. }), _) if highlighter.is_valid_theme(&val) => Some(val),
            (Some(Value::String { .. }), _) => {
                return Err(LabeledError {
                    label: "Unknown theme, use `highlight --list-themes` to list all themes".into(),
                    msg: "unknown theme".into(),
                    span: Some(theme_span)
                })
            }
            (Some(v), _) => {
                return Err(LabeledError {
                    label: "Expected theme value to be a string".into(),
                    msg: format!("expected string, got {}", v.get_type()),
                    span: Some(theme_span)
                })
            }
            (_, Some(t)) if highlighter.is_valid_theme(&t) => Some(t),
            (_, Some(t)) => {
                return Err(LabeledError {
                    label: format!("Unknown theme \"{}\"", t),
                    msg: "use `highlight --list-themes` to list all themes".into(),
                    span: None
                })
            }
            _ => None
        };

        // check whether to use true colors from env variable, default to true
        let true_colors = env::var(TRUE_COLORS_ENV)
            .ok()
            .map(|s| match s.trim().to_lowercase().as_str() {
                "true" | "yes" | "1" | "" => Ok(true),
                "false" | "no" | "0" => Ok(false),
                s => Err(LabeledError {
                    label: format!("Could not parse \"{}\" as boolean", s),
                    msg: format!(
                        "consider unsetting $env.{} or set it to \"true\" or \"false\"",
                        TRUE_COLORS_ENV
                    ),
                    span: None
                })
            })
            .unwrap_or(Ok(true))?;

        // extract language parameter, doesn't need any validation
        let param: Option<Spanned<String>> = call.opt(0)?;
        let language = param.map(|Spanned { item, .. }| item);

        // try to highlight if input is a string
        let ret_val = match input {
            Value::String { val, .. } => Value::string(
                highlighter.highlight(val, &language, &theme, true_colors),
                call.head
            ),
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
