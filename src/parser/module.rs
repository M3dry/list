use crate::tokenizer::{Keywords, Token};

use super::{defun::Scope, error, file::FileOps, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub enum Mod {
    Full {
        scope: Scope,
        name: String,
        body: Vec<FileOps>,
    },
    Header(Scope, String),
}

impl TryFrom<&mut Parser> for Mod {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Mod")?;
        if next != Token::ParenOpen {
            return Err(error!("Mod", format!("Expected parenOpen, got {next:#?}")));
        }

        let scope = Scope::try_from(&mut *value).unwrap();
        let next = value.pop_front_err("Mod")?;
        if next != Token::Keyword(Keywords::Mod) {
            return Err(error!(
                "Mod",
                format!("Expected mod keyword, got {next:#?}")
            ));
        }

        let name = match value.pop_front_err("Mod")? {
            Token::Identifier(iden) => iden,
            token => return Err(error!("Mod", format!("Expected iden, got {token:#?}"))),
        };

        let next = value.pop_front_err("Mod")?;
        if next == Token::ParenClose {
            return Ok(Self::Header(scope, name));
        } else if next != Token::BracketOpen {
            return Err(error!(
                "Mod",
                format!("Expected bracketOpen, got {next:#?}")
            ));
        }

        let mut body = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Mod", format!("Expected more tokens")))?;

            if peek == &Token::BracketClose {
                value.pop_front();
                break;
            }

            body.push(error!(FileOps::try_from(&mut *value), "Mod")?)
        }

        let next = value.pop_front_err("Mod")?;
        if next != Token::ParenClose {
            return Err(error!("Mod", format!("Expected parenClose, got {next:#?}")));
        }

        Ok(Self::Full { scope, name, body })
    }
}

impl ToString for Mod {
    fn to_string(&self) -> String {
        match self {
            Self::Full { scope, name, body } => {
                format!(
                    "{}mod {} {{{}}}",
                    scope.to_string(),
                    name,
                    body.iter()
                        .map(|file_op| file_op.to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
            Self::Header(scope, name) => format!("{}mod {name};", scope.to_string()),
        }
    }
}
