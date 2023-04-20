use crate::tokenizer::{Keywords, Token};

use super::{error, r#type::Type, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub enum TurboIden {
    TurboFish(TurboFish),
    Identifier(String),
}

impl TryFrom<&mut Parser> for TurboIden {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(
            if value.nth(1) == Some(&Token::Keyword(Keywords::TurboStart)) {
                Self::TurboFish(error!(TurboFish::try_from(&mut *value), "TurboIden")?)
            } else if let Token::Identifier(iden) =
                error!("TurboIden", value.pop_front(), [Token::Identifier(_)])?
            {
                Self::Identifier(iden)
            } else {
                unreachable!()
            },
        )
    }
}

impl ToString for TurboIden {
    fn to_string(&self) -> String {
        match self {
            TurboIden::TurboFish(turbofish) => turbofish.to_string(),
            TurboIden::Identifier(iden) => iden.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct TurboFish(String, Type);

impl TryFrom<&mut Parser> for TurboFish {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let var = if let Token::Identifier(iden) =
            error!("TurboFish", value.pop_front(), [Token::Identifier(_)])?
        {
            iden
        } else {
            unreachable!()
        };

        let _ = error!(
            "TurboFish",
            value.pop_front(),
            [Token::Keyword(Keywords::TurboStart)]
        )?;
        let r#type = error!(Type::try_from(&mut *value), "TurboFish")?;
        let _ = error!("TurboFish", value.pop_front(), [Token::AngleBracketClose])?;

        Ok(Self(var, r#type))
    }
}

impl ToString for TurboFish {
    fn to_string(&self) -> String {
        format!("{}::<{}>", self.0, self.1.to_string())
    }
}
