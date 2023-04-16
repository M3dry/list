use either::Either;

use crate::tokenizer::{Keywords, Token};

use super::{
    error, exp::TurboFish, file::FileOps, r#type::Generic, Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub enum Impl {
    /// (impl Trait -> Type [
    ///     (defun whatever()->i32 100)
    /// ])
    Trait {
        lifetimes: Vec<String>,
        generics: Vec<Generic>,
        r#trait: Either<String, TurboFish>,
        r#for: Either<String, TurboFish>,
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
        r#for: Either<String, TurboFish>,
        /// Can only be Function, Attribute, Use, TypeAlias
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

        let mut generics = vec![];
        let mut lifetimes = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Impl", format!("Expected more tokens")))?;

            match peek {
                Token::Char(':') => generics.push(error!(Generic::try_from(&mut *value), "Impl")?),
                Token::BackTick => {
                    value.pop_front();
                    match value.pop_front_err("Impl")? {
                        Token::Identifier(iden) => lifetimes.push(iden),
                        token => {
                            return Err(error!(
                                "Impl",
                                format!("Expected identifier, got {token:#?}")
                            ))
                        }
                    }
                }
                _ => break,
            }
        }

        let name = if value.nth(1) == Some(&Token::Keyword(Keywords::TurboStart)) {
            Either::Right(error!(TurboFish::try_from(&mut *value), "Impl")?)
        } else {
            match value.pop_front_err("Impl")? {
                Token::Identifier(iden) => Either::Left(iden),
                token => {
                    return Err(error!(
                        "Impl",
                        format!("Expected iden or turbofish, got {token:#?}")
                    ))
                }
            }
        };

        let for_trait = if value.first() == Some(&Token::Keyword(Keywords::LeftArrow)) {
            value.pop_front();
            let name = if value.nth(1) == Some(&Token::Keyword(Keywords::TurboStart)) {
                Either::Right(error!(TurboFish::try_from(&mut *value), "Impl")?)
            } else {
                match value.pop_front_err("Impl")? {
                    Token::Identifier(iden) => Either::Left(iden),
                    token => {
                        return Err(error!(
                            "Impl",
                            format!("Expected iden or turbofish, got {token:#?}")
                        ))
                    }
                }
            };
            Some(name)
        } else {
            None
        };

        let next = value.pop_front_err("Impl")?;
        if next != Token::BracketOpen {
            return Err(error!(
                "Impl",
                format!("Expected bracketOpen, got {next:#?}")
            ));
        }

        let mut funcs = vec![];

        loop {
            let peek = value
                .first()
                .ok_or(error!("Impl", format!("Expected more tokens")))?;

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
                        format!("Expected function, use, type alias or attribute, got {file:#?}")
                    ))
                }
            })
        }

        let next = value.pop_front_err("Impl")?;
        if next != Token::ParenClose {
            return Err(error!(
                "Impl",
                format!("Expected parenClose, got {next:#?}")
            ));
        }

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
                match r#trait {
                    Either::Left(str) => str.to_string(),
                    Either::Right(turbofish) => turbofish.to_string(),
                },
                match r#for {
                    Either::Left(str) => str.to_string(),
                    Either::Right(turbofish) => turbofish.to_string(),
                },
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
                match r#for {
                    Either::Left(str) => str.to_string(),
                    Either::Right(turbofish) => turbofish.to_string(),
                },
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
