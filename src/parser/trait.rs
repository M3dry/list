use crate::tokenizer::{Keywords, Token};

use super::{
    error,
    turbofish::TurboIden,
    file::FileOps,
    Parser, ParserError, ParserErrorStack, Error,
};

#[derive(Debug)]
pub struct Trait {
    name: TurboIden,
    body: Vec<FileOps>,
}

impl TryFrom<&mut Parser> for Trait {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Trait", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Trait", value.pop_front(), [Token::Keyword(Keywords::Trait)])?;
        let name = error!(TurboIden::try_from(&mut *value), "Trait")?;
        let mut body = vec![];
        let _ = error!("Trait", value.pop_front(), [Token::BracketOpen])?;

        loop {
            let peek = value.first_err("Trait")?;

            if peek == &Token::BracketClose {
                value.pop_front();
                break;
            }

            body.push(match error!(FileOps::try_from(&mut *value), "Trait")? {
                file @ (FileOps::Use(_)
                | FileOps::Function(_)
                | FileOps::TypeAlias(_)
                | FileOps::Attribute(_)) => file,
                file => {
                    return Err(error!(
                        "Trait",
                        Error::Other(format!("Expected function, use, type alias or attribute, got {file:#?}"))
                    ))
                }
            })
        }

        let _ = error!("Trait", value.pop_front(), [Token::ParenClose])?;

        Ok(Self { name, body })
    }
}

impl ToString for Trait {
    fn to_string(&self) -> String {
        format!(
            "trait {} {{{}}}",
            self.name.to_string(),
            &if self.body.is_empty() {
                format!("\n")
            } else {
                self.body.iter().fold(String::new(), |str, file_op| {
                    format!("{str}\n{}", file_op.to_string())
                })
            }[1..]
        )
    }
}
