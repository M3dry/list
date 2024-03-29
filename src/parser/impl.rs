use crate::tokenizer::{Keywords, Token};

use super::{
    error,
    file::FileOps,
    r#type::Generic,
    turbofish::TurboIden,
    Error, Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub enum Impl {
    /// (impl Trait -> Type [
    ///     (defun whatever()->i32 100)
    /// ])
    Trait {
        lifetimes: Vec<String>,
        generics: Vec<Generic>,
        r#trait: TurboIden,
        r#for: TurboIden,
        /// Can only be Function, Attribute, Use, TypeAlias
        body: Vec<FileOps>,
    },
    /// (impl Type [
    ///     (defun len(&self)->usize
    ///         self/len)
    /// ])
    Funcs {
        lifetimes: Vec<String>,
        generics: Vec<Generic>,
        r#for: TurboIden,
        /// Can only be Function, Attribute, Use, TypeAlias
        body: Vec<FileOps>,
    },
}

impl TryFrom<&mut Parser> for Impl {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Impl", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Impl", value.pop_front(), [Token::Keyword(Keywords::Impl)])?;
        let mut generics = vec![];
        let mut lifetimes = vec![];

        loop {
            match value.first_err("Impl")? {
                Token::Char(':') => generics.push(error!(Generic::try_from(&mut *value), "Impl")?),
                Token::BackTick => {
                    value.pop_front();
                    match error!("Impl", value.pop_front(), [Token::Identifier(_)])? {
                        Token::Identifier(iden) => lifetimes.push(iden),
                        _ => unreachable!(),
                    }
                }
                _ => break,
            }
        }

        let name = error!(TurboIden::try_from(&mut *value), "Impl")?;
        let for_trait = if value.first() == Some(&Token::Keyword(Keywords::LeftArrow)) {
            value.pop_front();

            Some(error!(TurboIden::try_from(&mut *value), "Impl")?)
        } else {
            None
        };

        let _ = error!("Impl", value.pop_front(), [Token::BracketOpen])?;
        let mut funcs = vec![];

        loop {
            let peek = value.first_err("Impl")?;

            if peek == &Token::BracketClose {
                value.pop_front();
                break;
            }

            funcs.push(match error!(FileOps::try_from(&mut *value), "Impl")? {
                file @ (FileOps::Use(_)
                | FileOps::Function(_)
                | FileOps::TypeAlias(_)
                | FileOps::Attribute(_)) => file,
                file => {
                    return Err(error!(
                        "Trait",
                        Error::Other(format!(
                            "Expected function, use, type alias or attribute, got {file:#?}"
                        ))
                    ))
                }
            })
        }

        let _ = error!("Impl", value.pop_front(), [Token::ParenClose])?;

        Ok(if let Some(r#trait) = for_trait {
            Self::Trait {
                lifetimes,
                generics,
                r#trait,
                r#for: name,
                body: funcs,
            }
        } else {
            Self::Funcs {
                lifetimes,
                generics,
                r#for: name,
                body: funcs,
            }
        })
    }
}

impl ToString for Impl {
    fn to_string(&self) -> String {
        match self {
            Self::Trait {
                lifetimes,
                generics,
                r#trait,
                r#for,
                body,
            } => format!(
                "impl{} {} for {} {{{}}}",
                if generics.is_empty() && lifetimes.is_empty() {
                    format!("")
                } else {
                    format!("<{}>", {
                        let lifetimes = lifetimes
                            .iter()
                            .map(|lifetime| format!("'{lifetime}"))
                            .collect::<Vec<String>>()
                            .join(", ");
                        let generics = generics
                            .iter()
                            .map(|generic| generic.to_string())
                            .collect::<Vec<String>>()
                            .join(", ");

                        if lifetimes.is_empty() {
                            generics
                        } else {
                            lifetimes + ", " + &generics
                        }
                    })
                },
                r#trait.to_string(),
                r#for.to_string(),
                &if body.is_empty() {
                    format!("\n")
                } else {
                    body.iter().fold(String::new(), |str, body| {
                        format!("{str}\n{}", body.to_string())
                    })
                }[1..]
            ),
            Self::Funcs {
                lifetimes,
                generics,
                r#for,
                body,
            } => format!(
                "impl{} {} {{{}}}",
                if generics.is_empty() && lifetimes.is_empty() {
                    format!("")
                } else {
                    format!("<{}>", {
                        let lifetimes = lifetimes
                            .iter()
                            .map(|lifetime| format!("'{lifetime}"))
                            .collect::<Vec<String>>()
                            .join(", ");
                        let generics = generics
                            .iter()
                            .map(|generic| generic.to_string())
                            .collect::<Vec<String>>()
                            .join(", ");

                        if lifetimes.is_empty() {
                            generics
                        } else {
                            lifetimes + ", " + &generics
                        }
                    })
                },
                r#for.to_string(),
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
