use std::ops::Deref;
use std::path::Path;

use bat::assets::HighlightingAssets;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::LoadingError;

use crate::terminal;
use crate::theme::{ListThemes, ThemeDescription};

const SYNTAX_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/syntax_set.bin"));

/// The struct that handles the highlighting of code.
pub struct Highlighter {
    syntax_set: SyntaxSet,
    highlighting_assets: HighlightingAssets,
    custom_themes: Option<ThemeSet>
}

impl Highlighter {
    /// Creates a new instance of the Highlighter.
    pub fn new() -> Self {
        Highlighter {
            syntax_set: syntect::dumps::from_uncompressed_data(SYNTAX_SET).unwrap(),
            highlighting_assets: HighlightingAssets::from_binary(),
            custom_themes: None
        }
    }

    pub fn custom_themes_from_folder(
        &mut self,
        path: impl AsRef<Path>
    ) -> Result<(), LoadingError> {
        let path = nu_path::expand_to_real_path(path);
        self.custom_themes = Some(ThemeSet::load_from_folder(path)?);
        Ok(())
    }

    /// Lists all the available themes.
    pub fn list_themes(&self, user_default: Option<&str>) -> ListThemes {
        let ha = &self.highlighting_assets;
        let default_theme_id = user_default.unwrap_or(HighlightingAssets::default_theme());

        let mut themes: Vec<_> = ha
            .themes()
            .map(|t_id| {
                let theme = ha.get_theme(t_id);
                ThemeDescription {
                    id: t_id.to_owned(),
                    name: theme.name.clone(),
                    author: theme.author.clone(),
                    default: default_theme_id == t_id
                }
            })
            .collect();

        if let Some(custom_themes) = self.custom_themes.as_ref() {
            for (id, theme) in custom_themes.themes.iter() {
                themes.push(ThemeDescription {
                    id: id.to_owned(),
                    name: theme.name.clone(),
                    author: theme.author.clone(),
                    default: default_theme_id == id
                });
            }
        }

        ListThemes(themes)
    }

    /// Checks if a given theme id is valid.
    pub fn is_valid_theme(&self, theme_name: &str) -> bool {
        let ha = &self.highlighting_assets;
        let custom_themes = self
            .custom_themes
            .as_ref()
            .map(|themes| themes.themes.keys())
            .unwrap_or_default()
            .map(Deref::deref);
        custom_themes.chain(ha.themes()).any(|t| t == theme_name)
    }

    /// Highlights the given input text based on the provided language and
    /// theme.
    pub fn highlight(
        &self,
        input: &str,
        language: Option<&str>,
        theme: Option<&str>,
        true_colors: bool
    ) -> String {
        let syntax_set = &self.syntax_set;
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

        let theme_id = match theme {
            None => HighlightingAssets::default_theme(),
            Some(theme) => theme
        };
        let theme = self
            .custom_themes
            .as_ref()
            .map(|themes| themes.themes.get(theme_id))
            .flatten();
        let theme = theme.unwrap_or_else(|| self.highlighting_assets.get_theme(theme_id));

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

                let styled_lines = highlighter.highlight_line(&l, &syntax_set).unwrap();
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
