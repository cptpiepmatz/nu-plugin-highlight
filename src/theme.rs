use nu_protocol::{record, Span, Value};

/// Description of a theme.
pub struct ThemeDescription {
    pub id: String,
    pub name: Option<String>,
    pub author: Option<String>,
    pub default: bool
}

/// List of theme descriptions.
pub struct ListThemes(pub Vec<ThemeDescription>);

impl From<ThemeDescription> for Value {
    fn from(value: ThemeDescription) -> Value {
        let ThemeDescription {
            id,
            name,
            author,
            default
        } = value;
        Value::record(
            record! {
                "id" => Value::test_string(id),
                "name" => match name {
                    Some(name) => Value::test_string(name),
                    None => Value::test_nothing(),
                },
                "author" => match author {
                    Some(author) => Value::test_string(author),
                    None => Value::test_nothing(),
                },
                "default" => Value::test_bool(default),
            },
            Span::unknown()
        )
    }
}

impl From<ListThemes> for Value {
    fn from(value: ListThemes) -> Self {
        Value::list(
            value.0.into_iter().map(Value::from).collect(),
            Span::unknown()
        )
    }
}
