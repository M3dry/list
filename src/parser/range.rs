use crate::tokenizer::{Int, Literals, Token};

use super::{error, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub enum Range {
    Normal(Int, Int),
    Inclusive(Int, Int),
    InfiniteUpper(Int),
}

impl TryFrom<&mut Parser> for Range {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Range")?;
        match next {
            Token::Literal(Literals::Int(start)) => {
                let next = value.pop_front_err("Range")?;
                if next != Token::DoubleDot {
                    return Err(error!(
                        "Range",
                        format!("Expected doubleDot, got {next:#?}")
                    ));
                }

                let next = value.first();
                match next {
                    Some(Token::Literal(Literals::Int(_))) => {
                        if let Token::Literal(Literals::Int(end)) = value.pop_front().unwrap() {
                            Ok(Self::Normal(start, end))
                        } else {
                            unreachable!()
                        }
                    }
                    Some(Token::Char('=')) => {
                        value.pop_front();
                        let next = value.pop_front_err("Range")?;
                        match next {
                            Token::Literal(Literals::Int(end)) => Ok(Self::Inclusive(start, end)),
                            token => Err(error!("Range", format!("Expected int, got {token:#?}"))),
                        }
                    }
                    _ => Ok(Self::InfiniteUpper(start)),
                }
            }
            token => Err(error!("Range", format!("Expected int, got {token:#?}"))),
        }
    }
}

impl ToString for Range {
    fn to_string(&self) -> String {
        match self {
            Self::Normal(start, end) => format!("{}..{}", start.to_string(), end.to_string()),
            Self::Inclusive(start, end) => format!("{}..={}", start.to_string(), end.to_string()),
            Self::InfiniteUpper(start) => format!("{}..", start.to_string()),
        }
    }
}
