use std::env;
use std::env::VarError;

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
        config: &Option<Value>,
        call: &EvaluatedCall,
        input: &Value
    ) -> Result<Value, LabeledError> {
        assert_eq!(name, "highlight");
        let highlighter = Highlighter::new();

        // ignore everything else and return the list of themes
        if call.has_flag("list-themes")? {
            return Ok(highlighter.list_themes().into());
        }

        let theme = extract_theme(
            |t| highlighter.is_valid_theme(t),
            call.get_flag_value("theme"),
            config
                .as_ref()
                .and_then(|v| v.get_data_by_key("theme"))
                .clone()
        )?;

        let true_colors = extract_true_colors(
            config
                .as_ref()
                .and_then(|v| v.get_data_by_key("true_colors"))
                .clone()
        )?
        .unwrap_or(true);

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

/// Extract theme.
///
/// Try to pull the theme out of these in that order:
/// - passed flag value
/// - passed config value
/// - env variable [`THEME_ENV`]
fn extract_theme(
    is_valid_theme: impl Fn(&str) -> bool,
    flag_value: Option<Value>,
    config_value: Option<Value>
) -> Result<Option<String>, LabeledError> {
    use Value::String as VS;
    let ok = |v| Ok(Some(v));

    match flag_value {
        Some(VS { val, .. }) if is_valid_theme(&val) => return ok(val),
        Some(VS { val, internal_span }) => {
            return Err(LabeledError {
                label: format!("Unknown passed theme {val:?}"),
                msg: "use `highlight --list-themes` to list all themes".into(),
                span: Some(internal_span)
            })
        }
        Some(v) => {
            return Err(LabeledError {
                label: "Passed theme is not a string".into(),
                msg: format!("expected string, got {}", v.get_type()),
                span: Some(v.span())
            })
        }
        None => ()
    }

    match config_value {
        Some(VS { val, .. }) if is_valid_theme(&val) => return ok(val),
        Some(VS { val, .. }) => {
            return Err(LabeledError {
                label: format!("Unknown config theme {val:?}"),
                msg: "use `highlight --list-themes` to list all themes".into(),
                span: None
            })
        }
        Some(v) => {
            return Err(LabeledError {
                label: "Configured theme is not a string".into(),
                msg: format!("expected string, got {}", v.get_type()),
                span: Some(v.span())
            })
        }
        None => ()
    }

    match env::var(THEME_ENV) {
        Ok(val) if is_valid_theme(&val) => return ok(val),
        Ok(val) => {
            return Err(LabeledError {
                label: format!("Unknown env theme {val:?}"),
                msg: "use `highlight --list-themes` to list all themes".into(),
                span: None
            })
        }
        Err(VarError::NotUnicode(_)) => {
            return Err(LabeledError {
                label: format!("{THEME_ENV} is not unicode"),
                msg: "make sure only unicode characters are used".into(),
                span: None
            })
        }
        Err(VarError::NotPresent) => ()
    }

    Ok(None)
}

/// Extract true colors setting.
///
/// Try to extract true colors setting either from config or from env variable
/// [`TRUE_COLORS_ENV`]
fn extract_true_colors(config_value: Option<Value>) -> Result<Option<bool>, LabeledError> {
    use Value::Bool as VB;
    let ok = |v| Ok(Some(v));

    match config_value {
        Some(VB { val, .. }) => return ok(val),
        Some(v) => {
            return Err(LabeledError {
                label: "True colors configuration is not a boolean".into(),
                msg: format!("expected bool, got {}", v.get_type()),
                span: None
            })
        }
        None => ()
    }

    match env::var(TRUE_COLORS_ENV).as_ref().map(|v| v.as_str()) {
        Ok("true" | "yes" | "1" | "") => return ok(true),
        Ok("false" | "no" | "0") => return ok(false),
        Ok(s) => {
            return Err(LabeledError {
                label: format!("Could not parse {s:?} as boolean"),
                msg: format!(
                    "consider unsetting $env.{TRUE_COLORS_ENV} or set it to {:?} or {:?}",
                    true, false
                ),
                span: None
            })
        }
        Err(VarError::NotUnicode(_)) => {
            return Err(LabeledError {
                label: format!("{TRUE_COLORS_ENV} is not unicode"),
                msg: "make sure only unicode characters are used".into(),
                span: None
            })
        }
        Err(VarError::NotPresent) => ()
    }

    Ok(None)
}
