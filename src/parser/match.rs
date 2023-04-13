use either::Either;

use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error, exp::Exp, r#type::NamespacedType, range::Range, Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub struct Match {
    against: Exp,
    branches: Vec<Branch>,
}

impl TryFrom<&mut Parser> for Match {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Match")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Match",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value.pop_front_err("Match")?;
        if next != Token::Keyword(Keywords::Match) {
            return Err(error!(
                "Match",
                format!("Expected Match keyword, got {next:#?}"),
            ));
        }

        let against = error!(Exp::try_from(&mut *value), "Match")?;
        let mut branches = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Match", format!("Expected more tokens"),))?;

            if *peek == Token::ParenClose {
                value.pop_front();
                break;
            } else {
                branches.push(error!(Branch::try_from(&mut *value), "Match")?);
            }
        }

        Ok(Match { against, branches })
    }
}

impl ToString for Match {
    fn to_string(&self) -> String {
        format!(
            "match {} {{{}}}",
            self.against.to_string(),
            self.branches.iter().fold(String::new(), |str, branch| {
                format!("{str}\n{},", branch.to_string())
            })
        )
    }
}

#[derive(Debug)]
pub(crate) struct Branch {
    pattern: Pattern,
    check: Option<Exp>,
    ret: Exp,
}

impl TryFrom<&mut Parser> for Branch {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Branch")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Branch",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let pattern = error!(Pattern::try_from(&mut *value), "Branch")?;
        println!("{pattern:#?}");

        let ret = if value.first() == Some(&Token::Keyword(Keywords::If)) {
            value.pop_front();

            Ok(Branch {
                pattern,
                check: Some(error!(Exp::try_from(&mut *value), "Branch")?),
                ret: error!(Exp::try_from(&mut *value), "Branch")?,
            })
        } else {
            Ok(Branch {
                pattern,
                check: None,
                ret: error!(Exp::try_from(&mut *value), "Branch")?,
            })
        };

        let next = value.pop_front_err("Branch close")?;
        if next != Token::ParenClose {
            Err(error!(
                "Branch",
                format!("Expected ParenClose, got {next:#?}"),
            ))
        } else {
            ret
        }
    }
}

impl ToString for Branch {
    fn to_string(&self) -> String {
        format!(
            "{}{} => {}",
            self.pattern.to_string(),
            if let Some(check) = &self.check {
                format!(" if {}", check.to_string())
            } else {
                format!("")
            },
            self.ret.to_string()
        )
    }
}

#[derive(Debug)]
pub enum Pattern {
    Variable(String),
    Literal(Literals),
    Touple(Vec<Self>),
    Array(Vec<Self>),
    Capture(String, Box<Self>),
    Range(Range),
    Enum(NamespacedType),
    EnumVars(NamespacedType, Vec<Self>),
    Struct(NamespacedType, Vec<Either<(String, Self), String>>),
    Rest,
    Or(Vec<Self>),
}

impl TryFrom<&mut Parser> for Pattern {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Pattern")?;
        Ok(match next {
            Token::Identifier(iden) if matches!(value.first(), Some(&Token::Char('@'))) => {
                value.pop_front();
                Self::Capture(
                    iden,
                    Box::new(error!(Self::try_from(&mut *value), "Pattern")?),
                )
            }
            iden @ Token::Identifier(_)
                if value.first() == Some(&Token::Keyword(Keywords::LeftArrow)) =>
            {
                value.tokens.push_front(iden);
                Self::Enum(error!(NamespacedType::try_from(&mut *value), "Pattern")?)
            }
            Token::Identifier(iden) => Self::Variable(iden),
            lit @ Token::Literal(Literals::Int(..)) if value.first() == Some(&Token::DoubleDot) => {
                value.tokens.push_front(lit);

                Self::Range(error!(Range::try_from(&mut *value), "Pattern")?)
            }
            Token::Literal(literal) => Self::Literal(literal),
            Token::BracketOpen => {
                let mut elems = vec![];

                loop {
                    let peek = value
                        .first()
                        .ok_or(error!("Pattern", format!("Expected more tokens")))?;

                    match peek {
                        Token::BracketClose => {
                            value.pop_front();
                            break;
                        }
                        _ => elems.push(error!(Self::try_from(&mut *value), "Pattern")?),
                    }
                }

                Self::Array(elems)
            }
            Token::AngleBracketOpen => {
                let mut elems = vec![];

                loop {
                    let peek = value
                        .first()
                        .ok_or(error!("Pattern", format!("Expected more tokens")))?;

                    match peek {
                        Token::AngleBracketClose => {
                            value.pop_front();
                            break;
                        }
                        _ => elems.push(error!(Self::try_from(&mut *value), "Pattern")?),
                    }
                }

                Self::Touple(elems)
            }
            Token::DoubleDot => Self::Rest,
            Token::ParenOpen => {
                let next = value.pop_front_err("Pattern")?;
                let ret = match next {
                    Token::Keyword(Keywords::Or) => {
                        let mut pats = vec![];

                        loop {
                            let peek = value
                                .first()
                                .ok_or(error!("Pattern", format!("Expected more tokens")))?;
                            if peek == &Token::ParenClose {
                                value.pop_front();
                                return Ok(Self::Or(pats));
                            }

                            pats.push(error!(Self::try_from(&mut *value), "Pattern")?)
                        }
                    }
                    iden @ Token::Identifier(_)
                        if value.first() != Some(&Token::ParenClose) =>
                    {
                        value.tokens.push_front(iden);
                        let namespace = error!(NamespacedType::try_from(&mut *value), "Pattern")?;

                        return Ok(
                            match value
                                .first()
                                .ok_or(error!("Pattern", format!("Expected more tokens")))?
                            {
                                Token::ParenClose => {
                                    value.pop_front();
                                    Self::Enum(namespace)
                                }
                                Token::CurlyOpen => {
                                    value.pop_front();
                                    let mut fields = vec![];

                                    loop {
                                        let peek = value.first().ok_or(error!(
                                            "Pattern",
                                            format!("Expected more tokens")
                                        ))?;
                                        if peek == &Token::CurlyClose {
                                            value.pop_front();
                                            let next = value.pop_front_err("Pattern")?;
                                            if next != Token::ParenClose {
                                                return Err(error!("Pattern", format!("Expected parenClose, got {next:#?}")))
                                            }

                                            break Self::Struct(namespace, fields);
                                        }

                                        if value.nth(1) == Some(&Token::Keyword(Keywords::LeftArrow)) {
                                            let field = match value.pop_front_err("Patter")? {
                                                Token::Identifier(iden) => iden,
                                                token => {
                                                    return Err(error!(
                                                        "Pattern",
                                                        format!("Expected iden, got {token:#?}")
                                                    ))
                                                }
                                            };
                                            value.pop_front();
                                            let pat =
                                                error!(Self::try_from(&mut *value), "Pattern")?;

                                            fields.push(Either::Left((field, pat)))
                                        } else {
                                            match value.pop_front_err("Pattern")? {
                                                Token::Identifier(iden) => {
                                                    fields.push(Either::Right(iden))
                                                }
                                                token => {
                                                    return Err(error!(
                                                        "Pattern",
                                                        format!("Expected iden, got {token:#?}")
                                                    ))
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    let mut pats = vec![];

                                    loop {
                                        let peek = value.first().ok_or(error!(
                                            "Pattern",
                                            format!("Expected more tokens")
                                        ))?;
                                        if peek == &Token::ParenClose {
                                            value.pop_front();
                                            break Self::EnumVars(namespace, pats);
                                        }

                                        pats.push(error!(Self::try_from(&mut *value), "Pattern")?)
                                    }
                                }
                            },
                        );
                    }
                    Token::Identifier(iden) => Self::Variable(iden),
                    Token::Literal(literal) => Self::Literal(literal),
                    Token::ParenOpen => error!(Self::try_from(&mut *value), "Pattern")?,
                    _ => todo!(),
                };

                let next = value.pop_front_err("Pattern")?;
                if next != Token::ParenClose {
                    return Err(error!(
                        "Pattern",
                        format!("Expected parenClose, got {next:#?}")
                    ));
                }

                ret
            }
            _ => todo!(),
        })
    }
}

impl ToString for Pattern {
    fn to_string(&self) -> String {
        match self {
            Self::Variable(var) => format!("{var}"),
            Self::Literal(literal) => format!("{}", literal.to_string()),
            Self::Touple(pats) => format!(
                "({})",
                &if pats.is_empty() {
                    format!(", ")
                } else {
                    pats.iter().fold(String::new(), |str, pat| {
                        format!("{str}, {}", pat.to_string())
                    })
                }[2..]
            ),
            Self::Array(pats) => format!(
                "[{}]",
                &if pats.is_empty() {
                    format!(", ")
                } else {
                    pats.iter().fold(String::new(), |str, pat| {
                        format!("{str}, {}", pat.to_string())
                    })
                }[2..]
            ),
            Self::Capture(capture, pat) => format!("{capture}@{}", pat.to_string()),
            Self::Range(range) => format!("{}", range.to_string()),
            Self::Enum(path) => format!("{}", path.to_string()),
            Self::EnumVars(path, pats) => format!(
                "{}({})",
                path.to_string(),
                &if pats.is_empty() {
                    format!(", ")
                } else {
                    pats.iter().fold(String::new(), |str, pat| {
                        format!("{str}, {}", pat.to_string())
                    })
                }[2..]
            ),
            Self::Struct(path, fields) => format!(
                "{}{{{}}}",
                path.to_string(),
                &if fields.is_empty() {
                    format!(", ")
                } else {
                    fields.iter().fold(String::new(), |str, field| {
                        format!(
                            "{str}, {}",
                            match field {
                                Either::Left((name, path)) =>
                                    format!("{name}: {}", path.to_string()),
                                Either::Right(name) => format!("{name}"),
                            }
                        )
                    })
                }[2..]
            ),
            Self::Rest => format!(".."),
            Self::Or(ors) => format!(
                "{}",
                &if ors.is_empty() {
                    format!("| ")
                } else {
                    ors.iter().fold(String::new(), |str, or| {
                        format!("{str}| {}", or.to_string())
                    })
                }[2..]
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_match, Match::try_from, "match.lt");
    snapshot!(test_match_rust, Match::try_from, "match.lt", rust);
}
