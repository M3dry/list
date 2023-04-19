use crate::tokenizer::{Token, Keywords};

use super::{Parser, ParserError, ParserErrorStack, error, args::Args, exp::Exp};

#[derive(Debug)]
pub struct Lambda {
    args: Args,
    body: Exp,
}

impl TryFrom<&mut Parser> for Lambda {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Lambda", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Lambda", value.pop_front(), [Token::Keyword(Keywords::Lambda)])?;
        let args = error!(Args::try_from(&mut *value), "Lambda")?;
        let body = error!(Exp::try_from(&mut *value), "Lambda")?;
        let _ = error!("Lambda", value.pop_front(), [Token::ParenClose])?;

        Ok(Lambda { args, body })
    }
}

impl ToString for Lambda {
    fn to_string(&self) -> String {
        format!("{} {}", self.args.to_string(), self.body.to_string())
    }
}
