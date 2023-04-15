use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error, exp::Exp, file::FileOps, r#type::NamespacedType, range::Range, Parser, ParserError,
    ParserErrorStack,
};

#[derive(Debug)]
pub enum Impl {
    /// (impl Trait -> Type [
    ///     (defun whatever()->i32 100)
    /// ])
    Trait {
        r#trait: String,
        r#for: String,
        /// Can only be Function, Attribute, Use
        body: Vec<FileOps>,
    },
    /// (impl Type [
    ///     (defun len(&self)->usize
    ///         self/len)
    /// ])
    Funcs {
        r#for: String,
        /// Can only be Function, Attribute, Use
        body: Vec<FileOps>,
    },
}

impl TryFrom<&mut Parser> for Impl {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Impl")?;
        if next != Token::ParenOpen {
            return Err(error!("Impl", format!("Expected parenOpen, got {next:#?}")));
        }
        let next = value.pop_front_err("Impl")?;
        if next != Token::Keyword(Keywords::Impl) {
            return Err(error!("Impl", format!("Expected parenOpen, got {next:#?}")));
        }

        let ret = if value.nth(1) == Some(&Token::Keyword(Keywords::LeftArrow)) {
            let r#trait = match value.pop_front_err("Impl")? {
                Token::Identifier(iden) => iden,
                token => return Err(error!("Impl", format!("Expected iden, got {token:#?}"))),
            };
            value.pop_front();
            let r#for = match value.pop_front_err("Impl")? {
                Token::Identifier(iden) => iden,
                token => return Err(error!("Impl", format!("Expected iden, got {token:#?}"))),
            };
            let next = value.pop_front_err("Impl")?;
            if next != Token::BracketOpen {
                return Err(error!(
                    "Impl",
                    format!("Expected bracketOpen, got {next:#?}")
                ));
            }

            let mut file_ops = vec![];

            loop {
                let peek = value
                    .first()
                    .ok_or(error!("Imp", format!("Expected more tokens")))?;

                if peek == &Token::BracketClose {
                    value.pop_front();
                    break Self::Trait {
                        r#trait,
                        r#for,
                        body: file_ops,
                    };
                }

                file_ops.push(match error!(FileOps::try_from(&mut *value), "Impl")? {
                    file @ (FileOps::Use(_) | FileOps::Struct(_) | FileOps::Enum(_)) => {
                        return Err(error!(
                            "Impl",
                            format!("Expected function or attribute, got {file:#?}")
                        ))
                    }
                    file => file,
                })
            }
        } else {
            let r#for = match value.pop_front_err("Impl")? {
                Token::Identifier(iden) => iden,
                token => return Err(error!("Impl", format!("Expected iden, got {token:#?}"))),
            };
            let next = value.pop_front_err("Impl")?;
            if next != Token::BracketOpen {
                return Err(error!(
                    "Impl",
                    format!("Expected bracketOpen, got {next:#?}")
                ));
            }

            let mut file_ops = vec![];

            loop {
                let peek = value
                    .first()
                    .ok_or(error!("Imp", format!("Expected more tokens")))?;

                if peek == &Token::BracketClose {
                    value.pop_front();
                    break Self::Funcs {
                        r#for,
                        body: file_ops,
                    };
                }

                file_ops.push(error!(FileOps::try_from(&mut *value), "Impl")?)
            }
        };

        let next = value.pop_front_err("Impl")?;
        if next != Token::ParenClose {
            return Err(error!(
                "Impl",
                format!("Expected parenClose, got {next:#?}")
            ));
        }

        Ok(ret)
    }
}

impl ToString for Impl {
    fn to_string(&self) -> String {
        match self {
            Self::Trait {
                r#trait,
                r#for,
                body,
            } => format!(
                "impl {trait} for {for} {{{}}}",
                &if body.is_empty() {
                    format!("\n")
                } else {
                    body.iter().fold(String::new(), |str, body| {
                        format!("{str}\n{}", body.to_string())
                    })
                }[1..]
            ),
            Self::Funcs { r#for, body } => format!(
                "impl {for} {{{}}}",
                &if body.is_empty() {
                    format!("\n")
                } else {
                    body.iter().fold(String::new(), |str, body| {
                        format!("{str}\n{}", body.to_string())
                    })
                }[1..]
            ),
        }
    }
}
