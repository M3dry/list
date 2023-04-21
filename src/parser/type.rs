use crate::tokenizer::{BuiltinTypes, Int, Keywords, Literals, Token};

use super::{error, turbofish::TurboIden, Parser, ParserError, ParserErrorStack};

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
        match error!(
            "Type",
            value.pop_front(),
            [
                Token::Ref,
                Token::Type(_),
                Token::Char(':'),
                Token::Identifier(_),
                Token::BracketOpen,
                Token::AngleBracketOpen,
                Token::ParenOpen
            ]
        )? {
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

                match error!(
                    "Type",
                    value.pop_front(),
                    [Token::BracketClose, Token::Char(';')]
                )? {
                    Token::BracketClose => Ok(Type::Array(r#type, None)),
                    Token::Char(';') => {
                        let Token::Literal(Literals::Int(Int(false, len))) =
                            error!("Type", value.pop_front(), [Token::Literal(Literals::Int(Int(false, _)))])?
                        else {
                            unreachable!()
                        };

                        let _ = error!("Type", value.pop_front(), [Token::BracketClose])?;

                        Ok(Type::Array(r#type, Some(len as usize)))
                    }
                    _ => unreachable!(),
                }
            }
            Token::AngleBracketOpen => {
                let mut types = vec![];

                loop {
                    let peek = value.first_err("Type")?;

                    if peek == &Token::AngleBracketClose {
                        value.pop_front();
                        break;
                    }

                    types.push(error!(Type::try_from(&mut *value), "Type")?);
                }

                Ok(Type::Touple(types))
            }
            Token::ParenOpen => {
                match error!("Type", value.pop_front(), [Token::Identifier(_)])? {
                    Token::Identifier(iden) => {
                        let mut types = vec![];

                        while value.first() != Some(&Token::ParenClose) {
                            types.push(error!(Type::try_from(&mut *value), "Type")?);
                        }
                        value.pop_front();

                        Ok(Type::Complex(iden, types))
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
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
        let _ = error!("Lifetimes", value.pop_front(), [Token::Slash])?;
        let mut lifetimes = vec![];

        loop {
            let peek = value.first_err("Lifetimes")?;

            if peek == &Token::Slash {
                value.pop_front();
                break;
            }

            lifetimes.push(error!("Lifetimes", value));
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
    Space(TurboIden, Box<NamespacedType>),
    Str(TurboIden),
}

impl TryFrom<&mut Parser> for NamespacedType {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let name = error!(TurboIden::try_from(&mut *value), "NamespacedType")?;

        Ok(
            if value.first() == Some(&Token::Keyword(Keywords::LeftArrow)) {
                value.pop_front();
                Self::Space(
                    name,
                    Box::new(error!(Self::try_from(&mut *value), "NamespacedType")?),
                )
            } else {
                Self::Str(name)
            },
        )
    }
}

impl ToString for NamespacedType {
    fn to_string(&self) -> String {
        match self {
            Self::Space(name, namespaces) => {
                format!("{}::{}", name.to_string(), namespaces.to_string())
            }
            Self::Str(name) => format!("{}", name.to_string()),
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
        let _ = error!("Generic", value.pop_front(), [Token::Char(':')])?;
        let name = error!("Generic", value);

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
        let _ = error!("Constraints", value.pop_front(), [Token::Slash])?;
        let mut constraints = vec![];

        loop {
            let next = error!(
                "Constraints",
                value.pop_front(),
                [Token::Slash, Token::Identifier(_)]
            )?;
            match next {
                Token::Slash => {
                    return Ok(Self(constraints));
                }
                Token::Identifier(iden) => constraints.push(iden),
                _ => unreachable!(),
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

#[derive(Debug)]
pub enum TypeAlias {
    Alias { name: String, r#type: Type },
    Def(String),
}

impl TryFrom<&mut Parser> for TypeAlias {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("TypeALias", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!(
            "TypeALias",
            value.pop_front(),
            [Token::Keyword(Keywords::Type)]
        )?;
        let name = error!("TypeAlias", value);

        if value.first() == Some(&Token::ParenClose) {
            value.pop_front();
            return Ok(Self::Def(name));
        }

        let r#type = error!(Type::try_from(&mut *value), "TypeALias")?;
        let _ = error!("TypeALias", value.pop_front(), [Token::ParenClose])?;

        Ok(Self::Alias { name, r#type })
    }
}

impl ToString for TypeAlias {
    fn to_string(&self) -> String {
        match self {
            Self::Alias { name, r#type } => {
                format!("type {} = {};", name, r#type.to_string())
            }
            Self::Def(name) => format!("type {name};"),
        }
    }
}
