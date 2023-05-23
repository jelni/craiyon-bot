use std::borrow::Cow;

use tdlib::enums::TextEntityType;
use tdlib::types::{FormattedText, TextEntity, TextEntityTypeTextUrl};

pub trait Utf16Len {
    fn utf16_len(&self) -> usize;
}

impl Utf16Len for str {
    fn utf16_len(&self) -> usize {
        self.bytes()
            .filter(|byte| (byte & 0xC0) != 0x80)
            .map(|byte| if byte >= 0xF0 { 2 } else { 1 })
            .sum()
    }
}

pub enum Entity<'a> {
    Text(Cow<'a, str>),
    Bold(Vec<Entity<'a>>),
    Italic(Vec<Entity<'a>>),
    Code(Vec<Entity<'a>>),
    TextUrl { text: Vec<Entity<'a>>, url: Cow<'a, str> },
}

pub trait ToEntity<'a> {
    fn text(&self) -> Entity;
    fn bold(&self) -> Entity;
    fn italic(&self) -> Entity;
    fn code(&self) -> Entity;
    fn text_url(&'a self, url: impl Into<Cow<'a, str>>) -> Entity<'a>;
}

impl<'a> ToEntity<'a> for str {
    fn text(&self) -> Entity {
        Entity::Text(self.into())
    }

    fn bold(&self) -> Entity {
        Entity::Text(self.into()).bold()
    }

    fn italic(&self) -> Entity {
        Entity::Text(self.into()).italic()
    }

    fn code(&self) -> Entity {
        Entity::Text(self.into()).code()
    }

    fn text_url(&'a self, url: impl Into<Cow<'a, str>>) -> Entity<'a> {
        Entity::Text(self.into()).text_url(url)
    }
}

pub trait ToEntityOwned<'a> {
    fn text_owned(self) -> Entity<'a>;
    fn bold_owned(self) -> Entity<'a>;
    fn italic_owned(self) -> Entity<'a>;
    fn code_owned(self) -> Entity<'a>;
    fn text_url_owned(self, url: impl Into<Cow<'a, str>>) -> Entity<'a>;
}

impl<'a> ToEntityOwned<'a> for String {
    fn text_owned(self) -> Entity<'a> {
        Entity::Text(self.into())
    }

    fn bold_owned(self) -> Entity<'a> {
        Entity::Text(self.into()).bold()
    }

    fn italic_owned(self) -> Entity<'a> {
        Entity::Text(self.into()).italic()
    }

    fn code_owned(self) -> Entity<'a> {
        Entity::Text(self.into()).code()
    }

    fn text_url_owned(self, url: impl Into<Cow<'a, str>>) -> Entity<'a> {
        Entity::Text(self.into()).text_url(url)
    }
}

pub trait ToNestedEntity<'a> {
    fn bold(self) -> Entity<'a>;
    fn italic(self) -> Entity<'a>;
    fn code(self) -> Entity<'a>;
    fn text_url(self, url: impl Into<Cow<'a, str>>) -> Entity<'a>;
}

impl<'a> ToNestedEntity<'a> for Entity<'a> {
    fn bold(self) -> Self {
        Self::Bold(vec![self])
    }

    fn italic(self) -> Self {
        Self::Italic(vec![self])
    }

    fn code(self) -> Self {
        Self::Code(vec![self])
    }

    fn text_url(self, url: impl Into<Cow<'a, str>>) -> Self {
        Self::TextUrl { text: vec![self], url: url.into() }
    }
}

fn format_entities(
    mut text: String,
    input_entities: Vec<Entity>,
    mut offset: usize,
) -> (String, Vec<TextEntity>, usize) {
    let mut entities = Vec::new();

    for entity in input_entities {
        let (new_text, new_entities, new_offset, r#type) = match entity {
            Entity::Text(inner) => {
                text.push_str(&inner);
                (text, Vec::new(), offset + inner.utf16_len(), None)
            }
            Entity::Bold(entities) => {
                let ret = format_entities(text, entities, offset);
                (ret.0, ret.1, ret.2, Some(TextEntityType::Bold))
            }
            Entity::Italic(entities) => {
                let ret = format_entities(text, entities, offset);
                (ret.0, ret.1, ret.2, Some(TextEntityType::Italic))
            }
            Entity::Code(entities) => {
                let ret = format_entities(text, entities, offset);
                (ret.0, ret.1, ret.2, Some(TextEntityType::Code))
            }
            Entity::TextUrl { text: entities, url } => {
                let ret = format_entities(text, entities, offset);
                (
                    ret.0,
                    ret.1,
                    ret.2,
                    Some(TextEntityType::TextUrl(TextEntityTypeTextUrl { url: url.into_owned() })),
                )
            }
        };

        text = new_text;
        if let Some(r#type) = r#type {
            entities.push(TextEntity {
                offset: offset.try_into().unwrap(),
                length: (new_offset - offset).try_into().unwrap(),
                r#type,
            });
        }
        entities.extend(new_entities);
        offset = new_offset;
    }

    (text, entities, offset)
}

pub fn formatted_text(entities: Vec<Entity>) -> FormattedText {
    let (text, entities, _) = format_entities(String::new(), entities, 0);

    FormattedText { text, entities }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_formatted_text() {
        let formatted_text = formatted_text(vec![
            "🦀 ".text(),
            "one ".text(),
            "two".bold(),
            " ".text(),
            "three".code(),
            " ".text(),
            "four".text_url("example.com"),
        ]);

        assert_eq!(
            formatted_text,
            FormattedText {
                text: "🦀 one two three four".into(),
                entities: vec![
                    TextEntity { offset: 7, length: 3, r#type: TextEntityType::Bold },
                    TextEntity { offset: 11, length: 5, r#type: TextEntityType::Code },
                    TextEntity {
                        offset: 17,
                        length: 4,
                        r#type: TextEntityType::TextUrl(TextEntityTypeTextUrl {
                            url: "example.com".into()
                        })
                    },
                ]
            }
        );
    }

    #[test]
    fn test_nested_formatted_text() {
        let formatted_text =
            formatted_text(vec!["🦀".bold(), " ".text(), "foo".italic().text_url("bar")]);

        assert_eq!(
            formatted_text,
            FormattedText {
                text: "🦀 foo".into(),
                entities: vec![
                    TextEntity { offset: 0, length: 2, r#type: TextEntityType::Bold },
                    TextEntity {
                        offset: 3,
                        length: 3,
                        r#type: TextEntityType::TextUrl(TextEntityTypeTextUrl {
                            url: "bar".into()
                        })
                    },
                    TextEntity { offset: 3, length: 3, r#type: TextEntityType::Italic },
                ]
            }
        )
    }
}
