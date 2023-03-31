use crate::{
    tokenizer::{Keywords, Literals, Token},
};

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
        let next = value.pop_front_err("Match", "Expected more tokens")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Match",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value.pop_front_err("Match", "Expected more tokens")?;
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
            "match {{{}}} {{{}}}",
            self.against.to_string(),
            self.branches.iter().fold(String::new(), |str, branch| {
                format!("{str}\n{}", branch.to_string())
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
        let next = value.pop_front_err("Branch", "Expected more tokens")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Branch",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

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

        // let next = value.pop_front_err("Branch", "Expected more tokens")?;
        // let ret = match next {
        //     Token::Literal(literal) => Ok(Branch {
        //         pattern: Pattern::Literal(literal),
        //         check: None,
        //         ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
        //     }),
        //     Token::Identifier(iden) if value.first() != Some(&Token::Keyword(Keywords::Arrow)) => Ok(Branch {
        //         pattern: Pattern::Var(iden),
        //         check: None,
        //         ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
        //     }),
        //     Token::ParenOpen => {
        //         let mut tokens = vec![];

        //         while !matches!(
        //             value.first(),
        //             Some(Token::ParenClose | Token::Keyword(Keywords::If))
        //         ) {
        //             tokens.push(value.pop_front().unwrap())
        //         }

        //         let pattern = error!(
        //             Pattern::try_from(&mut Parser::new(crate::tokenizer::Tokens(tokens))),
        //             "Branch"
        //         )?;
        //         let next = value.pop_front_err("Branch", "Expected more tokens")?;

        //         match next {
        //             Token::Keyword(Keywords::If) => {
        //                 let check = error!(Exp::try_from(&mut *value), "Branch")?;
        //                 let next = value
        //                     .pop_front()
        //                     .ok_or(error!("Branch", format!("Expected more tokens"),))?;
        //                 if next != Token::ParenClose {
        //                     return Err(error!(
        //                         "Branch",
        //                         format!("Expected ParenClose, got {next:#?}"),
        //                     ));
        //                 }
        //                 let ret = Ok(Branch {
        //                     pattern,
        //                     check: Some(check),
        //                     ret: error!(Exp::try_from(&mut *value), "Branch")?,
        //                 });

        //                 ret
        //             }
        //             Token::ParenClose => Ok(Branch {
        //                 pattern,
        //                 check: None,
        //                 ret: error!(Exp::try_from(&mut *value), "Branch")?,
        //             }),
        //             _ => unreachable!(),
        //         }
        //     }
        //     _ => unreachable!(),
        // };

        let next = value.pop_front_err("Branch close", "Expected more tokens")?;
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
            "{}{} => {{{}}}",
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
            let ret = if value.nth(2) == Some(&Token::Keyword(Keywords::Arrow)) {
                return Ok(Pattern::Type(error!(TypeCreation::try_from(&mut *value), "Pattern")?))
            } else {
                value.pop_front();
                Ok(
                    match value.pop_front_err("Pattern", "Expected more tokens")? {
                        Token::Literal(literal) => Pattern::Literal(literal),
                        Token::Identifier(name) => Pattern::Var(name),
                        _ => todo!(),
                    },
                )
            };

            let next = value.pop_front_err("Pattern", "Expected more tokens")?;
            if next != Token::ParenClose {
                return Err(error!("Pattern", format!("Expected parenClose, got {next:#?}")))
            }

            ret
        } else if value.nth(1) == Some(&Token::Keyword(Keywords::Arrow)) {
            return Ok(Pattern::Type(error!(TypeCreation::try_from(&mut *value), "Pattern")?))
        } else {
            Ok(
                match value.pop_front_err("Pattern", "Expected more tokens")? {
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
