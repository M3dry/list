use crate::tokenizer::{Keywords, Token};

use super::{error, r#type::Type, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub(crate) enum Arg {
    Generic(String),
    Named(String, Type),
    Simple(String),
}

impl TryFrom<&mut Parser> for Arg {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Arg", "Expected more tokens")?;
        let name = match next {
            Token::Identifier(iden) => iden,
            Token::Generic(generic) => return Ok(Arg::Generic(generic)),
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
    generics: Vec<String>,
    args: Vec<(String, Type)>,
}

impl TryFrom<&mut Parser> for ArgsTyped {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut args = vec![];
        let mut generics = vec![];

        let next = value.pop_front_err("ArgsTyped", "Expected more tokens")?;
        if next != Token::ParenOpen {
            return Err(error!(
                "ArgsTyped",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        while let Some(Token::Generic(_)) = value.first() {
            if let Some(Token::Generic(generic)) = value.pop_front() {
                generics.push(generic);
            } else {
                panic!("wtf")
            }
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
                        format!("{str}, {generic}")
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
        let next = value.pop_front_err("Args", "Expected more tokens")?;
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
                _ => {
                    return Err(error!(
                        "Args arg",
                        format!("Expected just simple and generic args"),
                    ))
                }
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
