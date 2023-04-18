pub mod args;
pub mod r#as;
pub mod attribute;
pub mod defun;
pub mod r#do;
pub mod r#enum;
pub mod exp;
pub mod file;
pub mod r#if;
pub mod r#impl;
pub mod lambda;
pub mod r#let;
pub mod r#match;
pub mod module;
pub mod range;
pub mod r#struct;
mod tests;
pub mod r#trait;
pub mod r#type;
pub mod r#use;

use std::{collections::VecDeque, path::PathBuf};

use std::ops::Index;

use crate::tokenizer::{Token, Tokens};

macro_rules! error {
    ($error:expr, $name:literal) => {
        $error.map_err(|mut err| {
            err.stack.push(ParserErrorStack {
                name: $name,
                file: file!(),
                location: (line!(), column!()),
            });
            err
        })
    };
    ($initial:expr, $err:expr$(,)?) => {
        ParserError {
            stack: vec![ParserErrorStack {
                name: $initial,
                file: file!(),
                location: (line!(), column!()),
            }],
            err: $err,
        }
    };
}

pub(crate) use error;

use self::file::File;

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
    file: &'static str,
    location: (u32, u32),
}

impl std::fmt::Display for ParserErrorStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}:{};{})",
            self.name, self.file, self.location.0, self.location.0
        )
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens: VecDeque::from(tokens.0),
        }
    }

    pub fn to_file_str(&mut self) -> String {
        match File::try_from(self) {
            Ok(file) => file.to_string(),
            Err(err) => panic!("{}", err.to_string()),
        }
    }

    fn first_err(&self, func: &'static str) -> Result<&Token, ParserError> {
        if self.tokens.len() > 0 {
            Ok(self.tokens.index(0))
        } else {
            Err(error!(func, format!("Expected more tokens")))
        }
    }

    fn first(&self) -> Option<&Token> {
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

    fn pop_front_err(&mut self, func: &'static str) -> Result<Token, ParserError> {
        self.tokens
            .pop_front()
            .ok_or(error!(func, format!("Expected more tokens")))
    }

    fn pop_front(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }
}

pub(crate) fn from_file(path: &PathBuf) -> String {
    Parser::new(
        match match std::fs::read_to_string(&path) {
            Ok(file) => file,
            Err(err) => panic!("Couldn't open the file at {path:#?}: {err}"),
        }
        .parse()
        {
            Ok(tokens) => tokens,
            Err(err) => panic!("Couldn't tokenize file at {path:#?}: {err}"),
        },
    )
    .to_file_str()
}
