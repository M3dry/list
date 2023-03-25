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
        let next = value.pop_front_err("Lambda", "Expected more tokens")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Lambda",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value.pop_front_err("Lambda", "Expected more tokens")?;
        if next != Token::Keyword(Keywords::Lambda) {
            return Err(error!(
                "Lambda",
                format!("Expected Lambda Keyword, got {next:#?}"),
            ));
        }

        let args = error!(Args::try_from(&mut *value), "Lambda")?;
        let body = error!(Exp::try_from(&mut *value), "Lambda")?;

        let next = value.pop_front_err("Lambda", "Expected more tokens")?;
        if next != Token::ParenClose {
            return Err(error!(
                "Lambda",
                format!("Expected ParenClose, got {next:#?}"),
            ));
        }

        Ok(Lambda { args, body })
    }
}

impl ToString for Lambda {
    fn to_string(&self) -> String {
        format!("{} {{{}}}", self.args.to_string(), self.body.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_lambda, Lambda::try_from, "lambda.lt");
    snapshot!(test_lambda_rust, Lambda::try_from, "lambda.lt", rust);
}
