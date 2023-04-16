use either::Either;

use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    attribute::Attribute,
    error,
    exp::Exp,
    r#type::{NamespacedType, TypeAlias},
    range::Range,
    Parser, ParserError, ParserErrorStack, r#use::Use,
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
    /// (do let
    ///     StructType { field1 field2->varname field3-><_ var2> .. }
    ///     <- (func arg1 arg2))
    Let(bool, Destructuring, Exp),
    Assignment(String, Exp),
    /// (do if true
    ///         break
    ///     elif (= x 10)
    ///         continue)
    If {
        condition: Exp,
        true_branch: Box<DoActions>,
        elif_branch: Vec<(Exp, DoActions)>,
        false_branch: Option<Box<DoActions>>,
    },
    /// (do
    ///     for i, val <- (variable.iter)
    ///         (println! "{}: {}" i val))
    For {
        vals: Destructuring,
        iter: Exp,
        body: Box<DoActions>,
    },
    /// (do loop
    ///     (if true break continue))
    Loop(Box<DoActions>),
    While(Exp, Box<DoActions>),
    TypeAlias(TypeAlias),
    Attribute(Attribute),
    Use(Use),
    Ret(Exp),
    Semicolon(Exp),
    Break,
    Continue,
}

impl TryFrom<&mut Parser> for DoActions {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(
            match value
                .first()
                .ok_or(error!("DoActions", format!("Expected more tokens")))?
            {
                Token::Char('#') => {
                    Self::Attribute(error!(Attribute::try_from(&mut *value), "DoActions")?)
                }
                Token::Keyword(Keywords::Let) => {
                    value.pop_front();

                    let mutable = if let Some(&Token::Keyword(Keywords::Mut)) = value.first() {
                        value.pop_front();
                        true
                    } else {
                        false
                    };

                    let pattern = error!(Destructuring::try_from(&mut *value), "DoActions")?;
                    let next = value.pop_front_err("DoActions")?;
                    if next != Token::Keyword(Keywords::RightArrow) {
                        return Err(error!(
                            "DoActions",
                            format!("Expected rightArrow, got {next:#?}")
                        ));
                    }

                    Self::Let(
                        mutable,
                        pattern,
                        error!(Exp::try_from(&mut *value), "DoActions")?,
                    )
                }
                Token::Identifier(_) => {
                    if value.nth(1) == Some(&Token::Keyword(Keywords::RightArrow)) {
                        let var = if let Some(Token::Identifier(iden)) = value.pop_front() {
                            iden
                        } else {
                            unreachable!()
                        };
                        value.pop_front();

                        Self::Assignment(var, error!(Exp::try_from(&mut *value), "DoActions")?)
                    } else {
                        let exp = error!(Exp::try_from(&mut *value), "DoActions")?;

                        if let Some(&Token::Char(';')) = value.first() {
                            value.pop_front();
                            Self::Semicolon(exp)
                        } else {
                            Self::Ret(exp)
                        }
                    }
                }
                Token::Keyword(Keywords::If) => {
                    value.pop_front();
                    let condition = error!(Exp::try_from(&mut *value), "DoActions")?;
                    let true_branch =
                        Box::new(error!(DoActions::try_from(&mut *value), "DoActions")?);
                    let mut elif_branch = vec![];

                    loop {
                        let peek = value.first();

                        if peek == Some(&Token::Keyword(Keywords::Elif)) {
                            value.pop_front();
                            let cond = error!(Exp::try_from(&mut *value), "DoActions")?;
                            let body = error!(DoActions::try_from(&mut *value), "DoActions")?;

                            elif_branch.push((cond, body))
                        }

                        break;
                    }

                    if let Some(&Token::Keyword(Keywords::Else)) = value.first() {
                        value.pop_front();

                        Self::If {
                            condition,
                            true_branch,
                            elif_branch,
                            false_branch: Some(Box::new(error!(
                                DoActions::try_from(&mut *value),
                                "DoActions"
                            )?)),
                        }
                    } else {
                        Self::If {
                            condition,
                            true_branch,
                            elif_branch,
                            false_branch: None,
                        }
                    }
                }
                Token::Keyword(Keywords::For) => {
                    value.pop_front();
                    let vals = error!(Destructuring::try_from(&mut *value), "DoActions")?;

                    let next = value.pop_front_err("DoActions")?;
                    if next != Token::Keyword(Keywords::RightArrow) {
                        return Err(error!(
                            "DoActions",
                            format!("Expected rightArrow, got {next:#?}")
                        ));
                    }

                    let iter = error!(Exp::try_from(&mut *value), "DoActions")?;
                    let body = Box::new(error!(DoActions::try_from(&mut *value), "DoActions")?);

                    Self::For { vals, iter, body }
                }
                Token::Keyword(Keywords::Loop) => {
                    value.pop_front();
                    Self::Loop(Box::new(error!(
                        DoActions::try_from(&mut *value),
                        "DoActions"
                    )?))
                }
                Token::Keyword(Keywords::While) => {
                    value.pop_front();
                    let cond = error!(Exp::try_from(&mut *value), "DoActions")?;
                    let body = Box::new(error!(DoActions::try_from(&mut *value), "DoActions")?);

                    Self::While(cond, body)
                }
                Token::Keyword(Keywords::Type) => {
                    Self::TypeAlias(error!(TypeAlias::try_from(&mut *value), "DoActions")?)
                }
                Token::Keyword(Keywords::Use) => {
                    Self::Use(error!(Use::try_from(&mut *value), "DoActions")?)
                }
                Token::Keyword(Keywords::Break) => {
                    value.pop_front();
                    Self::Break
                }
                Token::Keyword(Keywords::Continue) => {
                    value.pop_front();
                    Self::Continue
                }
                _ => {
                    let exp = error!(Exp::try_from(&mut *value), "DoActions")?;

                    if let Some(&Token::Char(';')) = value.first() {
                        value.pop_front();
                        Self::Semicolon(exp)
                    } else {
                        Self::Ret(exp)
                    }
                }
            },
        )
    }
}

impl ToString for DoActions {
    fn to_string(&self) -> String {
        match self {
            Self::Let(mutable, pattern, exp) => {
                format!(
                    "let{} {} = {};",
                    if *mutable {
                        format!(" mut")
                    } else {
                        format!("")
                    },
                    pattern.to_string(),
                    exp.to_string()
                )
            }
            Self::Assignment(var, exp) => format!("{var} = {};", exp.to_string()),
            Self::If {
                condition,
                true_branch,
                elif_branch,
                false_branch,
            } => {
                format!(
                    "{{if {} {{{}}}{}{}}}",
                    condition.to_string(),
                    true_branch.to_string(),
                    &if elif_branch.is_empty() {
                        format!(" else if ")
                    } else {
                        elif_branch.iter().fold(String::new(), |str, elif| {
                            format!(
                                "{str} else if {} {{{}}}",
                                elif.0.to_string(),
                                elif.1.to_string()
                            )
                        })
                    }[9..],
                    match false_branch {
                        Some(body) => format!(" else {{{}}}", body.to_string()),
                        None => format!(""),
                    }
                )
            }
            Self::For { vals, iter, body } => {
                format!(
                    "{{for {} in {} {{{}}}}};",
                    vals.to_string(),
                    iter.to_string(),
                    body.to_string()
                )
            }
            Self::Loop(body) => format!("loop {{{}}}", body.to_string()),
            Self::While(cond, body) => {
                format!("while {} {{{}}}", cond.to_string(), body.to_string())
            }
            Self::TypeAlias(type_alias) => type_alias.to_string(),
            Self::Attribute(attr) => attr.to_string(),
            Self::Use(r#use) => r#use.to_string(),
            Self::Ret(action) => action.to_string(),
            Self::Semicolon(exp) => format!("{};", exp.to_string()),
            Self::Break => format!("break;"),
            Self::Continue => format!("continue;"),
        }
    }
}

#[derive(Debug)]
pub enum Destructuring {
    Touple(Vec<Self>),
    Array(Vec<Self>),
    Struct(NamespacedType, Vec<LetStructField>),
    Variable(String),
    Rest,
}

impl TryFrom<&mut Parser> for Destructuring {
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

                    matches.push(error!(Destructuring::try_from(&mut *value), "LetMatch")?)
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

                    matches.push(error!(Destructuring::try_from(&mut *value), "LetMatch")?)
                }
            }
            Token::DoubleDot => Self::Rest,
            Token::ParenOpen => {
                let ret = error!(Destructuring::try_from(&mut *value), "LetMatch")?;

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

impl ToString for Destructuring {
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
    Named(String, Box<Destructuring>),
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
                    Box::new(error!(
                        Destructuring::try_from(&mut *value),
                        "LetStructField"
                    )?),
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
