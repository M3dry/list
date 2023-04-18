use crate::tokenizer::{Keywords, Token};

use super::{
    attribute::Attribute,
    error,
    r#type::{Generic, Type},
    Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub enum Struct {
    Touple {
        name: String,
        generics: Vec<Generic>,
        types: Vec<Type>,
    },
    Normal {
        name: String,
        generics: Vec<Generic>,
        fields: StructFields,
    },
}

impl TryFrom<&mut Parser> for Struct {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Struct")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Struct",
                format!("Expected ParenOpen, got {next:#?}")
            ));
        }

        let next = value.pop_front_err("Struct")?;
        if next != Token::Keyword(Keywords::Struct) {
            return Err(error!(
                "Struct",
                format!("Expected Struct keyword, got {next:#?}")
            ));
        }

        let next = value.pop_front_err("Struct")?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(error!(
                "Struct",
                format!("Expected identifier, got {next:#?}")
            ));
        };

        let mut generics = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Struct", format!("Expected more tokens")))?;

            if let Token::Char(':') = peek {
                generics.push(error!(Generic::try_from(&mut *value), "Struct")?)
            } else {
                break;
            }
        }

        if value.first() == Some(&Token::CurlyOpen) {
            let fields = error!(StructFields::try_from(&mut *value), "Struct")?;

            let next = value.pop_front_err("Struct")?;
            if next != Token::ParenClose {
                return Err(error!(
                    "Struct",
                    format!("Expected ParenClose, got {next:#?}")
                ));
            }

            Ok(Struct::Normal {
                name,
                generics,
                fields,
            })
        } else {
            let mut types = vec![];

            loop {
                let peek = value
                    .first()
                    .ok_or(error!("Struct", format!("Expected more tokens")))?;

                if peek == &Token::ParenClose {
                    value.pop_front();
                    break Ok(Self::Touple {
                        name,
                        generics,
                        types,
                    });
                }

                types.push(error!(Type::try_from(&mut *value), "Struct")?)
            }
        }
    }
}

impl ToString for Struct {
    fn to_string(&self) -> String {
        match self {
            Self::Touple {
                name,
                generics,
                types,
            } => {
                format!(
                    "struct {name}{}({});",
                    if !generics.is_empty() {
                        format!(
                            "<{}>",
                            &generics.iter().fold(String::new(), |str, generic| {
                                format!("{str}, {}", generic.to_string())
                            })[2..]
                        )
                    } else {
                        format!("")
                    },
                    &if types.is_empty() {
                        format!(", ")
                    } else {
                        format!(
                            "{}",
                            &types.iter().fold(String::new(), |str, r#type| format!(
                                "{str}, {}",
                                r#type.to_string()
                            ))
                        )
                    }[2..]
                )
            }
            Self::Normal {
                name,
                generics,
                fields,
            } => {
                format!(
                    "struct {}{} {}",
                    name,
                    if !generics.is_empty() {
                        format!(
                            "<{}>",
                            &generics.iter().fold(String::new(), |str, generic| {
                                format!("{str}, {}", generic.to_string())
                            })[2..]
                        )
                    } else {
                        format!("")
                    },
                    fields.to_string()
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct StructFields(Vec<StructField>);

impl TryFrom<&mut Parser> for StructFields {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("StructFields")?;
        if next != Token::CurlyOpen {
            return Err(error!(
                "StructFields",
                format!("Expected CurlyOpen, got {next:#?}")
            ));
        }

        let mut fields = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("StructFields", format!("Expected more tokens")))?;
            if peek == &Token::CurlyClose {
                value.pop_front();
                break;
            }

            fields.push(error!(StructField::try_from(&mut *value), "StructFields")?);
        }

        Ok(StructFields(fields))
    }
}

impl ToString for StructFields {
    fn to_string(&self) -> String {
        format!(
            "{{{}}}",
            if !self.0.is_empty() {
                (&self.0.iter().fold(String::new(), |str, field| {
                    format!("{str}, {}", field.to_string())
                })[2..])
                    .to_string()
            } else {
                format!("")
            }
        )
    }
}

#[derive(Debug)]
struct StructField {
    attr: Option<Attribute>,
    name: String,
    r#type: Type,
}

impl TryFrom<&mut Parser> for StructField {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let peek = value
            .first()
            .ok_or(error!("StructField", format!("Expected more tokens")))?;

        let (attr, next) = if peek == &Token::Char('#') {
            (
                Some(error!(Attribute::try_from(&mut *value), "StructField")?),
                value.pop_front_err("StructField")?,
            )
        } else {
            (None, value.pop_front_err("StructField")?)
        };

        Ok(match next {
            Token::Identifier(iden) => {
                let name = iden;

                let next = value.pop_front_err("StructField")?;
                if next != Token::Keyword(Keywords::LeftArrow) {
                    return Err(error!(
                        "StructField",
                        format!("Expected arror keyword, got {next:#?}")
                    ))?;
                }

                let r#type = error!(Type::try_from(&mut *value), "StructField")?;

                StructField { attr, name, r#type }
            }
            token => {
                return Err(error!(
                    "StructField",
                    format!("Expected identifier, got {token:#?}")
                ))
            }
        })
    }
}

impl ToString for StructField {
    fn to_string(&self) -> String {
        format!(
            "{}{}: {}",
            if let Some(attr) = &self.attr {
                format!("{}\n", attr.to_string())
            } else {
                format!("")
            },
            self.name,
            self.r#type.to_string()
        )
    }
}
