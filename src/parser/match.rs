use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error,
    exp::{Exp, TypeCreation},
    Parser, ParserError, ParserErrorStack,
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
enum Pattern {
    Type(TypeCreation),
    Literal(Literals),
    Var(String),
}

impl TryFrom<&mut Parser> for Pattern {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        if value.first() == Some(&Token::ParenOpen) {
            let ret = if matches!(
                value.nth(2),
                Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyOpen)
            ) {
                let type_creation = error!(TypeCreation::try_from(&mut *value), "Pattern")?;
                match type_creation {
                    TypeCreation::Vars(_, args)
                        if !args
                            .iter()
                            .all(|arg| matches!(arg, Exp::Literal(_) | Exp::Variable(_))) =>
                    {
                        return Err(error!(
                            "Pattern",
                            format!("Expected all args to be literals or identifiers")
                        ))
                    }
                    TypeCreation::Struct(_, fields)
                        if !fields.iter().all(|(_, exp)| {
                            matches!(exp, Exp::Literal(_) | Exp::Variable(_))
                        }) =>
                    {
                        return Err(error!(
                            "Pattern",
                            format!("Expected all fields to be literals or identifiers")
                        ))
                    }
                    _ => (),
                };
                return Ok(Pattern::Type(type_creation));
            } else {
                value.pop_front();
                Ok(
                    match value.pop_front_err("Pattern")? {
                        Token::Literal(literal) => Pattern::Literal(literal),
                        Token::Identifier(name) => Pattern::Var(name),
                        _ => todo!(),
                    },
                )
            };

            let next = value.pop_front_err("Pattern")?;
            if next != Token::ParenClose {
                return Err(error!(
                    "Pattern",
                    format!("Expected parenClose, got {next:#?}")
                ));
            }

            ret
        } else if matches!(
            value.nth(1),
            Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyOpen)
        ) {
            let type_creation = error!(TypeCreation::try_from(&mut *value), "Pattern")?;
            match type_creation {
                TypeCreation::Vars(_, args)
                    if !args
                        .iter()
                        .all(|arg| matches!(arg, Exp::Literal(_) | Exp::Variable(_))) =>
                {
                    return Err(error!(
                        "Pattern",
                        format!("Expected all args to be literals or identifiers")
                    ))
                }
                TypeCreation::Struct(_, fields)
                    if !fields
                        .iter()
                        .all(|(_, exp)| matches!(exp, Exp::Literal(_) | Exp::Variable(_))) =>
                {
                    return Err(error!(
                        "Pattern",
                        format!("Expected all fields to be literals or identifiers")
                    ))
                }
                _ => (),
            };

            return Ok(Pattern::Type(type_creation));
        } else {
            Ok(
                match value.pop_front_err("Pattern")? {
                    Token::Literal(literal) => Pattern::Literal(literal),
                    Token::Identifier(name) => Pattern::Var(name),
                    _ => todo!(),
                },
            )
        }
    }
}

impl ToString for Pattern {
    fn to_string(&self) -> String {
        match self {
            Pattern::Type(type_creation) => format!("{}", type_creation.to_string()),
            Pattern::Literal(literal) => format!("{}", literal.to_string()),
            Pattern::Var(var) => format!("{var}"),
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
