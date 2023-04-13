use crate::tokenizer::{Keywords, Token};

use super::{r#type::{Generic, Type}, error, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub(crate) enum Arg {
    Generic(Generic),
    Named(String, Type),
    Simple(String),
}

impl TryFrom<&mut Parser> for Arg {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Arg")?;
        let name = match next {
            Token::Identifier(iden) => iden,
            w @ Token::Char(':') => {
                value.tokens.push_front(w);
                return Ok(Arg::Generic(error!(Generic::try_from(&mut *value), "Arg")?));
            }
            _ => {
                return Err(error!(
                    "Arg name",
                    format!("Expected identifier, got {next:#?}"),
                ));
            }
        };

        let next = value.first();
        if next != Some(&Token::Keyword(Keywords::Arrow)) {
            return Ok(Arg::Simple(name));
        }
        value.pop_front();

        let arg_type = error!(Type::try_from(&mut *value), "Arg type")?;

        Ok(Arg::Named(name, arg_type))
    }
}

#[derive(Debug)]
pub struct ArgsTyped {
    generics: Vec<Generic>,
    args: Vec<(String, Type)>,
}

impl TryFrom<&mut Parser> for ArgsTyped {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut args = vec![];
        let mut generics = vec![];

        let next = value.pop_front_err("ArgsTyped")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "ArgsTyped",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        while let Some(Token::Char(':')) = value.first() {
            generics.push(error!(Generic::try_from(&mut *value), "ArgsTyped")?);
        }

        loop {
            if value.first() == Some(&Token::ParenClose) {
                value.pop_front();
                break;
            }
            let arg = error!(Arg::try_from(&mut *value), "ArgsTyped")?;
            match arg {
                Arg::Generic(_) => return Err(error!("ArgsTyped", format!("Expected a named arg, got a generic, those should be defined before named args"))),
                Arg::Simple(_) => {
                    return Err(error!(
                        "ArgsTyped",
                        format!("Expected named arg, got a simple arg"),
                    ))
                }
                Arg::Named(name, arg_type) => args.push((name, arg_type)),
            }
        }

        Ok(ArgsTyped { generics, args })
    }
}

impl ToString for ArgsTyped {
    fn to_string(&self) -> String {
        format!(
            "{}({})",
            if !self.generics.is_empty() {
                format!(
                    "<{}>",
                    &self.generics.iter().fold(String::new(), |str, generic| {
                        format!("{str}, {}", generic.to_string())
                    })[2..]
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
        let next = value.pop_front_err("Args")?;
        if next != Token::ParenOpen {
            return Err(error!("Args", format!("Expected ParenOpen, got {next:#?}"),));
        }

        let mut args = vec![];
        loop {
            if value.first() == Some(&Token::ParenClose) {
                value.pop_front();
                break;
            }

            let arg = error!(Arg::try_from(&mut *value), "Args")?;
            match arg {
                Arg::Simple(name) => args.push(name),
                _ => return Err(error!("Args arg", format!("Expected just simple args"),)),
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
