use crate::tokenizer::{Int, Literals, Token};

use super::{error, exp::Exp, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub enum Range {
    Normal(Exp, Exp),
    Inclusive(Exp, Exp),
    Infinite(Exp),
}

impl TryFrom<&mut Parser> for Range {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let paren = if value.first() == Some(&Token::ParenClose) {
            value.pop_front();
            true
        } else {
            false
        };
        let start = error!(Exp::try_from(&mut *value), "Range")?;
        let next = value.pop_front_err("Range")?;
        if next != Token::DoubleDot {
            return Err(error!(
                "Range",
                format!("Expected doubleDot, got {next:#?}")
            ));
        }
        let inclusive = match value.first() {
            Some(&Token::Char('=')) => {
                value.pop_front();
                true
            },
            Some(&Token::ParenClose) if paren => return Ok(Self::Infinite(start)),
            _ => false,
        };
        let end = error!(Exp::try_from(&mut *value), "Range")?;

        Ok(if inclusive {
            Self::Inclusive(start, end)
        } else {
            Self::Normal(start, end)
        })
    }
}

impl ToString for Range {
    fn to_string(&self) -> String {
        match self {
            Self::Normal(start, end) => format!("({}..{})", start.to_string(), end.to_string()),
            Self::Inclusive(start, end) => format!("({}..={})", start.to_string(), end.to_string()),
            Self::Infinite(start) => format!("({}..)", start.to_string()),
        }
    }
}
