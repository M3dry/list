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
        let _ = error!("Mod", value.pop_front(), [Token::ParenOpen])?;
        let scope = Scope::try_from(&mut *value).unwrap();
        let _ = error!("Mod", value.pop_front(), [Token::Keyword(Keywords::Mod)])?;
        let name = error!("Mod", value);

        if error!("Mod", value.pop_front(), [Token::ParenClose, Token::BracketOpen])? == Token::ParenClose {
            return Ok(Self::Header(scope, name));
        }

        let mut body = vec![];

        loop {
            if value.first_err("Mod")? == &Token::BracketClose {
                value.pop_front();
                break;
            }

            body.push(error!(FileOps::try_from(&mut *value), "Mod")?)
        }

        let _ = error!("Mod", value.pop_front(), [Token::ParenClose])?;

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
