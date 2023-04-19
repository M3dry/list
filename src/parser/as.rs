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
        let _ = error!("As", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("As", value.pop_front(), [Token::Keyword(Keywords::As)])?;
        let exp = error!(Exp::try_from(&mut *value), "Exp")?;
        let to = error!(Type::try_from(&mut *value), "Exp")?;
        let _ = error!("As", value.pop_front(), [Token::ParenClose])?;

        Ok(Self { exp, to })
    }
}

impl ToString for As {
    fn to_string(&self) -> String {
        format!("({} as {})", self.exp.to_string(), self.to.to_string())
    }
}
