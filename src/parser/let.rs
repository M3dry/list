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
        let _ = error!("Let", value.pop_front(), [Token::ParenOpen])?;
        let _ = error!("Let", value.pop_front(), [Token::Keyword(Keywords::Let)])?;
        let mut vars = vec![];
        let _ = error!("Let", value.pop_front(), [Token::ParenOpen])?;

        match error!("Let", value.pop_front(), [Token::Identifier(_), Token::ParenOpen])? {
            Token::Identifier(iden) => {
                vars.push((iden, Exp::try_from(&mut *value)?));
                let _ = error!("Let", value.pop_front(), [Token::ParenClose])?;
            }
            Token::ParenOpen => {
                value.tokens.push_front(Token::ParenOpen);

                loop {
                    if error!("Let", value.pop_front(), [Token::ParenClose, Token::ParenOpen])? == Token::ParenClose {
                        break;
                    }

                    let iden = if let Token::Identifier(iden) = error!("Let", value.pop_front(), [Token::Identifier(_)])? {
                        iden
                    } else {
                        unreachable!()
                    };

                    let exp = Exp::try_from(&mut *value)?;
                    let _ = error!("Let vars", value.pop_front(), [Token::ParenClose])?;

                    vars.push((iden, exp));
                }
            }
            _ => unreachable!(),
        }

        let body = error!(Exp::try_from(&mut *value), "Let")?;
        let _ = error!("Let", value.pop_front(), [Token::ParenClose])?;

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
