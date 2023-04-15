use crate::tokenizer::Token;

use super::{error, exp::Exp, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub enum Attribute {
    Inner(AttributeOps),
    Outer(AttributeOps),
}

impl TryFrom<&mut Parser> for Attribute {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Attribute")?;
        if next != Token::Char('#') {
            return Err(error!("Attribute", format!("Expected #, got {next:#?}")));
        }
        let next = value.pop_front_err("Attribute")?;
        let (inner, next) = if next == Token::Char('!') {
            (true, value.pop_front_err("Attribute")?)
        } else {
            (false, next)
        };
        if next != Token::BracketOpen {
            return Err(error!(
                "Attribute",
                format!("Expected bracketOpen, got {next:#?}")
            ));
        };

        let attr_ops = error!(AttributeOps::try_from(&mut *value), "Attribute")?;

        let next = value.pop_front_err("Attribute")?;
        if next != Token::BracketClose {
            return Err(error!(
                "Attribute",
                format!("Expected bracketClose, got {next:#?}")
            ));
        }

        Ok(if inner {
            Self::Inner(attr_ops)
        } else {
            Self::Outer(attr_ops)
        })
    }
}

impl ToString for Attribute {
    fn to_string(&self) -> String {
        match self {
            Attribute::Inner(attr_ops) => format!("#![{}]", attr_ops.to_string()),
            Attribute::Outer(attr_ops) => format!("#[{}]", attr_ops.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum AttributeOps {
    Command(String, Vec<AttributeOps>),
    Assignment(String, Exp),
    Identifier(String),
}

impl TryFrom<&mut Parser> for AttributeOps {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("AttributeOps")?;
        Ok(match next {
            Token::ParenOpen => match value.pop_front_err("AttributeOps")? {
                Token::Identifier(iden) => {
                    let mut args = vec![];

                    loop {
                        let peek = value
                            .first()
                            .ok_or(error!("AttributeOps", format!("Expected more tokens")))?;

                        if peek == &Token::ParenClose {
                            value.pop_front();
                            break Self::Command(iden, args);
                        }

                        args.push(error!(AttributeOps::try_from(&mut *value), "AttributeOps")?)
                    }
                }
                Token::Char('=') => {
                    let iden = match value.pop_front_err("AttributeOps")? {
                        Token::Identifier(iden) => iden,
                        token => {
                            return Err(error!(
                                "AttributeOps",
                                format!("Expected iden, got {token:#?}")
                            ))
                        }
                    };
                    let exp = error!(Exp::try_from(&mut *value), "AttributeOps")?;
                    let next = value.pop_front_err("AttributeOps")?;
                    if next != Token::ParenClose {
                        return Err(error!("AttributeOps", format!("Expected parenClose, got {next:#?}")))
                    }

                    Self::Assignment(iden, exp)
                }
                token => {
                    return Err(error!(
                        "AttributeOps",
                        format!("Expected iden or =, got {token:#?}")
                    ))
                }
            },
            Token::Identifier(iden) => Self::Identifier(iden),
            token => {
                return Err(error!(
                    "AttributeOps",
                    format!("Expected iden or parenOpen, got {token:#?}")
                ))
            }
        })
    }
}

impl ToString for AttributeOps {
    fn to_string(&self) -> String {
        match self {
            Self::Command(name, args) => format!(
                "{name}({})",
                &if args.is_empty() {
                    format!(", ")
                } else {
                    args.iter().fold(String::new(), |str, arg| {
                        format!("{str}, {}", arg.to_string())
                    })
                }[2..]
            ),
            Self::Assignment(var, exp) => format!("{var} = {}", exp.to_string()),
            Self::Identifier(var) => var.to_string(),
        }
    }
}
