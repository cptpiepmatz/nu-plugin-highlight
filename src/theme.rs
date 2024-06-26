use nu_protocol::{IntoValue, Span, Value};

/// Description of a theme.
#[derive(Debug, IntoValue)]
pub struct ThemeDescription {
    pub id: String,
    pub name: Option<String>,
    pub author: Option<String>,
    pub default: bool
}

/// List of theme descriptions.
#[derive(Debug)]
pub struct ListThemes(pub Vec<ThemeDescription>);

impl IntoValue for ListThemes {
    fn into_value(self, span: Span) -> Value {
        self.0.into_value(span)
    }
}
