use either::Either;

use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error, exp::Exp, r#type::NamespacedType, range::Range, Parser, ParserError, ParserErrorStack, Error,
};

#[derive(Debug)]
pub struct Match {
    against: Exp,
    branches: Vec<Branch>,
}

impl TryFrom<&mut Parser> for Match {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Match", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Match", value.pop_front(), [Token::Keyword(Keywords::Match)])?;
        let against = error!(Exp::try_from(&mut *value), "Match")?;
        let mut branches = vec![];

        loop {
            let peek = value.first_err("Match")?;

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
        let _ = error!("Branch", value.pop_front(), [Token::ParenOpen])?;
        let pattern = error!(Pattern::try_from(&mut *value), "Branch")?;
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
        let _ = error!("Branch close", value.pop_front(), [Token::ParenClose])?;

        ret
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
        let next = error!("Pattern", value.pop_front(), [Token::Identifier(_), Token::Literal(_), Token::BracketOpen, Token::AngleBracketOpen, Token::DoubleDot, Token::ParenOpen])?;
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
            lit @ Token::Literal(Literals::Int(..) | Literals::Char(_)) if value.first() == Some(&Token::DoubleDot) => {
                value.tokens.push_front(lit);

                Self::Range(match error!(Range::try_from(&mut *value), "Pattern")? {
                    Range::Normal(start, end)
                        if matches!(start, Exp::Literal(_)) && matches!(end, Exp::Literal(_)) =>
                    {
                        Range::Normal(start, end)
                    }
                    Range::Inclusive(start, end)
                        if matches!(start, Exp::Literal(_)) && matches!(end, Exp::Literal(_)) =>
                    {
                        Range::Inclusive(start, end)
                    }
                    Range::Infinite(start) if matches!(start, Exp::Literal(_)) => {
                        Range::Infinite(start)
                    }
                    range => {
                        return Err(error!(
                            "Pattern",
                            Error::Other(format!("Expected range to consist of literals, got {range:#?}"))
                        ))
                    }
                })
            }
            Token::Literal(literal) => Self::Literal(literal),
            Token::BracketOpen => {
                let mut elems = vec![];

                loop {
                    let peek = value.first_err("Pattern")?;

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
                    let peek = value.first_err("Pattern")?;

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
                let ret = match error!("Pattern", value.pop_front(), [Token::Keyword(Keywords::Or), Token::Identifier(_), Token::ParenOpen, Token::Literal(_)])? {
                    Token::Keyword(Keywords::Or) => {
                        let mut pats = vec![];

                        loop {
                            let peek = value.first_err("Pattern")?;

                            if peek == &Token::ParenClose {
                                value.pop_front();
                                return Ok(Self::Or(pats));
                            }

                            pats.push(error!(Self::try_from(&mut *value), "Pattern")?)
                        }
                    }
                    iden @ Token::Identifier(_) if value.first() != Some(&Token::ParenClose) => {
                        value.tokens.push_front(iden);
                        let namespace = error!(NamespacedType::try_from(&mut *value), "Pattern")?;

                        return Ok(
                            match value.first_err("Pattern")?
                            {
                                Token::ParenClose => {
                                    value.pop_front();
                                    Self::Enum(namespace)
                                }
                                Token::CurlyOpen => {
                                    value.pop_front();
                                    let mut fields = vec![];

                                    loop {
                                        let peek = value.first_err("Pattern")?;
                                        if peek == &Token::CurlyClose {
                                            value.pop_front();
                                            let _ = error!("Pattern", value.pop_front(), [Token::ParenClose])?;

                                            break Self::Struct(namespace, fields);
                                        }

                                        if value.nth(1)
                                            == Some(&Token::Keyword(Keywords::LeftArrow))
                                        {
                                            let field = match error!("Pattern", value.pop_front(), [Token::Identifier(_)])? {
                                                Token::Identifier(iden) => iden,
                                                _ => unreachable!(),
                                            };
                                            value.pop_front();
                                            let pat =
                                                error!(Self::try_from(&mut *value), "Pattern")?;

                                            fields.push(Either::Left((field, pat)))
                                        } else {
                                            match error!("Pattern", value.pop_front(), [Token::Identifier(_)])? {
                                                Token::Identifier(iden) => {
                                                    fields.push(Either::Right(iden))
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    let mut pats = vec![];

                                    loop {
                                        let peek = value.first_err("Pattern")?;
                                        if peek == &Token::ParenClose {
                                            value.pop_front();
                                            break Self::EnumVars(namespace, pats);
                                        }

                                        pats.push(error!(Self::try_from(&mut *value), "Pattern")?)
                                    }
                                }
                            },
                        )
                    }
                    Token::Identifier(iden) => Self::Variable(iden),
                    Token::Literal(literal) => Self::Literal(literal),
                    Token::ParenOpen => error!(Self::try_from(&mut *value), "Pattern")?,
                    _ => todo!(),
                };

                let _ = error!("Pattern", value.pop_front(), [Token::ParenClose])?;

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
