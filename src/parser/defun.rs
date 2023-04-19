use crate::tokenizer::{Keywords, Token};

use super::{
    args::ArgsTyped, error, exp::Exp, r#type::Type, Parser, ParserError, ParserErrorStack, Error,
};

#[derive(Debug)]
pub enum Defun {
    Function {
        scope: Scope,
        name: String,
        args: ArgsTyped,
        return_type: Type,
        body: Exp,
    },
    Header {
        scope: Scope,
        name: String,
        args: ArgsTyped,
        return_type: Type,
    },
}

impl TryFrom<&mut Parser> for Defun {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Defun", value.pop_front(), [Token::ParenOpen])?;
        let scope = Scope::try_from(&mut *value).unwrap();
        let _ = error!("Defun", value.pop_front(), [Token::Keyword(Keywords::Defun)])?;
        let name = if let Token::Identifier(iden) = error!("Defun", value.pop_front(), [Token::Identifier(_)])? {
            iden
        } else {
            unreachable!()
        };
        let args = error!(ArgsTyped::try_from(&mut *value), "Defun")?;
        let _ = error!("Defun", value.pop_front(), [Token::Keyword(Keywords::LeftArrow)])?;
        let return_type = error!(Type::try_from(&mut *value), "Defun")?;

        if value.first() == Some(&Token::ParenClose) {
            value.pop_front();
            return Ok(Self::Header {
                scope,
                name,
                args,
                return_type,
            });
        }

        let body = error!(Exp::try_from(&mut *value), "Defun")?;
        let _ = error!("Defun", value.pop_front(), [Token::ParenClose])?;

        Ok(Defun::Function {
            scope,
            name,
            args,
            return_type,
            body,
        })
    }
}

impl ToString for Defun {
    fn to_string(&self) -> String {
        match self {
            Self::Function {
                scope,
                name,
                args,
                return_type,
                body,
            } => {
                format!(
                    "{}fn {}{} -> {} {{{}}}",
                    scope.to_string(),
                    name,
                    args.to_string(),
                    return_type.to_string(),
                    body.to_string()
                )
            }
            Self::Header {
                scope,
                name,
                args,
                return_type,
            } => {
                format!(
                    "{}fn {}{} -> {};",
                    scope.to_string(),
                    name,
                    args.to_string(),
                    return_type.to_string(),
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum Scope {
    File,
    Crate,
    Full,
}

impl TryFrom<&mut Parser> for Scope {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        match value.first_err("Scope")? {
            Token::Identifier(iden) => match &iden[..] {
                "crate" => {
                    value.pop_front();
                    Ok(Scope::Crate)
                }
                "pub" => {
                    value.pop_front();
                    Ok(Scope::Full)
                }
                iden => Err(error!("Scope iden", Error::Other(format!("Expected pub or crate, got {iden:#?}")))),
            },
            _ => Ok(Scope::File),
        }
    }
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        match self {
            Scope::File => "",
            Scope::Crate => "pub(crate) ",
            Scope::Full => "pub ",
        }
        .to_string()
    }
}
