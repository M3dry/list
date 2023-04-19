use crate::tokenizer::{Keywords, Token};

use super::{
    error,
    r#type::{Generic, Type},
    Parser, ParserError, ParserErrorStack, Error,
};

#[derive(Debug)]
pub(crate) enum Arg {
    Generic(Generic),
    Named(String, Type),
    Simple(String),
    SelfA(Type),
}

impl TryFrom<&mut Parser> for Arg {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = error!("Arg", value.pop_front(), [Token::Ref, Token::Char(':'), Token::Identifier(_)])?;
        let name = match next {
            Token::Identifier(iden) if &iden == "self" => return Ok(Arg::SelfA(Type::SelfA)),
            w @ Token::Ref => {
                value.tokens.push_front(w);
                let ret = error!(Type::try_from(&mut *value), "Arg")?;

                match &ret {
                    Type::Ref(_, t) | Type::RefMut(_, t) if matches!(**t, Type::SelfA) => {
                        return Ok(Self::SelfA(ret))
                    }
                    token => return Err(error!("Arg", Error::Other(format!("Expected self, got {token:#?}")))),
                }
            }
            w @ Token::Char(':') => {
                value.tokens.push_front(w);
                return Ok(Arg::Generic(error!(Generic::try_from(&mut *value), "Arg")?));
            }
            Token::Identifier(iden) => iden,
            _ => unreachable!(),
        };

        let next = value.first();
        if next != Some(&Token::Keyword(Keywords::LeftArrow)) {
            return Ok(Arg::Simple(name));
        }
        value.pop_front();

        let arg_type = error!(Type::try_from(&mut *value), "Arg type")?;

        Ok(Arg::Named(name, arg_type))
    }
}

#[derive(Debug)]
pub struct ArgsTyped {
    lifetimes: Vec<String>,
    generics: Vec<Generic>,
    selft: Option<Type>,
    args: Vec<(String, Type)>,
}

impl TryFrom<&mut Parser> for ArgsTyped {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut args = vec![];
        let mut generics = vec![];
        let mut lifetimes = vec![];

        let _ = error!("ArgsTyped", value.pop_front(), [Token::ParenOpen])?;

        loop {
            let peek = value.first_err("ArgsTyped")?;

            match peek {
                Token::Char(':') => {
                    generics.push(error!(Generic::try_from(&mut *value), "ArgsTyped")?)
                }
                Token::BackTick => {
                    value.pop_front();
                    match error!("ArgsTyped", value.pop_front(), [Token::Identifier(_)])? {
                        Token::Identifier(iden) => lifetimes.push(iden),
                        _ => unreachable!(),
                    }
                }
                _ => break,
            }
        }

        let mut selft = None;

        loop {
            if value.first() == Some(&Token::ParenClose) {
                value.pop_front();
                break;
            }
            let arg = error!(Arg::try_from(&mut *value), "ArgsTyped")?;
            match arg {
                Arg::Generic(_) => return Err(error!("ArgsTyped", Error::Other(format!("Expected a named arg, got a generic, those should be defined before named args")))),
                Arg::Simple(_) => {
                    return Err(error!(
                        "ArgsTyped",
                        Error::Other(format!("Expected named arg, got a simple arg")),
                    ))
                }
                Arg::SelfA(selft2) => {
                    selft = Some(selft2)
                },
                Arg::Named(name, arg_type) => args.push((name, arg_type)),
            }
        }

        Ok(ArgsTyped {
            lifetimes,
            selft,
            generics,
            args,
        })
    }
}

impl ToString for ArgsTyped {
    fn to_string(&self) -> String {
        format!(
            "{}({}{})",
            if !self.generics.is_empty() || !self.lifetimes.is_empty() {
                format!("<{}>", {
                    let lifetimes = self
                        .lifetimes
                        .iter()
                        .map(|lifetime| format!("'{lifetime}"))
                        .collect::<Vec<String>>()
                        .join(", ");
                    let generics = self
                        .generics
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
            } else {
                format!("")
            },
            if let Some(selft) = &self.selft {
                format!(
                    "{}{}",
                    selft.to_string(),
                    if self.args.is_empty() {
                        format!("")
                    } else {
                        format!(", ")
                    }
                )
            } else {
                format!("")
            },
            if !self.args.is_empty() {
                (&self.args.iter().fold(String::new(), |str, arg| {
                    format!("{str}, {}: {}", arg.0, arg.1.to_string())
                })[2..])
                    .to_string()
            } else {
                format!("")
            }
        )
    }
}

#[derive(Debug)]
pub struct Args(Vec<String>);

impl TryFrom<&mut Parser> for Args {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Args", value.pop_front(), [Token::ParenOpen])?;
        let mut args = vec![];

        loop {
            if value.first() == Some(&Token::ParenClose) {
                value.pop_front();
                break;
            }

            let arg = error!(Arg::try_from(&mut *value), "Args")?;
            match arg {
                Arg::Simple(name) => args.push(name),
                _ => return Err(error!("Args arg", Error::Other(format!("Expected just simple args")))),
            }
        }

        Ok(Self(args))
    }
}

impl ToString for Args {
    fn to_string(&self) -> String {
        format!(
            "|{}|",
            &self
                .0
                .iter()
                .fold(String::new(), |str, arg| { format!("{str}, {arg}") })[2..]
        )
    }
}
