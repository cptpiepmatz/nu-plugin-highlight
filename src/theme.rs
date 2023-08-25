use nu_protocol::{record, Span, Value};

/// Description of a theme.
pub struct ThemeDescription {
    pub id: String,
    pub name: Option<String>,
    pub author: Option<String>,
    pub default: bool,
}

/// List of theme descriptions.
pub struct ListThemes(pub Vec<ThemeDescription>);

impl From<ThemeDescription> for Value {
    fn from(value: ThemeDescription) -> Value {
        let ThemeDescription {
            id,
            name,
            author,
            default,
        } = value;
        Value::record(
            record! {
                "id" => Value::string(id, Span::unknown()),
                "name" => match name {
                    Some(name) => Value::string(name, Span::unknown()),
                    None => Value::nothing(Span::unknown()),

                },
                "author" => match author {
                    Some(author) => Value::string(author, Span::unknown()),
                    None => Value::nothing(Span::unknown()),
                },
                "default" => Value::bool(default, Span::unknown()),
            },
            Span::unknown(),
        )
    }
}

impl From<ListThemes> for Value {
    fn from(value: ListThemes) -> Self {
        Value::list(
            value.0.into_iter().map(Value::from).collect(),
            Span::unknown(),
        )
    }
}
