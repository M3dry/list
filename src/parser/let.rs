use crate::tokenizer::{Keywords, Token};

use super::{error, exp::Exp, Parser, ParserError, ParserErrorStack};

/// (let ((x 10)
///       (y (+ 4 10)))
///     (* x y))
#[derive(Debug)]
pub struct Let {
    vars: Vec<(String, Exp)>,
    body: Exp,
}

impl TryFrom<&mut Parser> for Let {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Let")?;
        if next != Token::ParenOpen {
            return Err(error!("Let", format!("Expected ParenOpen, got {next:#?}"),));
        }

        let next = value.pop_front_err("Let")?;
        if next != Token::Keyword(Keywords::Let) {
            return Err(error!(
                "Let keyword",
                format!("Expected Let keyword, got {next:#?}"),
            ));
        }

        let mut vars = vec![];

        if let Token::Identifier(_) = value
            .nth(1)
            .ok_or(error!("Let var", format!("Expected more tokens")))?
        {
            value.pop_front();
            let iden = if let Token::Identifier(iden) = value.pop_front().unwrap() {
                iden
            } else {
                panic!("this shouldn't happen")
            };

            vars.push((iden, Exp::try_from(&mut *value)?));
            value.pop_front();
        } else if value.pop_front().unwrap() == Token::ParenOpen {
            loop {
                let next = value.pop_front_err("Let vars")?;
                if next == Token::ParenClose {
                    break;
                }
                if next != Token::ParenOpen {
                    return Err(error!(
                        "Let vars",
                        format!("Expected ParenOpen, got {next:#?}"),
                    ));
                }

                let next = value.pop_front_err("Let vars")?;
                let iden = if let Token::Identifier(iden) = next {
                    iden
                } else {
                    return Err(error!(
                        "Let vars/name",
                        format!("Expected identifier, got {next:#?}"),
                    ));
                };

                let exp = Exp::try_from(&mut *value)?;

                let next = value.pop_front_err("Let vars")?;
                if next != Token::ParenClose {
                    return Err(error!(
                        "Let vars/close",
                        format!("Expected ParenClose, got {next:#?}"),
                    ));
                }

                vars.push((iden, exp));
            }
        }

        let body = error!(Exp::try_from(&mut *value), "Let")?;
        let next = value.pop_front_err("Let")?;
        if next != Token::ParenClose {
            return Err(error!("Let", format!("Expected parenClose, got {next:#?}")))
        }

        Ok(Let {
            vars,
            body,
        })
    }
}

impl ToString for Let {
    fn to_string(&self) -> String {
        format!(
            "{{{}{}}}",
            self.vars.iter().fold(String::new(), |str, (name, exp)| {
                format!("{str}let {name} = {};\n", exp.to_string())
            }),
            self.body.to_string(),
        )
    }
}
