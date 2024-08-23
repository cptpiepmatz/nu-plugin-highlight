use std::env;
use std::path::PathBuf;

use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand, SimplePluginCommand};
use nu_protocol::{
    Category, ErrorLabel, Example, FromValue, IntoValue, LabeledError, Signature, Span, Spanned,
    SyntaxShape, Type, Value
};

use crate::highlight::Highlighter;

/// The struct that handles the plugin itself.
pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Highlight)]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }
}

#[derive(Debug, FromValue, Default)]
struct Config {
    pub theme: Option<Spanned<String>>,
    pub true_colors: Option<bool>,
    pub custom_themes: Option<Spanned<PathBuf>>
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
                Type::Table(
                    vec![
                        (String::from("id"), Type::String),
                        (String::from("name"), Type::String),
                        (String::from("author"), Type::String),
                        (String::from("default"), Type::Bool),
                    ]
                    .into()
                )
            )
    }

    fn description(&self) -> &str {
        "Syntax highlight source code."
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value
    ) -> Result<Value, LabeledError> {
        let mut highlighter = Highlighter::new();

        let config = Option::<Config>::from_value(engine.get_plugin_config()?.unwrap_or_default())?
            .unwrap_or_default();

        if let Some(custom_themes_path) = config.custom_themes {
            if let Err(err) = highlighter.custom_themes_from_folder(&custom_themes_path.item) {
                return Err(labeled_error(
                    err,
                    "error while loading custom themes",
                    custom_themes_path.span
                ));
            }
        }

        let theme = call
            .get_flag_value("theme")
            .map(Spanned::<String>::from_value)
            .transpose()?
            .or(config.theme);
        if let Some(theme) = &theme {
            if !highlighter.is_valid_theme(&theme.item) {
                return Err(labeled_error(
                    "use `highlight --list-themes` to list all themes",
                    format!("Unknown passed theme {:?}", &theme.item),
                    theme.span
                ));
            }
        }
        let theme = theme.map(|spanned| spanned.item);
        let theme = theme.as_deref();

        let true_colors = config.true_colors.unwrap_or(true);

        if call.has_flag("list-themes")? {
            return Ok(highlighter.list_themes(theme).into_value(call.head));
        }

        let language = call.opt(0)?.map(String::from_value).transpose()?;

        // try to highlight if input is a string
        let ret_val = match input {
            Value::String { val, .. } => Value::string(
                highlighter.highlight(val, language.as_deref(), theme, true_colors),
                call.head
            ),
            v => {
                return Err(labeled_error(
                    format!("expected string, got {}", v.get_type()),
                    "Expected source code as string from pipeline",
                    call.head
                ));
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
fn labeled_error(
    msg: impl ToString,
    label: impl ToString,
    span: impl Into<Option<Span>>
) -> LabeledError {
    LabeledError {
        msg: msg.to_string(),
        labels: vec![ErrorLabel {
            text: label.to_string(),
            span: span.into().unwrap_or(Span::unknown())
        }],
        code: None,
        url: None,
        help: None,
        inner: vec![]
    }
}
