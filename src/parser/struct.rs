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
        let _ = error!("Struct", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Struct", value.pop_front(), [Token::Keyword(Keywords::Struct)])?;
        let name = if let Token::Identifier(iden) = error!("Struct", value.pop_front(), [Token::Identifier(_)])? {
            iden
        } else {
            unreachable!()
        };

        let mut generics = vec![];

        loop {
            let peek = value.first_err("Struct")?;

            if let Token::Char(':') = peek {
                generics.push(error!(Generic::try_from(&mut *value), "Struct")?)
            } else {
                break;
            }
        }

        if value.first() == Some(&Token::CurlyOpen) {
            let fields = error!(StructFields::try_from(&mut *value), "Struct")?;
            let _ = error!("Struct", value.pop_front(), [Token::ParenClose])?;

            Ok(Struct::Normal {
                name,
                generics,
                fields,
            })
        } else {
            let mut types = vec![];

            loop {
                let peek = value.first_err("Struct")?;

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
        let _ = error!("StructFields", value.pop_front(), [Token::CurlyOpen])?;
        let mut fields = vec![];

        loop {
            let peek = value.first_err("StructFields")?;
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
        let peek = value.first_err("StructField")?;

        let attr = if peek == &Token::Char('#') {
                Some(error!(Attribute::try_from(&mut *value), "StructField")?)
        } else {
            None
        };

        Ok(match error!("StructField", value.pop_front(), [Token::Identifier(_)])? {
            Token::Identifier(iden) => {
                let name = iden;
                let _ = error!("StructField", value.pop_front(), [Token::Keyword(Keywords::LeftArrow)])?;
                let r#type = error!(Type::try_from(&mut *value), "StructField")?;

                StructField { attr, name, r#type }
            }
            _ => unreachable!(),
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
