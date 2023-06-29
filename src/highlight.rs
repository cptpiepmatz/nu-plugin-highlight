use bat::assets::HighlightingAssets;
use nu_plugin::LabeledError;
use nu_protocol::{Span, Spanned, Value};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxReference;

use crate::terminal;
use crate::theme::{ListThemes, ThemeDescription};

pub struct Highlighter {
    highlighting_assets: HighlightingAssets
}

impl Highlighter {
    pub fn new() -> Self {
        Highlighter {
            highlighting_assets: HighlightingAssets::from_binary()
        }
    }

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

    pub fn is_valid_theme(&self, theme_name: &str) -> bool {
        let ha = &self.highlighting_assets;
        ha.themes().any(|t| t == theme_name)
    }

    pub fn highlight(
        &self,
        input: &str,
        language: &Option<String>,
        theme: &Option<String>
    ) -> String {
        let syntax_set = self.highlighting_assets.get_syntax_set().unwrap();
        let syntax_ref: Option<&SyntaxReference> = match language {
            None => None,
            Some(language) => syntax_set
                .find_syntax_by_name(language)
                .or(syntax_set.find_syntax_by_extension(language))
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
        input
            .lines()
            .map(|l| {
                let styled_lines = highlighter.highlight_line(l, syntax_set).unwrap();
                styled_lines
                    .iter()
                    .map(|(style, s)| {
                        terminal::as_terminal_escaped(style.clone(), s, true, true, false, None)
                    })
                    .collect::<String>() + "\n"
            })
            .collect::<String>().trim().to_owned()
    }
}
