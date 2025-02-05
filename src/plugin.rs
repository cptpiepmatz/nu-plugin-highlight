use std::path::PathBuf;
use std::str::FromStr;

use mime_guess::Mime;
use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::shell_error::io::IoError;
use nu_protocol::{
    Category, DataSource, ErrorLabel, Example, FromValue, IntoValue, LabeledError, PipelineData,
    PipelineMetadata, ShellError, Signature, Span, Spanned, SyntaxShape, Type, Value
};
use syntect::LoadingError;

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

impl PluginCommand for Highlight {
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
        input: PipelineData
    ) -> Result<PipelineData, LabeledError> {
        let mut highlighter = Highlighter::new();

        let config = Option::<Config>::from_value(engine.get_plugin_config()?.unwrap_or_default())?
            .unwrap_or_default();

        if let Some(custom_themes_path) = config.custom_themes {
            match highlighter.custom_themes_from_folder(&custom_themes_path.item) {
                Ok(_) => (),
                Err(LoadingError::Io(err)) => {
                    return Err(LabeledError::from(ShellError::from(
                        IoError::new_with_additional_context(
                            err.kind(),
                            custom_themes_path.span,
                            custom_themes_path.item,
                            "Error while loading custom themes"
                        )
                    )))
                }
                Err(err) => {
                    return Err(labeled_error(
                        err,
                        "Error while loading custom themes",
                        custom_themes_path.span,
                        None
                    ))
                }
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
                    theme.span,
                    None
                ));
            }
        }
        let theme = theme.map(|spanned| spanned.item);
        let theme = theme.as_deref();

        let true_colors = config.true_colors.unwrap_or(true);

        if call.has_flag("list-themes")? {
            let themes = highlighter.list_themes(theme).into_value(call.head);
            return Ok(PipelineData::Value(themes, None));
        }

        let metadata = input.metadata();
        let input = input.into_value(call.head)?;
        let Spanned { item: input, span } = Spanned::<String>::from_value(input)?;

        let language = language_hint(call, metadata.as_ref())?;
        let highlighted = highlighter.highlight(&input, language.as_deref(), theme, true_colors);
        let highlighted = Value::string(highlighted, span);
        Ok(PipelineData::Value(highlighted, metadata))
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
                "Highlight a Markdown file by guessing the type from the pipeline metadata",
                "open README.md | highlight"
            ),
            example(
                "Highlight a toml file by its file extension",
                "open Cargo.toml -r | echo $in | highlight toml"
            ),
            example(
                "Highlight a rust file by programming language",
                "open src/main.rs | echo $in | highlight Rust"
            ),
            example(
                "Highlight a bash script by inferring the language (needs shebang)",
                "open example.sh | echo $in | highlight"
            ),
            example(
                "Highlight a toml file with another theme",
                "open Cargo.toml -r | highlight -t ansi"
            ),
            example("List all available themes", "highlight --list-themes"),
        ]
    }
}

fn language_hint(
    call: &EvaluatedCall,
    metadata: Option<&PipelineMetadata>
) -> Result<Option<String>, ShellError> {
    // first use passed argument
    let arg = call.opt(0)?.map(String::from_value).transpose()?;

    // then try to parse a mime type
    let content_type = || -> Option<String> {
        let metadata = metadata?;
        let content_type = metadata.content_type.as_ref();
        let content_type = content_type?.as_str();
        let content_type = Mime::from_str(content_type).ok()?;
        let sub_type = content_type.subtype().to_string();
        match sub_type.as_str() {
            "tab-separated-values" => Some("tsv".to_string()),
            "x-toml" => Some("toml".to_string()),
            s if s.starts_with("x-") => None, // we cannot be sure about this type,
            _ => Some(sub_type)
        }
    };

    // as last resort, try to use the extension of data source
    let data_source = || -> Option<String> {
        let data_source = &metadata?.data_source;
        let DataSource::FilePath(path) = data_source
        else {
            return None;
        };
        let extension = path.extension()?.to_string_lossy();
        Some(extension.to_string())
    };

    Ok(arg.or_else(content_type).or_else(data_source))
}

/// Simple constructor for [`LabeledError`].
fn labeled_error(
    msg: impl ToString,
    label: impl ToString,
    span: Span,
    inner: impl Into<Option<ShellError>>
) -> LabeledError {
    LabeledError {
        msg: msg.to_string(),
        labels: Box::new(vec![ErrorLabel {
            text: label.to_string(),
            span
        }]),
        code: None,
        url: None,
        help: None,
        inner: match inner.into() {
            Some(inner) => Box::new(vec![inner.into()]),
            None => Box::new(vec![])
        }
    }
}
