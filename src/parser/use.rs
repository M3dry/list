use crate::tokenizer::{Keywords, Token};
use std::env::*;
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
        let next = value.pop_front_err("Use")?;
        if next != Token::ParenOpen {
            return Err(error!("Use", format!("Expected parenOpen, got {next:#?}")));
        }

        let next = value.pop_front_err("Use")?;
        if next != Token::Keyword(Keywords::Use) {
            return Err(error!(
                "Use",
                format!("Expected Use keyword, got {next:#?}")
            ));
        }

        let ret = error!(UsePath::try_from(&mut *value), "Use")?;

        let next = value.pop_front_err("Use")?;
        if next != Token::ParenClose {
            return Err(error!("Use", format!("Expected parenClose, got {next:#?}")));
        }

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
}

impl TryFrom<&mut Parser> for UsePath {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(
            match value.pop_front_err("UsePath")? {
                Token::Identifier(name)
                    if value.first() == Some(&Token::Keyword(Keywords::Arrow)) =>
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
                            .first()
                            .ok_or(error!("UsePath", format!("Expected more tokens")))?;

                        if peek == &Token::ParenClose {
                            value.pop_front();
                            break;
                        }

                        multiple.push(error!(Self::try_from(&mut *value), "UsePath")?)
                    }

                    Self::Multiple(multiple)
                }
                token => {
                    return Err(error!(
                        "UsePath",
                        format!("Expected Identifier or ParenOpen, got {token:#?}")
                    ))
                }
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_use, Use::try_from, "use.lt");
    snapshot!(test_use_rust, Use::try_from, "use.lt", rust);
}
