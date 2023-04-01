use crate::tokenizer::{BuiltinTypes, Keywords, Token};

use super::{error, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub enum Type {
    Builtin(BuiltinTypes),
    Generic(String),
    Custom(String),
    Complex(String, Vec<Type>),
    Array(Box<Type>),
    Touple(Vec<Type>),
}

impl TryFrom<&mut Parser> for Type {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        match value.pop_front_err("Type", "Expected more tokens")? {
            Token::Type(builtin) => Ok(Type::Builtin(builtin)),
            Token::Generic(generic) => Ok(Type::Generic(generic)),
            Token::Identifier(iden) => Ok(Type::Custom(iden)),
            Token::BracketOpen => {
                let r#type = Box::new(error!(Type::try_from(&mut *value), "Type")?);

                match value.pop_front_err("Type", "Expected more tokens")? {
                    Token::BracketClose => Ok(Type::Array(r#type)),
                    token => Err(error!(
                        "Type",
                        format!("Expected BracketClose, got {token:#?}")
                    )),
                }
            }
            Token::AngleBracketOpen => {
                let mut types = vec![];

                loop {
                    let peek = value
                        .first()
                        .ok_or(error!("Type", format!("Expected more tokens")))?;

                    if peek == &Token::AngleBracketClose {
                        value.pop_front();
                        break;
                    }

                    types.push(error!(Type::try_from(&mut *value), "Type")?);
                }

                Ok(Type::Touple(types))
            }
            Token::ParenOpen => {
                let next = value.pop_front_err("Type", "Expected more tokens")?;
                match next {
                    Token::Type(builtin) => {
                        if value.pop_front() != Some(Token::ParenClose) {
                            Err(error!(
                                "Type complex/builtin",
                                format!("Expected an identifier, got a builtin type"),
                            ))
                        } else {
                            Ok(Type::Builtin(builtin))
                        }
                    }
                    Token::Identifier(iden) => {
                        let mut types = vec![];

                        while value.first() != Some(&Token::ParenClose) {
                            types.push(error!(Type::try_from(&mut *value), "Type")?);
                        }
                        value.pop_front();

                        Ok(Type::Complex(iden, types))
                    }
                    next => Err(error!(
                        "Type complex/other",
                        format!("Expected an identifier, got {next:#?}"),
                    )),
                }
            }
            token => Err(error!(
                "Type",
                format!("Expected type, indentifier or OpenParen, got {token:#?}"),
            )),
        }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Builtin(builtin) => builtin.to_string(),
            Type::Generic(generic) => generic.to_string(),
            Type::Custom(name) => name.to_string(),
            Type::Complex(name, types) => format!(
                "{name}<{}>",
                &types.into_iter().fold(String::new(), |str, r#type| {
                    format!("{str}, {}", r#type.to_string())
                })[2..]
            ),
            Type::Array(r#type) => format!("Vec<{}>", r#type.to_string()),
            Type::Touple(types) => format!(
                "({})",
                &if types.is_empty() {
                    format!(", ")
                } else {
                    types.into_iter().fold(String::new(), |str, r#type| {
                        format!("{str}, {}", r#type.to_string())
                    })
                }[2..]
            ),
        }
    }
}

#[derive(Debug)]
pub enum NamespacedType {
    Space(String, Box<NamespacedType>),
    Final(String),
}

impl TryFrom<&mut Parser> for NamespacedType {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(
            match value.pop_front_err("NamespacedType", "Expected more tokens")? {
                Token::Identifier(iden)
                    if value.first() == Some(&Token::Keyword(Keywords::Arrow)) =>
                {
                    value.pop_front();
                    NamespacedType::Space(
                        iden,
                        Box::new(error!(
                            NamespacedType::try_from(&mut *value),
                            "NamescapedType"
                        )?),
                    )
                }
                Token::Identifier(iden) => NamespacedType::Final(iden),
                token => {
                    return Err(error!(
                        "NamespacedType",
                        format!("Expected identifier, got {token:#?}")
                    ))
                }
            },
        )
    }
}

impl ToString for NamespacedType {
    fn to_string(&self) -> String {
        match self {
            Self::Space(name, namespaces) => format!("{name}::{}", namespaces.to_string()),
            Self::Final(name) => format!("{name}"),
        }
    }
}
