use crate::tokenizer::{Keywords, Token};

use super::{
    attribute::Attribute,
    error,
    r#struct::StructFields,
    r#type::{Generic, Type},
    Parser, ParserError, ParserErrorStack,
};
#[derive(Debug)]
pub struct Enum {
    name: String,
    generics: Vec<Generic>,
    variants: Vec<Variant>,
}

impl TryFrom<&mut Parser> for Enum {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Enum", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Enum", value.pop_front(), [Token::Keyword(Keywords::Enum)])?;
        let name = error!("Enum", value);

        let mut generics = vec![];

        loop {
            let peek = value.first_err("Enum")?;

            if let Token::Char(':') = peek {
                generics.push(error!(Generic::try_from(&mut *value), "Enum")?)
            } else {
                break;
            }
        }

        let mut variants = vec![];

        loop {
            let peek = value.first_err("Enum")?;

            if peek == &Token::ParenClose {
                value.pop_front();
                break;
            }

            variants.push(error!(Variant::try_from(&mut *value), "Enum")?);
        }

        Ok(Self {
            name,
            generics,
            variants,
        })
    }
}

impl ToString for Enum {
    fn to_string(&self) -> String {
        format!(
            "enum {}{} {{{}}}",
            self.name,
            if !self.generics.is_empty() {
                format!(
                    "<{}>",
                    &self.generics.iter().fold(String::new(), |str, generic| {
                        format!("{str}, {}", generic.to_string())
                    })[2..]
                )
            } else {
                format!("")
            },
            &self.variants.iter().fold(String::new(), |str, variant| {
                format!("{str}, {}", variant.to_string())
            })[2..]
        )
    }
}

#[derive(Debug)]
enum Variant {
    Simple(String),
    WithType(String, Vec<Type>),
    Struct(String, StructFields),
    Attr(Attribute, Box<Variant>),
}

impl TryFrom<&mut Parser> for Variant {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        if value.first() == Some(&Token::Char('#')) {
            return Ok(Self::Attr(
                error!(Attribute::try_from(&mut *value), "Variant")?,
                Box::new(error!(Variant::try_from(&mut *value), "Variant")?),
            ));
        }
        match error!("Variant", value.pop_front(), [Token::Identifier(_), Token::ParenOpen])? {
            Token::Identifier(iden) => Ok(Variant::Simple(iden)),
            Token::ParenOpen => {
                let name = error!("Variant", value);

                match value.first_err("Variant")?
                {
                    &Token::ParenClose => {
                        value.pop_front();
                        return Ok(Variant::Simple(name));
                    }
                    &Token::CurlyOpen => {
                        let fields = error!(StructFields::try_from(&mut *value), "Variant")?;

                        match error!("Variant", value.pop_front(), [Token::ParenClose])? {
                            Token::ParenClose => Ok(Variant::Struct(name, fields)),
                            _ => unreachable!()
                        }
                    }
                    _ => {
                        let mut r#types = vec![];

                        loop {
                            let peek = value.first_err("Variant")?;

                            if peek == &Token::ParenClose {
                                value.pop_front();
                                break;
                            }

                            r#types.push(error!(Type::try_from(&mut *value), "Variant")?)
                        }

                        Ok(Variant::WithType(name, r#types))
                    }
                }
            }
            _ => unreachable!()
        }
    }
}

impl ToString for Variant {
    fn to_string(&self) -> String {
        match self {
            Self::Simple(name) => format!("{name}"),
            Self::WithType(name, types) => format!(
                "{name}({})",
                &types.iter().fold(String::new(), |str, r#type| {
                    format!("{str}, {}", r#type.to_string())
                })[2..]
            ),
            Self::Struct(name, fields) => format!("{name} {}", fields.to_string()),
            Self::Attr(attr, variant) => format!("{}\n{}", attr.to_string(), variant.to_string()),
        }
    }
}
