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
        let _ = error!("Attribute", value.pop_front(), [Token::Char('#')])?;
        let next = error!(
            "Attribute",
            value.pop_front(),
            [Token::Char('!'), Token::BracketOpen]
        )?;
        let inner = if next == Token::Char('!') {
            let _ = error!("Attribute", value.pop_front(), [Token::BracketOpen])?;
            true
        } else {
            false
        };

        let attr_ops = error!(AttributeOps::try_from(&mut *value), "Attribute")?;
        let _ = error!("Attribute", value.pop_front(), [Token::BracketClose])?;

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
        let next = error!(
            "AttributeOps",
            value.pop_front(),
            [Token::ParenOpen, Token::Identifier(_)]
        )?;
        Ok(match next {
            Token::ParenOpen => match error!(
                "AttributeOps",
                value.pop_front(),
                [Token::Identifier(_), Token::Char('=')]
            )? {
                Token::Identifier(iden) => {
                    let mut args = vec![];

                    loop {
                        let peek = value.first_err("AttributeOps")?;

                        if peek == &Token::ParenClose {
                            value.pop_front();
                            break Self::Command(iden, args);
                        }

                        args.push(error!(AttributeOps::try_from(&mut *value), "AttributeOps")?)
                    }
                }
                Token::Char('=') => {
                    let iden = error!("AttributeOps", value);
                    let exp = error!(Exp::try_from(&mut *value), "AttributeOps")?;
                    let _ = error!("AttributeOps", value.pop_front(), [Token::ParenClose])?;

                    Self::Assignment(iden, exp)
                }
                _ => unreachable!(),
            },
            Token::Identifier(iden) => Self::Identifier(iden),
            _ => unreachable!(),
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
