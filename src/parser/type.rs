use crate::tokenizer::{BuiltinTypes, Int, Keywords, Literals, Token};

use super::{error, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub enum Type {
    Ref(Option<Lifetimes>, Box<Type>),
    RefMut(Option<Lifetimes>, Box<Type>),
    Builtin(BuiltinTypes),
    Generic(Generic),
    Custom(String),
    Complex(String, Vec<Type>),
    Array(Box<Type>, Option<usize>),
    Touple(Vec<Type>),
    SelfA,
    SelfT,
}

impl TryFrom<&mut Parser> for Type {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        match value.pop_front_err("Type")? {
            Token::Ref if value.first() == Some(&Token::Slash) => {
                let lifetimes = error!(Lifetimes::try_from(&mut *value), "Type")?;

                if let Some(&Token::Keyword(Keywords::Mut)) = value.first() {
                    value.pop_front();
                    Ok(Self::RefMut(
                        Some(lifetimes),
                        Box::new(error!(Type::try_from(&mut *value), "Type")?),
                    ))
                } else {
                    Ok(Self::Ref(
                        Some(lifetimes),
                        Box::new(error!(Type::try_from(&mut *value), "Type")?),
                    ))
                }
            }
            Token::Ref if value.first() == Some(&Token::Keyword(Keywords::Mut)) => {
                value.pop_front();
                Ok(Type::RefMut(
                    None,
                    Box::new(error!(Type::try_from(&mut *value), "Type")?),
                ))
            }
            Token::Ref => Ok(Type::Ref(
                None,
                Box::new(error!(Type::try_from(&mut *value), "Type")?),
            )),
            Token::Type(builtin) => Ok(Type::Builtin(builtin)),
            Token::Char(':') => {
                value.tokens.push_front(Token::Char(':'));
                Ok(Type::Generic(error!(
                    Generic::try_from(&mut *value),
                    "Type"
                )?))
            }
            Token::Identifier(iden) if &iden == "self" => Ok(Self::SelfA),
            Token::Identifier(iden) if &iden == "Self" => Ok(Self::SelfT),
            Token::Identifier(iden) => Ok(Type::Custom(iden)),
            Token::BracketOpen => {
                let r#type = Box::new(error!(Type::try_from(&mut *value), "Type")?);

                match value.pop_front_err("Type")? {
                    Token::BracketClose => Ok(Type::Array(r#type, None)),
                    Token::Char(';') => {
                        let Token::Literal(Literals::Int(Int(neg, len))) =
                            value.pop_front_err("Type")?
                        else {
                                return Err(error!("Type", format!("")))
                        };

                        if neg {
                            return Err(error!(
                                "Type",
                                format!("Expected non negative number in array")
                            ));
                        }

                        let next = value.pop_front_err("Type")?;
                        if next != Token::BracketClose {
                            return Err(error!(
                                "Type",
                                format!("Expected bracketClose, got {next:#?}")
                            ));
                        }

                        Ok(Type::Array(r#type, Some(len as usize)))
                    }
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
                let next = value.pop_front_err("Type")?;
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
            Type::Ref(None, r#type) => format!("&{}", r#type.to_string()),
            Type::Ref(Some(lifetimes), r#type) => {
                format!("&{}{}", lifetimes.to_string(), r#type.to_string())
            }
            Type::RefMut(None, r#type) => format!("&mut {}", r#type.to_string()),
            Type::RefMut(Some(lifetimes), r#type) => {
                format!("&{}mut {}", lifetimes.to_string(), r#type.to_string())
            }
            Type::Builtin(builtin) => builtin.to_string(),
            Type::Generic(generic) => generic.to_string(),
            Type::Custom(name) => name.to_string(),
            Type::Complex(name, types) => format!(
                "{name}<{}>",
                &types.into_iter().fold(String::new(), |str, r#type| {
                    format!("{str}, {}", r#type.to_string())
                })[2..]
            ),
            Type::Array(r#type, None) => format!("[{}]", r#type.to_string()),
            Type::Array(r#type, Some(len)) => format!("[{}; {len}]", r#type.to_string()),
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
            Self::SelfT => format!("Self"),
            Self::SelfA => format!("self"),
        }
    }
}

#[derive(Debug)]
pub struct Lifetimes(Vec<String>);

impl TryFrom<&mut Parser> for Lifetimes {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Lifetimes")?;
        if next != Token::Slash {
            return Err(error!(
                "Lifetimes",
                format!("Expected slash, got {next:#?}")
            ));
        }

        let mut lifetimes = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Lifetimes", format!("Expected more tokens")))?;

            if peek == &Token::Slash {
                value.pop_front();
                break;
            }

            match value.pop_front_err("Lifetimes")? {
                Token::Identifier(iden) => lifetimes.push(iden),
                token => {
                    return Err(error!(
                        "Lifetimes",
                        format!("Expected identifier, got {token:#?}")
                    ))
                }
            }
        }

        Ok(Self(lifetimes))
    }
}

impl ToString for Lifetimes {
    fn to_string(&self) -> String {
        if self.0.is_empty() {
            format!("")
        } else {
            self.0
                .iter()
                .fold(String::new(), |str, lifetime| format!("{str}'{lifetime} "))
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
        Ok(match value.pop_front_err("NamespacedType")? {
            Token::Identifier(iden)
                if value.first() == Some(&Token::Keyword(Keywords::LeftArrow)) =>
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
        })
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

#[derive(Debug)]
pub enum Generic {
    Constrained {
        name: String,
        constraints: Constraints,
    },
    Use(String),
}

impl TryFrom<&mut Parser> for Generic {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Generic")?;
        if !matches!(next, Token::Char(':')) {
            return Err(error!(
                "Generic",
                format!("Expected char ':', got {next:#?}")
            ));
        }

        let name = match value.pop_front_err("Generic")? {
            Token::Identifier(iden) => iden,
            token => {
                return Err(error!(
                    "Generic",
                    format!("Expected identifier, got {token:#?}")
                ))
            }
        };

        if value.first() == Some(&Token::Slash) {
            Ok(Self::Constrained {
                name,
                constraints: error!(Constraints::try_from(&mut *value), "Generic")?,
            })
        } else {
            Ok(Self::Use(name))
        }
    }
}

impl ToString for Generic {
    fn to_string(&self) -> String {
        match self {
            Self::Constrained { name, constraints } => {
                format!("{name}: {}", constraints.to_string())
            }
            Self::Use(name) => format!("{name}"),
        }
    }
}

#[derive(Debug)]
pub struct Constraints(Vec<String>);

impl TryFrom<&mut Parser> for Constraints {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Constraints")?;
        if next != Token::Slash {
            return Err(error!(
                "Constraints",
                format!("Expected slash, got {next:#?}")
            ));
        }

        let mut constraints = vec![];

        loop {
            let next = value.pop_front_err("Constraints")?;
            match next {
                Token::Slash => {
                    return Ok(Self(constraints));
                }
                Token::Identifier(iden) => constraints.push(iden),
                token => {
                    return Err(error!(
                        "Constraints",
                        format!("Expected slash or identifier, got {token:#?}")
                    ))
                }
            }
        }
    }
}

impl ToString for Constraints {
    fn to_string(&self) -> String {
        format!(
            "{}",
            &if self.0.is_empty() {
                format!(" + ")
            } else {
                self.0.iter().fold(String::new(), |str, constraint| {
                    format!("{str} + {}", constraint.to_string())
                })
            }[3..]
        )
    }
}
