use std::env;
use std::env::VarError;
use std::time::SystemTime;

use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand, SimplePluginCommand};
use nu_protocol::{
    Category, ErrorLabel, Example, LabeledError, PluginExample, PluginSignature, Signature, Span,
    Spanned, SyntaxShape, Type, Value
};

use crate::highlight::Highlighter;

const THEME_ENV: &str = "NU_PLUGIN_HIGHLIGHT_THEME";
const TRUE_COLORS_ENV: &str = "NU_PLUGIN_HIGHLIGHT_TRUE_COLORS";

/// The struct that handles the plugin itself.
pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Highlight)]
    }
}

struct Highlight;

impl SimplePluginCommand for Highlight {
    type Plugin = HighlightPlugin;

    fn name(&self) -> &str {
        "highlight"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .optional(
                "language",
                SyntaxShape::String,
                "language or file extension to help language detection"
            )
            .named(
                "theme",
                SyntaxShape::String,
                "them used for highlighting",
                Some('t')
            )
            .switch("list-themes", "list all possible themes", None)
            .category(Category::Strings)
            .input_output_type(Type::String, Type::String)
            .input_output_type(
                Type::Any,
                Type::Table(vec![
                    (String::from("id"), Type::String),
                    (String::from("name"), Type::String),
                    (String::from("author"), Type::String),
                    (String::from("default"), Type::Bool),
                ])
            )
    }

    fn usage(&self) -> &str {
        "Syntax highlight source code."
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value
    ) -> Result<Value, LabeledError> {
        let highlighter = Highlighter::new();

        if call.has_flag("list-themes")? {
            return Ok(highlighter.list_themes().into());
        }

        let config = engine.get_plugin_config()?;

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
                    msg: format!("expected string, got {}", v.get_type()),
                    labels: vec![ErrorLabel {
                        text: "Expected source code as string from pipeline".to_owned(),
                        span: call.head
                    }],
                    code: None,
                    url: None,
                    help: None,
                    inner: vec![]
                });
            }
        };

        Ok(ret_val)
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["syntax", "highlight", "highlighting"]
    }

    fn examples(&self) -> Vec<Example> {
        const fn example<'e>(description: &'e str, example: &'e str) -> Example<'e>
        where
            'e: 'static
        {
            Example {
                example,
                description,
                result: None
            }
        }

        vec![
            example(
                "Highlight a toml file by its file extension",
                "open Cargo.toml -r | highlight toml"
            ),
            example(
                "Highlight a rust file by programming language",
                "open src/main.rs | highlight Rust"
            ),
            example(
                "Highlight a bash script by inferring the language (needs shebang)",
                "open example.sh | highlight"
            ),
            example(
                "Highlight a toml file with another theme",
                "open Cargo.toml -r | highlight toml -t ansi"
            ),
            example("List all available themes", "highlight --list-themes"),
        ]
    }
}

/// Simple constructor for [`LabeledError`].
fn labeled_error(msg: String, label: String, span: Option<Span>) -> LabeledError {
    LabeledError {
        msg,
        labels: vec![ErrorLabel {
            text: label,
            span: span.unwrap_or(Span::unknown())
        }],
        code: None,
        url: None,
        help: None,
        inner: vec![]
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
            return Err(labeled_error(
                "use `highlight --list-themes` to list all themes".into(),
                format!("Unknown passed theme {val:?}"),
                Some(internal_span)
            ))
        }
        Some(v) => {
            return Err(labeled_error(
                format!("expected string, got {}", v.get_type()),
                "Passed theme is not a string".into(),
                Some(v.span())
            ))
        }
        None => ()
    }

    match config_value {
        Some(VS { val, .. }) if is_valid_theme(&val) => return ok(val),
        Some(VS { val, .. }) => {
            return Err(labeled_error(
                "use `highlight --list-themes` to list all themes".into(),
                format!("Unknown config theme {val:?}"),
                None
            ))
        }
        Some(v) => {
            return Err(labeled_error(
                format!("expected string, got {}", v.get_type()),
                "Configured theme is not a string".into(),
                Some(v.span())
            ))
        }
        None => ()
    }

    match env::var(THEME_ENV) {
        Ok(val) if is_valid_theme(&val) => return ok(val),
        Ok(val) => {
            return Err(labeled_error(
                "use `highlight --list-themes` to list all themes".into(),
                format!("Unknown env theme {val:?}"),
                None
            ))
        }
        Err(VarError::NotUnicode(_)) => {
            return Err(labeled_error(
                "make sure only unicode characters are used".into(),
                format!("{THEME_ENV} is not unicode"),
                None
            ))
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
            return Err(labeled_error(
                format!("expected bool, got {}", v.get_type()),
                "True colors configuration is not a boolean".into(),
                None
            ))
        }
        None => ()
    }

    match env::var(TRUE_COLORS_ENV).as_ref().map(|v| v.as_str()) {
        Ok("true" | "yes" | "1" | "") => return ok(true),
        Ok("false" | "no" | "0") => return ok(false),
        Ok(s) => {
            return Err(labeled_error(
                format!(
                    "consider unsetting $env.{TRUE_COLORS_ENV} or set it to {:?} or {:?}",
                    true, false
                ),
                format!("Could not parse {s:?} as boolean"),
                None
            ))
        }
        Err(VarError::NotUnicode(_)) => {
            return Err(labeled_error(
                "make sure only unicode characters are used".into(),
                format!("{TRUE_COLORS_ENV} is not unicode"),
                None
            ))
        }
        Err(VarError::NotPresent) => ()
    }

    Ok(None)
}
