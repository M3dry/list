use crate::tokenizer::{Keywords, Token};

use super::{error, exp::Exp, Parser, ParserError, ParserErrorStack};

#[derive(Debug)]
pub struct Let {
    vars: Vec<(String, Exp)>,
    body: Exp,
}

impl TryFrom<&mut Parser> for Let {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.pop_front_err("Let", "Expected more tokens")?;
        if next != Token::ParenOpen {
            return Err(error!("Let", format!("Expected ParenOpen, got {next:#?}"),));
        }

        let next = value.pop_front_err("Let", "Expected more tokens")?;
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
                let next = value.pop_front_err("Let vars", "Expected more tokens")?;
                if next == Token::ParenClose {
                    break;
                }
                if next != Token::ParenOpen {
                    return Err(error!(
                        "Let vars",
                        format!("Expected ParenOpen, got {next:#?}"),
                    ));
                }

                let next = value.pop_front_err("Let vars", "Expected more tokens")?;
                let iden = if let Token::Identifier(iden) = next {
                    iden
                } else {
                    return Err(error!(
                        "Let vars/name",
                        format!("Expected identifier, got {next:#?}"),
                    ));
                };

                let exp = Exp::try_from(&mut *value)?;

                let next = value.pop_front_err("Let vars", "Expected more tokens")?;
                if next != Token::ParenClose {
                    return Err(error!(
                        "Let vars/close",
                        format!("Expected ParenClose, got {next:#?}"),
                    ));
                }

                vars.push((iden, exp));
            }
        }

        Ok(Let {
            vars,
            body: Exp::try_from(&mut *value)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_let, Let::try_from, "let.lt");
    snapshot!(test_let_rust, Let::try_from, "let.lt", rust);
}