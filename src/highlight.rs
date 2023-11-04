use bat::assets::HighlightingAssets;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxReference;

use crate::terminal;
use crate::theme::{ListThemes, ThemeDescription};

/// The struct that handles the highlighting of code.
pub struct Highlighter {
    highlighting_assets: HighlightingAssets
}

impl Highlighter {
    /// Creates a new instance of the Highlighter.
    pub fn new() -> Self {
        Highlighter {
            highlighting_assets: HighlightingAssets::from_binary()
        }
    }

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
