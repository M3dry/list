use either::Either;

use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error, lambda::Lambda, r#if::If, r#let::Let, r#match::Match,
    r#type::{NamespacedType, self}, Parser, ParserError, ParserErrorStack,
};

// #[derive(Debug)]
// pub struct Call {
//     func: Either<Lambda, String>,
//     params: Vec<Exp>,
// }

// impl ToString for Call {
//     fn to_string(&self) -> String {
//         match &self.func {
//             Either::Left(lambda) => format!(
//                 "({})({})",
//                 lambda.to_string(),
//                 &self.params.iter().fold(String::new(), |str, param| {
//                     format!("{str}, {}", param.to_string())
//                 })[2..]
//             ),
//             Either::Right(name) => format!(
//                 "{name}({})",
//                 if !self.params.is_empty() {
//                     (&self.params.iter().fold(String::new(), |str, param| {
//                         format!("{str}, {}", param.to_string())
//                     })[2..])
//                         .to_string()
//                 } else {
//                     format!("")
//                 }
//             ),
//         }
//     }
// }

#[derive(Debug)]
pub enum TypeCreation {
    Simple(NamespacedType),
    Vars(NamespacedType, Vec<Exp>),
    Struct(NamespacedType, Vec<(String, Exp)>),
    Touple(Vec<Exp>),
}

impl TryFrom<&mut Parser> for TypeCreation {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut paren = false;
        let next = value
            .first()
            .ok_or(error!("TypeCreation", format!("Expected more tokens")))?;
        if next == &Token::ParenOpen {
            value.pop_front();
            paren = true;
        } else if next == &Token::AngleBracketOpen {
            value.pop_front();
            let mut exps = vec![];

            loop {
                let peek = value.first().ok_or(error!("TypeCreation", format!("Expected more tokens")))?;

                if peek == &Token::AngleBracketClose {
                    value.pop_front();
                    return Ok(TypeCreation::Touple(exps))
                }

                exps.push(error!(Exp::try_from(&mut *value), "TypeCreation")?)
            }
        }

        let name = error!(NamespacedType::try_from(&mut *value), "TypeCreation")?;

        let ret = match value.first() {
            Some(Token::CurlyOpen) => {
                value.pop_front();
                let mut fields = vec![];

                loop {
                    let peek = value
                        .first()
                        .ok_or(error!("TypeCreation", format!("Expected more tokens")))?;

                    if peek == &Token::CurlyClose {
                        value.pop_front();
                        break;
                    }

                    let next = value.pop_front_err("TypeCreation")?;
                    let name = if let Token::Identifier(iden) = next {
                        iden
                    } else {
                        return Err(error!(
                            "TypeCreation",
                            format!("Expected identifier, got {next:#?}")
                        ));
                    };

                    let next = value.pop_front_err("TypeCreation")?;
                    if next != Token::Keyword(Keywords::Arrow) {
                        return Err(error!(
                            "TypeCreation",
                            format!("Expected arrow keyword, got {next:#?}")
                        ));
                    }

                    let exp = error!(Exp::try_from(&mut *value), "TypeCreation")?;

                    fields.push((name, exp));
                }

                Ok(Self::Struct(name, fields))
            }
            Some(Token::ParenClose) => Ok(Self::Simple(name)),
            Some(_) if paren => {
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

                return Ok(Self::Vars(name, args));
            }
            _ => Ok(Self::Simple(name)),
        };

        if paren {
            let next = value.pop_front_err("TypeCreation")?;
            if next != Token::ParenClose {
                return Err(error!(
                    "TypeCreation",
                    format!("Expected parenClose, got {next:#?}")
                ));
            }
        }

        ret
    }
}

impl ToString for TypeCreation {
    fn to_string(&self) -> String {
        match self {
            TypeCreation::Simple(name) => format!("{}", name.to_string()),
            TypeCreation::Vars(name, args) => format!(
                "{}({})",
                name.to_string(),
                &args.iter().fold(String::new(), |str, arg| {
                    format!("{str}, {}", arg.to_string())
                })[2..]
            ),
            TypeCreation::Struct(name, fields) => {
                format!(
                    "{}{{{}}}",
                    name.to_string(),
                    &fields.iter().fold(String::new(), |str, field| {
                        format!("{str}, {}:{}", field.0, field.1.to_string())
                    })[2..]
                )
            },
            TypeCreation::Touple(exps) => format!("({})", &if exps.is_empty() {
                format!(", ")
            } else {
                exps.iter().fold(String::new(), |str, exp| {
                    format!("{str}, {}", exp.to_string())
                })
            }[2..])
        }
    }
}

#[derive(Debug)]
pub enum Exp {
    Lambda(Box<Lambda>),
    If(Box<If>),
    Match(Box<Match>),
    Let(Box<Let>),
    FuncCall(Box<Exp>, Vec<Exp>),
    Ref(Box<Exp>),
    MutRef(Box<Exp>),
    Deref(Box<Exp>),
    Negation(Box<Exp>),
    Infix(Box<Infix>),
    // TODO: Do(Box<Do>), 
    Variable(String),
    Literal(Literals),
    TypeCreation(TypeCreation),
}

impl TryFrom<&mut Parser> for Exp {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let peek = value.first().ok_or(error!("Exp", format!("Expected more tokens")))?;

        match peek {
            Token::Identifier(_) => {
                if matches!(value.nth(1), Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyOpen)) {
                    Ok(Exp::TypeCreation(error!(TypeCreation::try_from(&mut *value), "Exp")?))
                } else if let Token::Identifier(iden) = value.pop_front().unwrap() {
                    Ok(Exp::Variable(iden))
                } else {
                    unreachable!()
                }
            } 
            Token::Literal(_) => {
                if let Token::Literal(literal) = value.pop_front().unwrap() {
                    Ok(Exp::Literal(literal))
                } else {
                    unreachable!()
                }
            } 
            Token::Ref => {
                value.pop_front();
                println!("{value:#?}");
                if value.first() == Some(&Token::Keyword(Keywords::Mut)) {
                    value.pop_front();
                    Ok(Exp::MutRef(Box::new(error!(Exp::try_from(&mut *value), "Exp")?)))
                } else {
                    Ok(Exp::Ref(Box::new(error!(Exp::try_from(&mut *value), "Exp")?)))
                }
            },
            Token::Char('!') => {
                value.pop_front();

                Ok(Exp::Negation(Box::new(error!(Exp::try_from(&mut *value), "Exp")?)))
            }
            Token::Char('*') => {
                value.pop_front();

                Ok(Exp::Deref(Box::new(error!(Exp::try_from(&mut *value), "Exp")?)))
            }
            Token::AngleBracketOpen => {
                Ok(Exp::TypeCreation(error!(TypeCreation::try_from(&mut *value), "Exp")?))
            }
            Token::ParenOpen => {
                match value.nth(1).ok_or(error!("Exp", format!("Expected more tokens")))? {
                    Token::Keyword(keyword) => match keyword {
                        Keywords::If => Ok(Exp::If(Box::new(error!(If::try_from(&mut *value), "Exp")?))),
                        Keywords::Match => Ok(Exp::Match(Box::new(error!(Match::try_from(&mut *value), "Exp")?))),
                        Keywords::Let => Ok(Exp::Let(Box::new(error!(Let::try_from(&mut *value), "Exp")?))),
                        Keywords::Lambda => Ok(Exp::Lambda(Box::new(error!(Lambda::try_from(&mut *value), "Exp")?))),
                        _ => todo!()
                    }
                    Token::Identifier(_) => {
                        if matches!(value.nth(2), Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyClose)) {
                            Ok(Exp::TypeCreation(error!(TypeCreation::try_from(&mut *value), "Exp")?))
                        } else if value.nth(2) != Some(&Token::ParenClose) {
                            value.pop_front();
                            let func = if let Token::Identifier(iden) = value.pop_front_err("Exp")? {
                                iden
                            } else {
                                unreachable!()
                            };
                            let mut params = vec![];

                            loop {
                                let peek = value.first().ok_or(error!("Exp", format!("Expected more tokens")))?;

                                if peek == &Token::ParenClose {
                                    value.pop_front();
                                    break;
                                }

                                params.push(error!(Exp::try_from(&mut *value), "exp")?)
                            }

                            Ok(Exp::FuncCall(Box::new(Exp::Variable(func)), params))
                        } else {
                            todo!()
                        }
                    }
                    _ => {
                        value.pop_front();
                        let exp = error!(Exp::try_from(&mut *value), "Exp")?;
                        
                        let next = value.first().ok_or(error!("Exp", format!("Expected more tokens")))?;
                        if next == &Token::ParenClose {
                            value.pop_front();
                            Ok(exp)
                        } else {
                            let mut params = vec![];

                            loop {
                                let peek = value.first().ok_or(error!("Exp", format!("Expected more tokens")))?;

                                if peek == &Token::ParenClose {
                                    value.pop_front();
                                    break;
                                }

                                params.push(error!(Exp::try_from(&mut *value), "Exp")?)
                            }

                            Ok(Exp::FuncCall(Box::new(exp), params))
                        }
                    },
                }
            },
            token => {
                panic!("Not implemented {token:#?}")
            }
        }
    }
}

impl ToString for Exp {
    fn to_string(&self) -> String {
        todo!()
    }
}

// #[derive(Debug)]
// pub enum Exp {
//     Call(Box<Call>),
//     TypeCreation(Box<TypeCreation>),
//     If(Box<If>),
//     Match(Box<Match>),
//     Let(Box<Let>),
//     Lambda(Box<Lambda>),
//     Literal(Literals),
//     Identifier(String),
// }

// impl TryFrom<&mut Parser> for Exp {
//     type Error = ParserError;

//     fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
//         let peek = value
//             .first()
//             .ok_or(error!("Exp", format!("Expected more tokens")))?;

//         match peek {
//             Token::ParenOpen => {
//                 match value
//                     .nth(1)
//                     .ok_or(error!("Exp paren", format!("Expected more tokens"),))?
//                 {
//                     Token::Keyword(Keywords::If) => {
//                         Ok(Exp::If(Box::new(error!(If::try_from(&mut *value), "Exp")?)))
//                     }
//                     Token::Keyword(Keywords::Match) => Ok(Exp::Match(Box::new(error!(
//                         Match::try_from(&mut *value),
//                         "Exp"
//                     )?))),
//                     Token::Keyword(Keywords::Lambda) => Ok(Exp::Lambda(Box::new(error!(
//                         Lambda::try_from(&mut *value),
//                         "Exp"
//                     )?))),
//                     Token::Keyword(Keywords::Let) => Ok(Exp::Let(Box::new(error!(
//                         Let::try_from(&mut *value),
//                         "Exp"
//                     )?))),
//                     Token::Literal(_) => {
//                         value.pop_front();
//                         Ok(Exp::Literal(
//                             if let Token::Literal(literal) = value.pop_front().unwrap() {
//                                 literal
//                             } else {
//                                 panic!("wtf")
//                             },
//                         ))
//                     }
//                     Token::Identifier(_) => match value.nth(2) {
//                         Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyOpen) => Ok(Exp::TypeCreation(Box::new(
//                             error!(TypeCreation::try_from(&mut *value), "Exp")?,
//                         ))),
//                         _ => {
//                             value.pop_front();
//                             let func = if let Token::Identifier(iden) = value.pop_front().unwrap() {
//                                 iden
//                             } else {
//                                 panic!("wtf")
//                             };
//                             let mut params = vec![];

//                             loop {
//                                 let peek = value.first().ok_or(error!(
//                                     "Exp call/iden",
//                                     format!("Expected more tokens"),
//                                 ))?;

//                                 if *peek == Token::ParenClose {
//                                     value.pop_front();
//                                     break;
//                                 } else {
//                                     params.push(error!(Exp::try_from(&mut *value), "Exp")?)
//                                 }
//                             }

//                             Ok(Exp::Call(Box::new(Call {
//                                 func: Either::Right(func),
//                                 params,
//                             })))
//                         }
//                     },
//                     Token::AngleBracketOpen | Token::AngleBracketClose => {
//                         value.pop_front();

//                         let angle = match value.pop_front().unwrap() {
//                             Token::AngleBracketOpen => "<",
//                             Token::AngleBracketClose => ">",
//                             _ => panic!("wtf"),
//                         };

//                         let mut params = vec![];

//                         loop {
//                             let peek = value
//                                 .first()
//                                 .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

//                             if *peek == Token::ParenClose {
//                                 value.pop_front();
//                                 break;
//                             } else {
//                                 params.push(error!(Exp::try_from(&mut *value), "Exp")?)
//                             }
//                         }

//                         Ok(Exp::Call(Box::new(Call {
//                             func: Either::Right(String::from(angle)),
//                             params,
//                         })))
//                     }
//                     Token::ParenOpen => {
//                         value.pop_front();
//                         let lambda = error!(Lambda::try_from(&mut *value), "Exp call/lambda")?;
//                         let mut params = vec![];

//                         loop {
//                             let peek = value
//                                 .first()
//                                 .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

//                             if *peek == Token::ParenClose {
//                                 value.pop_front();
//                                 break;
//                             } else {
//                                 params.push(error!(Exp::try_from(&mut *value), "Exp")?)
//                             }
//                         }

//                         Ok(Exp::Call(Box::new(Call {
//                             func: Either::Left(lambda),
//                             params,
//                         })))
//                     }
//                     token => return Err(error!("Exp", format!("got {token:#?}"))),
//                 }
//             }
//             Token::Identifier(_) => match value.nth(1) {
//                 Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyOpen) => {
//                     Ok(Exp::TypeCreation(Box::new(error!(
//                         TypeCreation::try_from(&mut *value),
//                         "Exp"
//                     )?)))
//                 }
//                 _ => {
//                     let iden = value.pop_front().unwrap();
//                     if let Token::Identifier(iden) = iden {
//                         Ok(Exp::Identifier(iden))
//                     } else {
//                         panic!("how")
//                     }
//                 }
//             },
//             Token::AngleBracketOpen | Token::AngleBracketClose => {
//                 let angle = match value.pop_front().unwrap() {
//                     Token::AngleBracketOpen => "<",
//                     _ => ">",
//                 };
//                 let mut params = vec![];

//                 loop {
//                     let peek = value
//                         .first()
//                         .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

//                     if *peek == Token::ParenClose {
//                         value.pop_front();
//                         break;
//                     } else {
//                         params.push(error!(Exp::try_from(&mut *value), "Exp")?)
//                     }
//                 }

//                 Ok(Exp::Call(Box::new(Call {
//                     func: Either::Right(String::from(angle)),
//                     params,
//                 })))
//             }
//             Token::Literal(_) => {
//                 let literal = value.pop_front().unwrap();
//                 if let Token::Literal(literal) = literal {
//                     Ok(Exp::Literal(literal))
//                 } else {
//                     panic!("how")
//                 }
//             }
//             _ => Err(error!(
//                 "Exp",
//                 format!("Expected exp, got {:#?}", value.pop_front().unwrap()),
//             )),
//         }
//     }
// }

// impl ToString for Exp {
//     fn to_string(&self) -> String {
//         match self {
//             Exp::Call(call) => call.to_string(),
//             Exp::TypeCreation(creation) => creation.to_string(),
//             Exp::If(r#if) => r#if.to_string(),
//             Exp::Match(r#match) => r#match.to_string(),
//             Exp::Let(r#let) => r#let.to_string(),
//             Exp::Lambda(lambda) => lambda.to_string(),
//             Exp::Literal(literal) => literal.to_string(),
//             Exp::Identifier(name) => name.to_string(),
//         }
//     }
// }

#[derive(Debug)]
pub enum Infix {
    Add(Exp, Exp),
    Subtract(Exp, Exp),
    Multiply(Exp, Exp),
    Divide(Exp, Exp),
    Equality(Exp, Exp),
    NotEquality(Exp, Exp),
    And(Exp, Exp),
    Or(Exp, Exp),
    Less(Exp, Exp),
    LessEq(Exp, Exp),
    Greater(Exp, Exp),
    GreaterEq(Exp, Exp),
    Xor(Exp, Exp),
    BitwiseAnd(Exp, Exp),
    BitwiseOr(Exp, Exp),
}

impl Infix {
    fn is_infix(parser: &Parser) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::snapshot;

    snapshot!(test_calling, Exp::try_from, "calling.lt");
    snapshot!(test_calling_rust, Exp::try_from, "calling.lt", rust);
}
