use either::Either;

use crate::tokenizer::{Keywords, Token};

use super::{
    error,
    exp::TurboFish,
    file::FileOps,
    Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub struct Trait {
    name: Either<String, TurboFish>,
    body: Vec<FileOps>,
}

impl TryFrom<&mut Parser> for Trait {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Trait")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Trait",
                format!("Expected parenOpen, got {next:#?}")
            ));
        }

        let next = value.pop_front_err("Trait")?;
        if next != Token::Keyword(Keywords::Trait) {
            return Err(error!(
                "Trait",
                format!("Expected trait keyword, got {next:#?}")
            ));
        }

        let name = if value.nth(1) == Some(&Token::Keyword(Keywords::TurboStart)) {
            Either::Right(error!(TurboFish::try_from(&mut *value), "Trait")?)
        } else {
            match value.pop_front_err("Trait")? {
                Token::Identifier(iden) => Either::Left(iden),
                token => return Err(error!("Trait", format!("Expected iden, got {token:#?}"))),
            }
        };
        let mut body = vec![];
        let next = value.pop_front_err("Trait")?;
        if next != Token::BracketOpen {
            return Err(error!(
                "Trait",
                format!("Expected bracketOpen, got {next:#?}")
            ));
        }

        loop {
            let peek = value
                .first()
                .ok_or(error!("Trait", format!("Expected more tokens")))?;

            if peek == &Token::BracketClose {
                value.pop_front();
                break;
            }

            body.push(match error!(FileOps::try_from(&mut *value), "Impl")? {
                file @ (FileOps::Use(_)
                | FileOps::Function(_)
                | FileOps::TypeAlias(_)
                | FileOps::Attribute(_)) => file,
                file => {
                    return Err(error!(
                        "Trait",
                        format!("Expected function, use, type alias or attribute, got {file:#?}")
                    ))
                }
            })
        }

        let next = value.pop_front_err("Trait")?;
        if next != Token::ParenClose {
            return Err(error!(
                "Trait",
                format!("Expected parenClose, got {next:#?}")
            ));
        }

        Ok(Self { name, body })
    }
}

impl ToString for Trait {
    fn to_string(&self) -> String {
        format!(
            "trait {} {{{}}}",
            match &self.name {
                Either::Right(name) => name.to_string(),
                Either::Left(turbofish) => turbofish.to_string(),
            },
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
