use crate::tokenizer::{Keywords, Token};

use super::{
    defun::Defun, error, r#enum::Enum, r#struct::Struct, Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub struct File {
    structs: Vec<Struct>,
    enums: Vec<Enum>,
    functions: Vec<Defun>,
}

impl TryFrom<&mut Parser> for File {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut structs = vec![];
        let mut enums = vec![];
        let mut functions = vec![];

        loop {
            let peek = value.nth(1);

            if peek.is_none() {
                break;
            }

            match peek {
                Some(Token::Keyword(Keywords::Defun)) => {
                    functions.push(error!(Defun::try_from(&mut *value), "File")?)
                }
                Some(Token::Keyword(Keywords::Struct)) => {
                    structs.push(error!(Struct::try_from(&mut *value), "File")?)
                }
                Some(Token::Keyword(Keywords::Enum)) => {
                    enums.push(error!(Enum::try_from(&mut *value), "File")?)
                }
                token => {
                    return Err(error!(
                        "Defun",
                        format!("Expected a Defun keyword or a typedef, got {token:#?}")
                    ))
                }
            }
        }

        Ok(File {
            structs,
            enums,
            functions,
        })
    }
}

impl ToString for File {
    fn to_string(&self) -> String {
        format!(
            "{}\n{}\n{}",
            self.structs.iter().fold(String::new(), |str, r#struct| {
                format!("{str}\n{}", r#struct.to_string())
            }),
            self.enums.iter().fold(String::new(), |str, r#enum| {
                format!("{str}\n{}", r#enum.to_string())
            }),
            self.functions.iter().fold(String::new(), |str, function| {
                format!("{str}\n{}", function.to_string())
            })
        )
    }
}
