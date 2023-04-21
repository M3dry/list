use crate::tokenizer::{Keywords, Token};

use super::{
    attribute::Attribute, defun::Defun, error, module::Mod, r#enum::Enum, r#impl::Impl,
    r#struct::Struct, r#trait::Trait, r#type::TypeAlias, r#use::Use, Parser, ParserError,
    ParserErrorStack,
};

#[derive(Debug)]
pub struct File(Vec<FileOps>);

impl TryFrom<&mut Parser> for File {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut file_ops = vec![];
        while !value.tokens.is_empty() {
            file_ops.push(error!(FileOps::try_from(&mut *value), "File")?);
        }

        Ok(Self(file_ops))
    }
}

impl ToString for File {
    fn to_string(&self) -> String {
        format!(
            "{}",
            &if self.0.is_empty() {
                format!("\n")
            } else {
                self.0.iter().fold(String::new(), |str, file_op| {
                    format!("{str}\n{}", file_op.to_string())
                })
            }[1..]
        )
    }
}

#[derive(Debug)]
pub enum FileOps {
    Use(Use),
    Struct(Struct),
    Enum(Enum),
    Function(Defun),
    Attribute(Attribute),
    Impl(Impl),
    Trait(Trait),
    TypeAlias(TypeAlias),
    Mod(Mod),
}

impl TryFrom<&mut Parser> for FileOps {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        if value.first() == Some(&Token::Char('#')) {
            return Ok(Self::Attribute(error!(
                Attribute::try_from(&mut *value),
                "FileOps"
            )?));
        }

        Ok(
            match error!(
                "FileOps",
                value.nth(1),
                [
                    Token::Keyword(
                        Keywords::Use
                            | Keywords::Enum
                            | Keywords::Struct
                            | Keywords::Defun
                            | Keywords::Mod
                            | Keywords::Impl
                            | Keywords::Trait
                            | Keywords::Type
                    ),
                    Token::Identifier(_)
                ]
            )? {
                Token::Keyword(Keywords::Use) => {
                    Self::Use(error!(Use::try_from(&mut *value), "FileOps")?)
                }
                Token::Keyword(Keywords::Enum) => {
                    Self::Enum(error!(Enum::try_from(&mut *value), "FileOps")?)
                }
                Token::Keyword(Keywords::Struct) => {
                    Self::Struct(error!(Struct::try_from(&mut *value), "FileOps")?)
                }
                Token::Keyword(Keywords::Defun) => {
                    Self::Function(error!(Defun::try_from(&mut *value), "FileOps")?)
                }
                Token::Keyword(Keywords::Mod) => {
                    Self::Mod(error!(Mod::try_from(&mut *value), "FileOps")?)
                }
                Token::Identifier(iden) if &iden[..] == "pub" || &iden[..] == "crate" => {
                    match error!("FileOps", value.nth(2), [Token::Keyword(Keywords::Defun | Keywords::Mod)])?
                    {
                        Token::Keyword(Keywords::Defun) => {
                            Self::Function(error!(Defun::try_from(&mut *value), "FileOps")?)
                        }
                        Token::Keyword(Keywords::Mod) => {
                            Self::Mod(error!(Mod::try_from(&mut *value), "FileOps")?)
                        }
                        _ => unreachable!(),
                    }
                }
                Token::Keyword(Keywords::Impl) => {
                    Self::Impl(error!(Impl::try_from(&mut *value), "FileOps")?)
                }
                Token::Keyword(Keywords::Trait) => {
                    Self::Trait(error!(Trait::try_from(&mut *value), "FileOps")?)
                }
                Token::Keyword(Keywords::Type) => {
                    Self::TypeAlias(error!(TypeAlias::try_from(&mut *value), "FileOps")?)
                }
                _ => unreachable!(),
            },
        )
    }
}

impl ToString for FileOps {
    fn to_string(&self) -> String {
        match self {
            Self::Use(r#use) => r#use.to_string(),
            Self::Struct(r#struct) => r#struct.to_string(),
            Self::Enum(r#enum) => r#enum.to_string(),
            Self::Function(function) => function.to_string(),
            Self::Attribute(attr) => attr.to_string(),
            Self::Impl(r#impl) => r#impl.to_string(),
            Self::Trait(r#trait) => r#trait.to_string(),
            Self::TypeAlias(type_alias) => type_alias.to_string(),
            Self::Mod(module) => module.to_string(),
        }
    }
}
