use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

use bat::assets::HighlightingAssets;
use nu_protocol::LabeledError;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxReference;

use crate::terminal;
use crate::theme::{ListThemes, ThemeDescription};

// We try to use the system bat version by calling bat -V later, but if this
// doesn't exist, this is just a a default value (highlight doesn't error if
// there is a particular version used in metadata.yaml)
const DEFAULT_BAT_VERSION: &str = "0.24.0";

/// The struct that handles the highlighting of code.
pub struct Highlighter {
    highlighting_assets: HighlightingAssets
}

impl Highlighter {
    /// Creates a new instance of the Highlighter.
    pub fn new(cache_path: String) -> Self {
        let cache_path = PathBuf::from(cache_path);
        let highlighting_assets =
            if let Ok(highlighting_assets) = HighlightingAssets::from_cache(&cache_path) {
                highlighting_assets
            }
            else {
                HighlightingAssets::from_binary()
            };

        Highlighter {
            highlighting_assets
        }
    }

    pub fn build_cache(src_path: String, cache_path: String) -> Result<&'static str, LabeledError> {
        let src = PathBuf::from(src_path);
        let cache = PathBuf::from(cache_path);

        // if bat exists in the system use it (to avoid conflicts for config version)
        let current_version = if let Ok(s) = Command::new("bat").arg("-V").output() {
            let data = s.stdout;
            if let Ok(s) = from_utf8(&data) {
                s.to_owned()
                    .chars()
                    .filter(|&c| (c == '.' || c.is_ascii_digit()))
                    .collect::<String>()
            }
            else {
                DEFAULT_BAT_VERSION.to_owned()
            }
        }
        else {
            DEFAULT_BAT_VERSION.to_owned()
        };

        println!();
        bat::assets::build(&src, true, true, &cache, &current_version)
            .map(|_| "\nCache created succesfully!")
            .map_err(|e| LabeledError::new(format!("bat cache build failed. {e:?}")))
    }

    // fn src_cache_from_bat() -> Result<(String, String), LabeledError> {
    //     let src_path = from_utf8(
    //         &Command::new("bat")
    //             .arg("--config-dir")
    //             .output()
    //             .map_err(|e| LabeledError::new(format!("Failed to run bat:
    // {e}")))?             .stdout
    //     )
    //     .map_err(|e| LabeledError::new(format!("Parsing bat --config-dir failed:
    // {e:?}")))?     .trim_end()
    //     .to_owned();

    //     let cache_path = from_utf8(
    //         &Command::new("bat")
    //             .arg("--cache-dir")
    //             .output()
    //             .map_err(|e| LabeledError::new(format!("Failed to run bat:
    // {e}")))?             .stdout
    //     )
    //     .map_err(|e| LabeledError::new(format!("Parsing bat --cache-dir failed:
    // {e:?}")))?     .trim_end()
    //     .to_owned();

    //     Ok((src_path, cache_path))
    // }

    // fn src_cache_from_options(
    //     src_path: Option<Value>,
    //     cache_path: Option<Value>
    // ) -> Result<(String, String), LabeledError> {
    //     let src_path = if let Some(src_path) = src_path {
    //         match src_path {
    //             Value::String {
    //                 val: src_path,
    //                 internal_span: _
    //             } => Ok(src_path),
    //             _ => return Err(LabeledError::new("src_path field is not a
    // string"))         }
    //     }
    //     else {
    //         Err(LabeledError::new("src_path field is not defined"))
    //     }?;
    //     let cache_path = if let Some(cache_path) = cache_path {
    //         match cache_path {
    //             Value::String {
    //                 val: cache_path,
    //                 internal_span: _
    //             } => Ok(cache_path),
    //             _ => return Err(LabeledError::new("cache_path field is not a
    // string"))         }
    //     }
    //     else {
    //         Err(LabeledError::new("cache_path is not defined"))
    //     }?;

    //     Ok((src_path, cache_path))
    // }

    // create the cache (BAT_VERSION must match to avoid conflict, so try to pick it
    // up from bat -V command)
    // fn create_cache(src: &str, target_dir: &str) -> Result<(), LabeledError> {

    // }

    /// Lists all the available themes.
    pub fn list_themes(&self) -> ListThemes {
        let ha = &self.highlighting_assets;
        let default_theme_id = HighlightingAssets::default_theme();
        ListThemes(
            ha.themes()
                .map(|t_id| {
                    let theme = ha.get_theme(t_id);
                    ThemeDescription {
                        id: t_id.to_owned(),
                        name: theme.name.clone(),
                        author: theme.author.clone(),
                        default: default_theme_id == t_id
                    }
                })
                .collect()
        )
    }

    /// Checks if a given theme id is valid.
    pub fn is_valid_theme(&self, theme_name: &str) -> bool {
        let ha = &self.highlighting_assets;
        ha.themes().any(|t| t == theme_name)
    }

    /// Highlights the given input text based on the provided language and
    /// theme.
    pub fn highlight(
        &self,
        input: &str,
        language: &Option<String>,
        theme: &Option<String>,
        true_colors: bool
    ) -> String {
        let syntax_set = self.highlighting_assets.get_syntax_set().unwrap();
        let syntax_ref: Option<&SyntaxReference> = match language {
            Some(language) if !language.is_empty() => {
                // allow multiple variants to write the language
                let language_lowercase = language.to_lowercase();
                let language_capitalized = {
                    let mut chars = language.chars();
                    let mut out = String::with_capacity(language.len());
                    chars
                        .next()
                        .expect("language not empty")
                        .to_uppercase()
                        .for_each(|c| out.push(c));
                    chars.for_each(|c| out.push(c));
                    out
                };

                syntax_set
                    .find_syntax_by_name(language)
                    .or_else(|| syntax_set.find_syntax_by_name(&language_lowercase))
                    .or_else(|| syntax_set.find_syntax_by_name(&language_capitalized))
                    .or_else(|| syntax_set.find_syntax_by_extension(language))
                    .or_else(|| syntax_set.find_syntax_by_extension(&language_lowercase))
                    .or_else(|| syntax_set.find_syntax_by_extension(&language_capitalized))
            }
            _ => None
        };
        let syntax_ref = syntax_ref
            .or(syntax_set.find_syntax_by_first_line(input))
            .unwrap_or(syntax_set.find_syntax_plain_text());

        let theme = match theme {
            None => HighlightingAssets::default_theme(),
            Some(theme) => theme
        };
        let theme = self.highlighting_assets.get_theme(theme);

        let mut highlighter = HighlightLines::new(syntax_ref, theme);
        let line_count = input.lines().count();
        input
            .lines()
            .enumerate()
            .map(|(i, l)| {
                // insert a newline in between lines, this is necessary for bats syntax set
                let l = match i == line_count - 1 {
                    false => format!("{}\n", l.trim_end()),
                    true => l.trim_end().to_owned()
                };

                let styled_lines = highlighter.highlight_line(&l, syntax_set).unwrap();
                styled_lines
                    .iter()
                    .map(|(style, s)| {
                        terminal::as_terminal_escaped(*style, s, true_colors, true, false, None)
                    })
                    .collect::<String>()
            })
            .collect::<String>()
    }
}
