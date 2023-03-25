pub mod args;
pub mod defun;
pub mod r#enum;
pub mod exp;
pub mod file;
pub mod r#if;
pub mod lambda;
pub mod r#let;
pub mod r#match;
pub mod r#struct;
pub mod r#type;

use std::collections::VecDeque;
use std::ops::Index;

use crate::tokenizer::{Token, Tokens};

macro_rules! error {
    ($error:expr, $name:literal) => {
        $error.map_err(|mut err| {
            err.stack.push(ParserErrorStack {
                name: $name,
                location: (line!(), column!()),
            });
            err
        })
    };
    ($initial:expr, $err:expr$(,)?) => {
        ParserError {
            stack: vec![ParserErrorStack {
                name: $initial,
                location: (line!(), column!()),
            }],
            err: $err,
        }
    };
}

macro_rules! snapshot {
    ($name:tt, $func:expr, $path:tt) => {
        snapshot!($name, |parser| format!("{:#?}", $func(parser)), $path, "../testdata/parser/");
    };
    ($name:tt, $func:expr, $path:tt, rust) => {
        snapshot!($name, |parser| match $func(parser) { Ok(res) => res.to_string(), Err(err) => format!("{err:#?}") }, $path, "../testdata/rust/");
    };
    ($name:tt, $func:expr, $path:tt, $out:literal) => {
        #[test]
        fn $name() {
            use crate::parser::Parser;

            let contents = include_str!(concat!("../../testdata/input/", $path));
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_path($out);
            settings.bind(|| {
                insta::assert_snapshot!(contents
                    .lines()
                    .filter_map(|line| if line != "" {
                        Some(format!("{line}\n{}", $func(&mut Parser::new(line.parse().unwrap()))))
                    } else {
                        None
                    })
                    .collect::<Vec<String>>()
                    .join("\n\n"));
            });
        }
    };
}

pub(crate) use error;
pub(crate) use snapshot;

use self::{defun::Defun, exp::Exp, r#enum::Enum, r#struct::Struct};

#[derive(Debug)]
pub struct ParserError {
    stack: Vec<ParserErrorStack>,
    err: String,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.err)?;
        for stack in &self.stack {
            write!(f, "{stack}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ParserErrorStack {
    name: &'static str,
    location: (u32, u32),
}

impl std::fmt::Display for ParserErrorStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({};{})", self.name, self.location.0, self.location.0)
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    // pub fn parse(tokens: Tokens, exp: bool) -> Result<(), ParserError> {
    //     let mut parser = Self {
    //         tokens: tokens.0.into_iter().peekable_nth(),
    //     };

    //     if exp {
    //         println!("{:#?}", Exp::try_from(&mut parser));
    //     } else {
    //         println!("{:#?}", File::try_from(&mut parser));
    //     }
    //     Ok(())
    // }

    // pub fn file(tokens: Tokens) -> Result<File, ParserError> {
    //     File::try_from(&mut Self::new(tokens))
    // }

    // pub fn r#enum(tokens: Tokens) -> Result<Enum, ParserError> {
    //     Enum::try_from(&mut Self::new(tokens))
    // }

    // pub fn r#struct(tokens: Tokens) -> Result<Struct, ParserError> {
    //     Struct::try_from(&mut Self::new(tokens))
    // }

    // pub fn function(tokens: Tokens) -> Result<Defun, ParserError> {
    //     Defun::try_from(&mut Self::new(tokens))
    // }

    // pub fn expression(tokens: Tokens) -> Result<Exp, ParserError> {
    //     Exp::try_from(&mut Self::new(tokens))
    // }

    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens: VecDeque::from(tokens.0),
        }
    }

    fn first(&mut self) -> Option<&Token> {
        if self.tokens.len() > 0 {
            Some(self.tokens.index(0))
        } else {
            None
        }
    }

    fn nth(&mut self, nth: usize) -> Option<&Token> {
        if self.tokens.len() > nth {
            Some(self.tokens.index(nth))
        } else {
            None
        }
    }

    fn pop_front_err(
        &mut self,
        func: &'static str,
        err: &'static str,
    ) -> Result<Token, ParserError> {
        self.tokens.pop_front().ok_or(error!(func, err.to_string()))
    }

    fn pop_front(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }
}