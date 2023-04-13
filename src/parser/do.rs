use either::Either;

use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error, exp::Exp, r#type::NamespacedType, range::Range, Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub struct Do(Vec<DoActions>);

impl TryFrom<&mut Parser> for Do {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Do")?;
        if next != Token::ParenOpen {
            return Err(error!("Do", format!("Expected parenOpen, got {next:#?}")));
        }

        let next = value.pop_front_err("Do")?;
        if next != Token::Keyword(Keywords::Do) {
            return Err(error!("Do", format!("Expected parenOpen, got {next:#?}")));
        }

        let mut actions = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Do", format!("Expected more tokens")))?;

            if peek == &Token::ParenClose {
                value.pop_front();
                break Ok(Self(actions));
            }

            actions.push(error!(DoActions::try_from(&mut *value), "Do")?)
        }
    }
}

impl ToString for Do {
    fn to_string(&self) -> String {
        format!(
            "{{{}}}",
            self.0.iter().fold(String::new(), |str, doaction| {
                format!("{str}\n{}", doaction.to_string())
            })
        )
    }
}

#[derive(Debug)]
pub enum DoActions {
    Let(LetMatch, Exp),
    Exp(Exp),
    Return(Exp),
}

impl TryFrom<&mut Parser> for DoActions {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(
            match value
                .first()
                .ok_or(error!("DoActions", format!("Expected more tokens")))?
            {
                Token::Keyword(Keywords::Let) => {
                    value.pop_front();

                    let pattern = error!(LetMatch::try_from(&mut *value), "DoActions")?;
                    let next = value.pop_front_err("DoActions")?;
                    if next != Token::Keyword(Keywords::RightArrow) {
                        return Err(error!(
                            "DoActions",
                            format!("Expected rightArrow, got {next:#?}")
                        ));
                    }

                    Self::Let(pattern, error!(Exp::try_from(&mut *value), "DoActions")?)
                }
                Token::Keyword(Keywords::Return) => {
                    value.pop_front();

                    Self::Return(error!(Exp::try_from(&mut *value), "DoActions")?)
                }
                _ => Self::Exp(error!(Exp::try_from(&mut *value), "DoActions")?),
            },
        )
    }
}

impl ToString for DoActions {
    fn to_string(&self) -> String {
        match self {
            DoActions::Let(pattern, exp) => {
                format!("let {} = {};", pattern.to_string(), exp.to_string())
            }
            DoActions::Exp(exp) => format!("{};", exp.to_string()),
            DoActions::Return(exp) => format!("{}", exp.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum LetMatch {
    Touple(Vec<Self>),
    Array(Vec<Self>),
    Struct(NamespacedType, Vec<LetStructField>),
    Variable(String),
    Rest,
}

impl TryFrom<&mut Parser> for LetMatch {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("LetMatch")?;
        Ok(match next {
            iden @ Token::Identifier(_)
                if matches!(
                    value.first(),
                    Some(&Token::CurlyOpen | &Token::Keyword(Keywords::LeftArrow))
                ) =>
            {
                value.tokens.push_front(iden);
                let namespace = error!(NamespacedType::try_from(&mut *value), "LetMatch")?;
                let mut fields = vec![];
                value.pop_front();

                loop {
                    let peek = value.first().ok_or(error!("LetMatch", format!("Expected more tokens")))?;
                    match peek {
                        Token::CurlyClose => {
                            value.pop_front();
                            break Self::Struct(namespace, fields)
                        },
                        _ => fields.push(error!(LetStructField::try_from(&mut *value), "LetMatch")?),
                    }
                }
            }
            Token::Identifier(iden) => Self::Variable(iden),
            Token::AngleBracketOpen => {
                let mut matches = vec![];

                loop {
                    let peek = value
                        .first()
                        .ok_or(error!("LetMatch", format!("Expected more tokens")))?;

                    if peek == &Token::AngleBracketClose {
                        value.pop_front();
                        break Self::Touple(matches);
                    }

                    matches.push(error!(LetMatch::try_from(&mut *value), "LetMatch")?)
                }
            }
            Token::BracketOpen => {
                let mut matches = vec![];

                loop {
                    let peek = value
                        .first()
                        .ok_or(error!("LetMatch", format!("Expected more tokens")))?;

                    if peek == &Token::BracketClose {
                        value.pop_front();
                        break Self::Array(matches);
                    }

                    matches.push(error!(LetMatch::try_from(&mut *value), "LetMatch")?)
                }
            }
            Token::DoubleDot => Self::Rest,
            Token::ParenOpen => {
                let ret = error!(LetMatch::try_from(&mut *value), "LetMatch")?;

                let next = value.pop_front_err("LetMatch")?;
                if next != Token::ParenClose {
                    return Err(error!(
                        "LetMatch",
                        format!("Expected parenClose got {next:#?}")
                    ));
                }

                ret
            }
            token => return Err(error!("LetMatch", format!("Expected doubleDot, parenOpen, bracketOpen, angleBracketOpen, identifier, got {token:#?}"))),
        })
    }
}

impl ToString for LetMatch {
    fn to_string(&self) -> String {
        match self {
            Self::Touple(exps) => format!(
                "({})",
                &if exps.is_empty() {
                    format!(", ")
                } else {
                    exps.iter().fold(String::new(), |str, exp| {
                        format!("{str}, {}", exp.to_string())
                    })
                }[2..]
            ),
            Self::Array(exps) => format!(
                "[{}]",
                &if exps.is_empty() {
                    format!(", ")
                } else {
                    exps.iter().fold(String::new(), |str, exp| {
                        format!("{str}, {}", exp.to_string())
                    })
                }[2..]
            ),
            Self::Struct(namespace, fields) => format!(
                "{} {{{}}}",
                namespace.to_string(),
                &if fields.is_empty() {
                    format!(", ")
                } else {
                    fields.iter().fold(String::new(), |str, field| {
                        format!("{str}, {}", field.to_string())
                    })
                }[2..]
            ),
            Self::Variable(var) => var.to_string(),
            Self::Rest => format!(".."),
        }
    }
}

#[derive(Debug)]
pub enum LetStructField {
    Simple(String),
    Named(String, Box<LetMatch>),
    Rest,
}

impl TryFrom<&mut Parser> for LetStructField {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("LetStructField")?;
        Ok(match next {
            Token::DoubleDot => Self::Rest,
            Token::Identifier(name)
                if value.first() == Some(&Token::Keyword(Keywords::LeftArrow)) =>
            {
                value.pop_front();
                Self::Named(
                    name,
                    Box::new(error!(LetMatch::try_from(&mut *value), "LetStructField")?),
                )
            }
            Token::Identifier(iden) => Self::Simple(iden),
            token => {
                return Err(error!(
                    "LetStructField",
                    format!("Expected iden or doubleDot, got {token:#?}")
                ))
            }
        })
    }
}

impl ToString for LetStructField {
    fn to_string(&self) -> String {
        match self {
            LetStructField::Simple(name) => name.to_string(),
            LetStructField::Named(name, pat) => format!("{name}: {}", pat.to_string()),
            LetStructField::Rest => format!(".."),
        }
    }
}
