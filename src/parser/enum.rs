use crate::tokenizer::{Keywords, Token};

use super::{error, r#struct::StructFields, r#type::Type, Parser, ParserError, ParserErrorStack};
#[derive(Debug)]
pub struct Enum {
    name: String,
    generics: Vec<String>,
    variants: Vec<Variant>,
}

impl TryFrom<&mut Parser> for Enum {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Enum", "Expected more tokens")?;
        if next != Token::ParenOpen {
            return Err(error!("Enum", format!("Expected ParenOpen, got {next:#?}")));
        }

        let next = value.pop_front_err("Enum", "Expected more tokens")?;
        if next != Token::Keyword(Keywords::Enum) {
            return Err(error!(
                "Enum",
                format!("Expected Enum keyword, got {next:#?}")
            ));
        }

        let next = value.pop_front_err("Enum", "Expected more tokens")?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(error!(
                "Enum",
                format!("Expected identifier, got {next:#?}")
            ));
        };

        let mut generics = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Enum", format!("Expected more tokens")))?;

            if let Token::Generic(_) = peek {
                generics.push(
                    if let Token::Generic(generic) = value.pop_front().unwrap() {
                        generic
                    } else {
                        panic!("wtf")
                    },
                )
            } else {
                break;
            }
        }

        let mut variants = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Enum", format!("Expected more tokens")))?;
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
                        format!("{str}, {generic}")
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
}

impl TryFrom<&mut Parser> for Variant {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        match value.pop_front_err("Variant", "Expected more tokens")? {
            Token::Identifier(iden) => Ok(Variant::Simple(iden)),
            Token::ParenOpen => {
                let name = match value.pop_front_err("Variant", "Expected more tokens")? {
                    Token::Identifier(iden) => iden,
                    token => {
                        return Err(error!(
                            "Variant",
                            format!("Expected identifier, got {token:#?}")
                        ))
                    }
                };

                match value
                    .first()
                    .ok_or(error!("Variant", format!("Expected more tokens")))?
                {
                    &Token::ParenClose => {
                        value.pop_front();
                        return Ok(Variant::Simple(name));
                    }
                    &Token::CurlyOpen => {
                        let fields = error!(StructFields::try_from(&mut *value), "Variant")?;

                        match value.pop_front_err("Variant", "Expected more tokens")? {
                            Token::ParenClose => Ok(Variant::Struct(name, fields)),
                            token => Err(error!(
                                "Variant",
                                format!("Expected ParenClose, got {token:#?}")
                            )),
                        }
                    }
                    _ => {
                        let mut r#types = vec![];

                        loop {
                            let peek = value
                                .first()
                                .ok_or(error!("Variant", format!("Expected more tokens")))?;

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
            token => Err(error!(
                "Variant",
                format!("Expected ParenOpen or identifier, got {token:#?}")
            )),
        }
    }
}

impl ToString for Variant {
    fn to_string(&self) -> String {
        match self {
            Variant::Simple(name) => format!("{name}"),
            Variant::WithType(name, types) => format!(
                "{name}({})",
                &types.iter().fold(String::new(), |str, r#type| {
                    format!("{str}, {}", r#type.to_string())
                })[2..]
            ),
            Variant::Struct(name, fields) => format!("{name} {}", fields.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_enum, Enum::try_from, "enum.lt");
    snapshot!(test_enum_rust, Enum::try_from, "enum.lt", rust);
}
