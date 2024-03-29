use crate::tokenizer::{Keywords, Token};
use super::{error, Parser, ParserError, ParserErrorStack};

/// # language example
/// (use std->collections->VecDeque)
/// (use std->collections->(VecDeque HashMap HashSet))
/// (use std->(collections->(VecDeque HashMap HashSet) env->args))
#[derive(Debug)]
pub struct Use(UsePath);

impl TryFrom<&mut Parser> for Use {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let _ = error!("Use", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Use", value.pop_front(), [Token::Keyword(Keywords::Use)])?;
        let ret = error!(UsePath::try_from(&mut *value), "Use")?;
        let _ = error!("Use", value.pop_front(), [Token::ParenClose])?;

        Ok(Self(ret))
    }
}

impl ToString for Use {
    fn to_string(&self) -> String {
        format!("use {};", self.0.to_string())
    }
}

#[derive(Debug)]
pub enum UsePath {
    Path(String, Box<UsePath>),
    Name(String),
    Multiple(Vec<UsePath>),
    All,
}

impl TryFrom<&mut Parser> for UsePath {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(
            match error!("UsePath", value.pop_front(), [Token::ParenOpen, Token::Identifier(_), Token::Keyword(Keywords::Deref), Token::Char('*')])? {
                Token::Char('*') | Token::Keyword(Keywords::Deref) => Self::All,
                Token::Identifier(name)
                    if value.first() == Some(&Token::Keyword(Keywords::LeftArrow)) =>
                {
                    value.pop_front();
                    Self::Path(
                        name,
                        Box::new(error!(Self::try_from(&mut *value), "UsePath")?),
                    )
                }
                Token::Identifier(name) => Self::Name(name),
                Token::ParenOpen => {
                    let mut multiple = vec![];

                    loop {
                        let peek = value
                            .first_err("UsePath")?;

                        if peek == &Token::ParenClose {
                            value.pop_front();
                            break;
                        }

                        multiple.push(error!(Self::try_from(&mut *value), "UsePath")?)
                    }

                    Self::Multiple(multiple)
                }
                _ => unreachable!(),
            },
        )
    }
}

impl ToString for UsePath {
    fn to_string(&self) -> String {
        match self {
            UsePath::Path(name, path) => format!("{name}::{}", path.to_string()),
            UsePath::Name(name) => format!("{name}"),
            UsePath::Multiple(multiple) => format!(
                "{{{}}}",
                &if multiple.is_empty() {
                    format!(", ")
                } else {
                    multiple.iter().fold(String::new(), |str, path| {
                        format!("{str}, {}", path.to_string())
                    })
                }[2..]
            ),
            UsePath::All => format!("*"),
        }
    }
}
