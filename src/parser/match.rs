use crate::{tokenizer::{Token, Keywords}, pattern::{PatternTree, PatternToken}};

use super::{Parser, ParserError, ParserErrorStack, error, exp::Exp};

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
                .ok_or(error!("Match branches", format!("Expected more tokens"),))?;

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
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Branch {
    pattern: PatternTree,
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

        let next = value.pop_front_err("Branch", "Expected more tokens")?;
        let ret = match next {
            Token::Literal(literal) => Ok(Branch {
                pattern: PatternTree::new(PatternToken::Literal(literal)),
                check: None,
                ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
            }),
            Token::Identifier(iden) => Ok(Branch {
                pattern: PatternTree::new(PatternToken::Identifier(iden)),
                check: None,
                ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
            }),
            Token::ParenOpen => {
                let mut tokens = vec![];

                while !matches!(
                    value.first(),
                    Some(Token::ParenClose | Token::Keyword(Keywords::If))
                ) {
                    tokens.push(value.pop_front().unwrap())
                }

                let pattern = error!(PatternTree::try_from(&mut *value), "Branch")?;

                let next = value.pop_front_err("Branch", "Expected more tokens")?;
                match next {
                    Token::Keyword(Keywords::If) => {
                        let check =
                            error!(Exp::try_from(&mut *value), "Branch paren/if-condition")?;
                        let next = value.pop_front().ok_or(error!(
                            "Branch paren/if/close",
                            format!("Expected more tokens"),
                        ))?;
                        if next != Token::ParenClose {
                            return Err(error!(
                                "Branch paren/if/close",
                                format!("Expected ParenClose, got {next:#?}"),
                            ));
                        }
                        let ret = Ok(Branch {
                            pattern,
                            check: Some(check),
                            ret: error!(Exp::try_from(&mut *value), "Branch")?,
                        });

                        ret
                    }
                    Token::ParenClose => Ok(Branch {
                        pattern,
                        check: None,
                        ret: error!(Exp::try_from(&mut *value), "Branch")?,
                    }),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };

        let next = value.pop_front_err("Branch close", "Expected more tokens")?;
        if next != Token::ParenClose {
            Err(error!(
                "Branch close",
                format!("Expected ParenClose, got {next:#?}"),
            ))
        } else {
            ret
        }
    }
}