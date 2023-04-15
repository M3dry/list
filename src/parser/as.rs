use crate::tokenizer::{Keywords, Token};

use super::{
    error,
    exp::Exp,
    r#type::Type,
    Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub struct As {
    exp: Exp,
    to: Type,
}
impl TryFrom<&mut Parser> for As {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("As")?;
        if next != Token::ParenOpen {
            return Err(error!("As", format!("Expected parenOpen, got {next:#?}")));
        }
        let next = value.pop_front_err("As")?;
        if next != Token::Keyword(Keywords::As) {
            return Err(error!("As", format!("Expected as, got {next:#?}")));
        }

        let exp = error!(Exp::try_from(&mut *value), "Exp")?;
        let to = error!(Type::try_from(&mut *value), "Exp")?;

        let next = value.pop_front_err("As")?;
        if next != Token::ParenClose {
            return Err(error!("As", format!("Expected parenClose, got {next:#?}")));
        }

        Ok(Self { exp, to })
    }
}

impl ToString for As {
    fn to_string(&self) -> String {
        format!("({} as {})", self.exp.to_string(), self.to.to_string())
    }
}
