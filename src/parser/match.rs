use std::collections::VecDeque;

use crate::tokenizer::{Token, Keywords, Literals};

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
        todo!()
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

        let next = value.pop_front_err("Branch", "Expected more tokens")?;
        let ret = match next {
            Token::Literal(literal) => Ok(Branch {
                pattern: Pattern::Literal(literal),
                check: None,
                ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
            }),
            Token::Identifier(iden) => Ok(Branch {
                pattern: Pattern::Var(iden),
                check: None,
                ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
            }),
            Token::ParenOpen => {
                let mut tokens = VecDeque::new();

                while !matches!(
                    value.first(),
                    Some(Token::ParenClose | Token::Keyword(Keywords::If))
                ) {
                    tokens.push_back(value.pop_front().unwrap())
                }

                let pattern = error!(Pattern::try_from(tokens), "Branch")?;
                let next = value.pop_front_err("Branch", "Expected more tokens")?;

                match next {
                    Token::Keyword(Keywords::If) => {
                        let check =
                            error!(Exp::try_from(&mut *value), "Branch")?;
                        let next = value.pop_front().ok_or(error!(
                            "Branch",
                            format!("Expected more tokens"),
                        ))?;
                        if next != Token::ParenClose {
                            return Err(error!(
                                "Branch",
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

#[derive(Debug)]
enum Pattern {
    Literal(Literals),
    Var(String),
    Type(String, Box<Pattern>),
    Enum,
}

impl TryFrom<VecDeque<Token>> for Pattern {
    type Error = ParserError;

    fn try_from(mut value: VecDeque<Token>) -> Result<Self, Self::Error> {
        if value.len() == 1 {
            match value.pop_back().unwrap() {
                Token::Identifier(iden) => Ok(Pattern::Var(iden)),
                Token::Literal(literal) => Ok(Pattern::Literal(literal)),
                token => Err(error!("Pattern", format!("Expected an identifier, literal or parenClose, got {token:#?}"))),
            }
        } else {
            match value.pop_front().unwrap() {
                Token::Identifier(iden) => Ok(Pattern::Type(iden, Box::new(error!(Pattern::try_from(value), "Pattern")?))),
                token => Err(error!("Pattern", format!("Expected an identifier, got {token:#?}")))
            }
        }
    }
}