use either::Either;

use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error, lambda::Lambda, r#if::If, r#let::Let, r#match::Match, r#type::NamespacedType, Parser,
    ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub struct Call {
    func: Either<Lambda, String>,
    params: Vec<Exp>,
}

impl ToString for Call {
    fn to_string(&self) -> String {
        match &self.func {
            Either::Left(lambda) => format!(
                "({})({})",
                lambda.to_string(),
                &self.params.iter().fold(String::new(), |str, param| {
                    format!("{str}, {}", param.to_string())
                })[2..]
            ),
            Either::Right(name) => format!(
                "{name}({})",
                if !self.params.is_empty() {
                    (&self.params.iter().fold(String::new(), |str, param| {
                        format!("{str}, {}", param.to_string())
                    })[2..])
                        .to_string()
                } else {
                    format!("")
                }
            ),
        }
    }
}

#[derive(Debug)]
pub struct TypeCreation {
    name: NamespacedType,
    args: Vec<Exp>,
}

impl TryFrom<&mut Parser> for TypeCreation {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        if value.first() == Some(&Token::ParenOpen) {
            let next = value.pop_front_err("TypeCreation", "Expected more Tokens")?;
            if next != Token::ParenOpen {
                return Err(error!(
                    "TypeCreation",
                    format!("Expected parenOpen, got {next:#?}")
                ));
            }

            let name = error!(NamespacedType::try_from(&mut *value), "TypeCreation")?;
            let mut args = vec![];

            loop {
                let peek = value
                    .first()
                    .ok_or(error!("TypeCreation", format!("Expected more tokens")))?;

                if peek == &Token::ParenClose {
                    value.pop_front();
                    break;
                }

                args.push(error!(Exp::try_from(&mut *value), "TypeCreation")?)
            }

            Ok(TypeCreation { name, args })
        } else {
            Ok(TypeCreation {
                name: error!(NamespacedType::try_from(&mut *value), "TypeCreation")?,
                args: vec![],
            })
        }
    }
}

impl ToString for TypeCreation {
    fn to_string(&self) -> String {
        format!(
            "{}{}",
            self.name.to_string(),
            if !self.args.is_empty() {
                format!(
                    "({})",
                    &self.args.iter().fold(String::new(), |str, arg| {
                        format!("{str}, {}", arg.to_string())
                    })[2..]
                )
            } else {
                format!("")
            }
        )
    }
}

#[derive(Debug)]
pub enum Exp {
    Call(Box<Call>),
    TypeCreation(Box<TypeCreation>),
    If(Box<If>),
    Match(Box<Match>),
    Let(Box<Let>),
    Lambda(Box<Lambda>),
    Literal(Literals),
    Identifier(String),
}

impl TryFrom<&mut Parser> for Exp {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let peek = value
            .first()
            .ok_or(error!("Exp", format!("Expected more tokens")))?;

        match peek {
            Token::ParenOpen => {
                match value
                    .nth(1)
                    .ok_or(error!("Exp paren", format!("Expected more tokens"),))?
                {
                    Token::Keyword(Keywords::If) => {
                        Ok(Exp::If(Box::new(error!(If::try_from(&mut *value), "Exp")?)))
                    }
                    Token::Keyword(Keywords::Match) => Ok(Exp::Match(Box::new(error!(
                        Match::try_from(&mut *value),
                        "Exp"
                    )?))),
                    Token::Keyword(Keywords::Lambda) => Ok(Exp::Lambda(Box::new(error!(
                        Lambda::try_from(&mut *value),
                        "Exp"
                    )?))),
                    Token::Keyword(Keywords::Let) => Ok(Exp::Let(Box::new(error!(
                        Let::try_from(&mut *value),
                        "Exp"
                    )?))),
                    Token::Literal(_) => {
                        value.pop_front();
                        Ok(Exp::Literal(
                            if let Token::Literal(literal) = value.pop_front().unwrap() {
                                literal
                            } else {
                                panic!("wtf")
                            },
                        ))
                    }
                    Token::Identifier(_) => {
                        if value.nth(2) == Some(&Token::Keyword(Keywords::Arrow)) {
                            Ok(Exp::TypeCreation(Box::new(error!(
                                TypeCreation::try_from(&mut *value),
                                "Exp"
                            )?)))
                        } else {
                            value.pop_front();
                            let func = if let Token::Identifier(iden) = value.pop_front().unwrap() {
                                iden
                            } else {
                                panic!("wtf")
                            };
                            let mut params = vec![];

                            loop {
                                let peek = value.first().ok_or(error!(
                                    "Exp call/iden",
                                    format!("Expected more tokens"),
                                ))?;

                                if *peek == Token::ParenClose {
                                    value.pop_front();
                                    break;
                                } else {
                                    params.push(error!(Exp::try_from(&mut *value), "Exp")?)
                                }
                            }

                            Ok(Exp::Call(Box::new(Call {
                                func: Either::Right(func),
                                params,
                            })))
                        }
                    }
                    Token::AngleBracketOpen | Token::AngleBracketClose => {
                        value.pop_front();

                        let angle = match value.pop_front().unwrap() {
                            Token::AngleBracketOpen => "<",
                            Token::AngleBracketClose => ">",
                            _ => panic!("wtf"),
                        };

                        let mut params = vec![];

                        loop {
                            let peek = value
                                .first()
                                .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

                            if *peek == Token::ParenClose {
                                value.pop_front();
                                break;
                            } else {
                                params.push(error!(Exp::try_from(&mut *value), "Exp")?)
                            }
                        }

                        Ok(Exp::Call(Box::new(Call {
                            func: Either::Right(String::from(angle)),
                            params,
                        })))
                    }
                    Token::ParenOpen => {
                        value.pop_front();
                        let lambda = error!(Lambda::try_from(&mut *value), "Exp call/lambda")?;
                        let mut params = vec![];

                        loop {
                            let peek = value
                                .first()
                                .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

                            if *peek == Token::ParenClose {
                                value.pop_front();
                                break;
                            } else {
                                params.push(error!(Exp::try_from(&mut *value), "Exp")?)
                            }
                        }

                        Ok(Exp::Call(Box::new(Call {
                            func: Either::Left(lambda),
                            params,
                        })))
                    }
                    token => return Err(error!("Exp", format!("got {token:#?}"))),
                }
            }
            Token::Identifier(_) => {
                let iden = value.pop_front().unwrap();
                if let Token::Identifier(iden) = iden {
                    Ok(Exp::Identifier(iden))
                } else {
                    panic!("how")
                }
            }
            Token::AngleBracketOpen | Token::AngleBracketClose => {
                let angle = match value.pop_front().unwrap() {
                    Token::AngleBracketOpen => "<",
                    _ => ">",
                };
                let mut params = vec![];

                loop {
                    let peek = value
                        .first()
                        .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

                    if *peek == Token::ParenClose {
                        value.pop_front();
                        break;
                    } else {
                        params.push(error!(Exp::try_from(&mut *value), "Exp")?)
                    }
                }

                Ok(Exp::Call(Box::new(Call {
                    func: Either::Right(String::from(angle)),
                    params,
                })))
            }
            Token::Literal(_) => {
                let literal = value.pop_front().unwrap();
                if let Token::Literal(literal) = literal {
                    Ok(Exp::Literal(literal))
                } else {
                    panic!("how")
                }
            }
            _ => Err(error!(
                "Exp",
                format!("Expected exp, got {:#?}", value.pop_front().unwrap()),
            )),
        }
    }
}

impl ToString for Exp {
    fn to_string(&self) -> String {
        match self {
            Exp::Call(call) => call.to_string(),
            Exp::TypeCreation(creation) => creation.to_string(),
            Exp::If(r#if) => r#if.to_string(),
            Exp::Match(r#match) => r#match.to_string(),
            Exp::Let(r#let) => r#let.to_string(),
            Exp::Lambda(lambda) => lambda.to_string(),
            Exp::Literal(literal) => literal.to_string(),
            Exp::Identifier(name) => name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_calling, Exp::try_from, "calling.lt");
    snapshot!(test_calling_rust, Exp::try_from, "calling.lt", rust);
}
