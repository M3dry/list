use crate::tokenizer::{Keywords, Literals, Token};

use super::{
    error, lambda::Lambda, r#if::If, r#let::Let, r#match::Match, r#type::NamespacedType, Parser,
    ParserError, ParserErrorStack,
};

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
        let peek = value
            .first()
            .ok_or(error!("Exp", format!("Expected more tokens")))?;

        match peek {
            Token::Identifier(_) => {
                if matches!(
                    value.nth(1),
                    Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyOpen)
                ) {
                    Ok(Exp::TypeCreation(error!(
                        TypeCreation::try_from(&mut *value),
                        "Exp"
                    )?))
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

                if value.first() == Some(&Token::Keyword(Keywords::Mut)) {
                    value.pop_front();
                    Ok(Exp::MutRef(Box::new(error!(
                        Exp::try_from(&mut *value),
                        "Exp"
                    )?)))
                } else {
                    Ok(Exp::Ref(Box::new(error!(
                        Exp::try_from(&mut *value),
                        "Exp"
                    )?)))
                }
            }
            Token::Char('*') => {
                value.pop_front();

                Ok(Exp::Deref(Box::new(error!(
                    Exp::try_from(&mut *value),
                    "Exp"
                )?)))
            }
            Token::AngleBracketOpen => Ok(Exp::TypeCreation(error!(
                TypeCreation::try_from(&mut *value),
                "Exp"
            )?)),
            Token::ParenOpen => {
                match value
                    .nth(1)
                    .ok_or(error!("Exp", format!("Expected more tokens")))?
                {
                    Token::Keyword(Keywords::Not) => {
                        value.pop_front();
                        value.pop_front();

                        Ok(Exp::Negation(Box::new(error!(
                            Exp::try_from(&mut *value),
                            "Exp"
                        )?)))
                    }
                    Token::Keyword(Keywords::If) => {
                        Ok(Exp::If(Box::new(error!(If::try_from(&mut *value), "Exp")?)))
                    }
                    Token::Keyword(Keywords::Match) => Ok(Exp::Match(Box::new(error!(
                        Match::try_from(&mut *value),
                        "Exp"
                    )?))),
                    Token::Keyword(Keywords::Let) => Ok(Exp::Let(Box::new(error!(
                        Let::try_from(&mut *value),
                        "Exp"
                    )?))),
                    Token::Keyword(Keywords::Lambda) => Ok(Exp::Lambda(Box::new(error!(
                        Lambda::try_from(&mut *value),
                        "Exp"
                    )?))),
                    Token::Identifier(_) => {
                        if matches!(
                            value.nth(2),
                            Some(&Token::Keyword(Keywords::Arrow) | &Token::CurlyClose)
                        ) {
                            Ok(Exp::TypeCreation(error!(
                                TypeCreation::try_from(&mut *value),
                                "Exp"
                            )?))
                        } else {
                            value.pop_front();
                            let func =
                                if let Token::Identifier(iden) = value.pop_front_err("Exp")? {
                                    iden
                                } else {
                                    unreachable!()
                                };
                            let mut args = vec![];

                            loop {
                                let peek = value
                                    .first()
                                    .ok_or(error!("Exp", format!("Expected more tokens")))?;

                                if peek == &Token::ParenClose {
                                    value.pop_front();
                                    break;
                                }

                                args.push(error!(Exp::try_from(&mut *value), "exp")?)
                            }

                            Ok(Exp::FuncCall(Box::new(Exp::Variable(func)), args))
                        }
                    }
                    _ => {
                        value.pop_front();

                        if let Ok(infix) = Infix::try_from(&mut *value) {
                            let infix = Ok(Self::Infix(Box::new(infix)));
                            let next = value.pop_front_err("Exp")?;
                            if next != Token::ParenClose {
                                return Err(error!(
                                    "Exp",
                                    format!("Expected parenClose, got {next:#?}")
                                ));
                            }
                            return infix;
                        }

                        let exp = error!(Exp::try_from(&mut *value), "Exp")?;

                        let next = value
                            .first()
                            .ok_or(error!("Exp", format!("Expected more tokens")))?;
                        if next == &Token::ParenClose {
                            value.pop_front();
                            Ok(exp)
                        } else {
                            let mut params = vec![];

                            loop {
                                let peek = value
                                    .first()
                                    .ok_or(error!("Exp", format!("Expected more tokens")))?;

                                if peek == &Token::ParenClose {
                                    value.pop_front();
                                    break;
                                }

                                params.push(error!(Exp::try_from(&mut *value), "Exp")?)
                            }

                            Ok(Exp::FuncCall(Box::new(exp), params))
                        }
                    }
                }
            }
            token => {
                panic!("Not implemented {token:#?}")
            }
        }
    }
}

impl ToString for Exp {
    fn to_string(&self) -> String {
        match self {
            Exp::Lambda(lambda) => format!("({})", lambda.to_string()),
            Exp::If(r#if) => format!("{{{}}}", r#if.to_string()),
            Exp::Match(r#match) => format!("{{{}}}", r#match.to_string()),
            Exp::Let(r#let) => format!("{{{}}}", r#let.to_string()),
            Exp::FuncCall(func, args) => format!(
                "{{{}({})}}",
                func.to_string(),
                &if args.is_empty() {
                    format!(", ")
                } else {
                    args.iter().fold(String::new(), |str, arg| {
                        format!("{str}, {}", arg.to_string())
                    })
                }[2..]
            ),
            Exp::Ref(exp) => format!("{{&{{{}}}}}", exp.to_string()),
            Exp::MutRef(exp) => format!("{{&mut {{{}}}}}", exp.to_string()),
            Exp::Deref(exp) => format!("{{*{{{}}}}}", exp.to_string()),
            Exp::Negation(exp) => format!("{{!{{{}}}}}", exp.to_string()),
            Exp::Infix(infix) => format!("{{{}}}", infix.to_string()),
            Exp::Variable(var) => format!("{var}"),
            Exp::Literal(literal) => format!("{}", literal.to_string()),
            Exp::TypeCreation(creation) => format!("{{{}}}", creation.to_string()),
        }
    }
}

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
                let peek = value
                    .first()
                    .ok_or(error!("TypeCreation", format!("Expected more tokens")))?;

                if peek == &Token::AngleBracketClose {
                    value.pop_front();
                    return Ok(TypeCreation::Touple(exps));
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
            }
            TypeCreation::Touple(exps) => format!(
                "({})",
                &if exps.is_empty() {
                    format!(", ")
                } else {
                    exps.iter().fold(String::new(), |str, exp| {
                        format!("{str}, {}", exp.to_string())
                    })
                }[2..]
            ),
        }
    }
}

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
    LeftShift(Exp, Exp),
    RightShift(Exp, Exp),
}

impl TryFrom<&mut Parser> for Infix {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(
            match value
                .first()
                .ok_or(error!("Infix", format!("Expected more tokens")))?
            {
                Token::Char('+') => {
                    value.pop_front();
                    Self::Add(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Char('-') => {
                    value.pop_front();
                    Self::Subtract(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Char('*') => {
                    value.pop_front();
                    Self::Multiply(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Slash => {
                    value.pop_front();
                    if value.first() == Some(&Token::Char('=')) {
                        value.pop_front();
                        Self::NotEquality(
                            error!(Exp::try_from(&mut *value), "Infix")?,
                            error!(Exp::try_from(&mut *value), "Infix")?,
                        )
                    } else {
                        Self::Divide(
                            error!(Exp::try_from(&mut *value), "Infix")?,
                            error!(Exp::try_from(&mut *value), "Infix")?,
                        )
                    }
                }
                Token::Char('=') => {
                    value.pop_front();
                    Self::Equality(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Keyword(Keywords::And) => {
                    value.pop_front();
                    Self::And(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Keyword(Keywords::Or) => {
                    value.pop_front();
                    Self::Or(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Keyword(Keywords::Xor) => {
                    value.pop_front();
                    Self::Xor(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Keyword(Keywords::BitwiseAnd) => {
                    value.pop_front();
                    Self::BitwiseAnd(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::Keyword(Keywords::BitwiseOr) => {
                    value.pop_front();
                    Self::BitwiseOr(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
                }
                Token::AngleBracketOpen => {
                    value.pop_front();
                    match value.first() {
                        Some(&Token::Char('=')) => {
                            value.pop_front();
                            Self::LessEq(
                                error!(Exp::try_from(&mut *value), "Infix")?,
                                error!(Exp::try_from(&mut *value), "Infix")?,
                            )
                        }
                        Some(&Token::AngleBracketOpen) => {
                            value.pop_front();
                            Self::LeftShift(
                                error!(Exp::try_from(&mut *value), "Infix")?,
                                error!(Exp::try_from(&mut *value), "Infix")?,
                            )
                        }
                        _ => Self::Less(
                            error!(Exp::try_from(&mut *value), "Infix")?,
                            error!(Exp::try_from(&mut *value), "Infix")?,
                        ),
                    }
                }
                Token::AngleBracketClose => {
                    value.pop_front();
                    match value.first() {
                        Some(&Token::Char('=')) => {
                            value.pop_front();
                            Self::GreaterEq(
                                error!(Exp::try_from(&mut *value), "Infix")?,
                                error!(Exp::try_from(&mut *value), "Infix")?,
                            )
                        }
                        Some(&Token::AngleBracketClose) => {
                            value.pop_front();
                            Self::RightShift(
                                error!(Exp::try_from(&mut *value), "Infix")?,
                                error!(Exp::try_from(&mut *value), "Infix")?,
                            )
                        }
                        _ => Self::Greater(
                            error!(Exp::try_from(&mut *value), "Infix")?,
                            error!(Exp::try_from(&mut *value), "Infix")?,
                        ),
                    }
                }
                token => {
                    return Err(error!(
                        "Infix",
                        format!("Token isn't infix, got {token:#?}")
                    ))
                }
            },
        )
    }
}

impl ToString for Infix {
    fn to_string(&self) -> String {
        match self {
            Infix::Add(left, right) => format!("({}) + ({})", left.to_string(), right.to_string()),
            Infix::Subtract(left, right) => {
                format!("({}) - ({})", left.to_string(), right.to_string())
            }
            Infix::Multiply(left, right) => {
                format!("({}) * ({})", left.to_string(), right.to_string())
            }
            Infix::Divide(left, right) => {
                format!("({}) / ({})", left.to_string(), right.to_string())
            }
            Infix::Equality(left, right) => {
                format!("({}) == ({})", left.to_string(), right.to_string())
            }
            Infix::NotEquality(left, right) => {
                format!("({}) != ({})", left.to_string(), right.to_string())
            }
            Infix::And(left, right) => format!("({}) && ({})", left.to_string(), right.to_string()),
            Infix::Or(left, right) => format!("({}) || ({})", left.to_string(), right.to_string()),
            Infix::Less(left, right) => format!("({}) < ({})", left.to_string(), right.to_string()),
            Infix::LessEq(left, right) => {
                format!("({}) <= ({})", left.to_string(), right.to_string())
            }
            Infix::Greater(left, right) => {
                format!("({}) > ({})", left.to_string(), right.to_string())
            }
            Infix::GreaterEq(left, right) => {
                format!("({}) >= ({})", left.to_string(), right.to_string())
            }
            Infix::Xor(left, right) => format!("({}) ^ ({})", left.to_string(), right.to_string()),
            Infix::BitwiseAnd(left, right) => {
                format!("({}) & ({})", left.to_string(), right.to_string())
            }
            Infix::BitwiseOr(left, right) => {
                format!("({}) | ({})", left.to_string(), right.to_string())
            }
            Infix::LeftShift(left, right) => {
                format!("({}) << ({})", left.to_string(), right.to_string())
            }
            Infix::RightShift(left, right) => {
                format!("({}) >> ({})", left.to_string(), right.to_string())
            }
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
