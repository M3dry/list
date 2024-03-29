use crate::tokenizer::{Int, Keywords, Literals, Token};

use super::{
    error,
    lambda::Lambda,
    r#as::As,
    r#do::Do,
    r#if::If,
    r#let::Let,
    r#match::Match,
    r#type::NamespacedType,
    range::Range,
    turbofish::{TurboFish, TurboIden},
    Error, Parser, ParserError, ParserErrorStack,
};

#[derive(Debug)]
pub enum Exp {
    Lambda(Box<Lambda>),
    If(Box<If>),
    Match(Box<Match>),
    Let(Box<Let>),
    As(Box<As>),
    FuncCall(Box<Exp>, Vec<Exp>),
    MethodCall(Box<Exp>, TurboIden, Vec<Exp>),
    Ref(Box<Exp>),
    MutRef(Box<Exp>),
    Deref(Box<Exp>),
    Not(Box<Exp>),
    Negation(Box<Exp>),
    Positive(Box<Exp>),
    Infix(Box<Infix>),
    Range(Box<Range>),
    Do(Box<Do>),
    Variable(String),
    Field(Box<Exp>, String),
    Literal(Literals),
    TypeCreation(TypeCreation),
    Return(Box<Exp>),
    ErrorOut(Box<Exp>),
    TurboFish(TurboFish),
}

impl TryFrom<&mut Parser> for Exp {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut ret = match error!(
            "Exp",
            value.first(),
            [
                Token::Identifier(_),
                Token::Literal(_),
                Token::Ref,
                Token::Char('-'),
                Token::Char('+'),
                Token::Keyword(Keywords::Deref),
                Token::AngleBracketOpen,
                Token::BracketOpen,
                Token::ParenOpen
            ]
        )? {
            Token::Identifier(_) => match value.nth(1) {
                Some(&Token::Keyword(Keywords::LeftArrow) | &Token::CurlyOpen) => {
                    Self::TypeCreation(error!(TypeCreation::try_from(&mut *value), "Exp")?)
                }
                Some(&Token::Keyword(Keywords::TurboStart)) => {
                    Self::TurboFish(error!(TurboFish::try_from(&mut *value), "Exp")?)
                }
                _ => {
                    Self::Variable(error!("Exp", value))
                }
            },
            Token::Literal(_) => {
                if let Token::Literal(literal) = value.pop_front().unwrap() {
                    Self::Literal(literal)
                } else {
                    unreachable!()
                }
            }
            Token::Ref => {
                value.pop_front();

                if value.first() == Some(&Token::Keyword(Keywords::Mut)) {
                    value.pop_front();
                    Self::MutRef(Box::new(error!(Self::try_from(&mut *value), "Exp")?))
                } else {
                    Self::Ref(Box::new(error!(Self::try_from(&mut *value), "Exp")?))
                }
            }
            Token::Char('-') => {
                value.pop_front();

                Self::Negation(Box::new(error!(Self::try_from(&mut *value), "Exp")?))
            }
            Token::Char('+') => {
                value.pop_front();

                Self::Positive(Box::new(error!(Self::try_from(&mut *value), "Exp")?))
            }
            Token::Keyword(Keywords::Deref) => {
                value.pop_front();

                Self::Deref(Box::new(error!(Self::try_from(&mut *value), "Exp")?))
            }
            Token::AngleBracketOpen => {
                Self::TypeCreation(error!(TypeCreation::try_from(&mut *value), "Exp")?)
            }
            Token::BracketOpen => {
                Self::TypeCreation(error!(TypeCreation::try_from(&mut *value), "Exp")?)
            }
            Token::ParenOpen => match value.nth_err(1, "Exp")? {
                Token::Keyword(Keywords::Not) => {
                    value.pop_front();
                    value.pop_front();

                    let ret = Self::Not(Box::new(error!(Self::try_from(&mut *value), "Exp")?));
                    let _ = error!("Exp", value.pop_front(), [Token::ParenClose])?;

                    ret
                }
                Token::Keyword(Keywords::If) => {
                    Self::If(Box::new(error!(If::try_from(&mut *value), "Exp")?))
                }
                Token::Keyword(Keywords::Match) => {
                    Self::Match(Box::new(error!(Match::try_from(&mut *value), "Exp")?))
                }
                Token::Keyword(Keywords::Let) => {
                    Self::Let(Box::new(error!(Let::try_from(&mut *value), "Exp")?))
                }
                Token::Keyword(Keywords::Lambda) => {
                    Self::Lambda(Box::new(error!(Lambda::try_from(&mut *value), "Exp")?))
                }
                Token::Keyword(Keywords::As) => {
                    Self::As(Box::new(error!(As::try_from(&mut *value), "Exp")?))
                }
                Token::Keyword(Keywords::Do) => {
                    Self::Do(Box::new(error!(Do::try_from(&mut *value), "Exp")?))
                }
                Token::Keyword(Keywords::Return) => {
                    value.pop_front();
                    value.pop_front();

                    Self::Return(Box::new(error!(Exp::try_from(&mut *value), "Exp")?))
                }
                Token::Identifier(_) => match value.nth(2) {
                    Some(&Token::Slash | &Token::Char('.')) => {
                        value.pop_front();
                        let mut exp = Self::Variable(error!("Exp", value));

                        loop {
                            exp = match error!(
                                "Exp",
                                value.first(),
                                [Token::Slash, Token::Char('.'), Token::ParenClose]
                            )? {
                                Token::Slash => {
                                    value.pop_front();
                                    Self::Field(
                                        Box::new(exp),
                                        error!("Exp", value),
                                    )
                                }
                                Token::Char('.') => {
                                    value.pop_front();
                                    let method = error!(TurboIden::try_from(&mut *value), "Exp")?;

                                    if matches!(
                                        value.first(),
                                        Some(&Token::Slash | &Token::Char('.'))
                                    ) {
                                        Self::MethodCall(Box::new(exp), method, vec![])
                                    } else {
                                        let mut args = vec![];

                                        break loop {
                                            let peek = value.first_err("Exp")?;

                                            if peek == &Token::ParenClose {
                                                value.pop_front();
                                                break Self::MethodCall(
                                                    Box::new(exp),
                                                    method,
                                                    args,
                                                );
                                            }

                                            args.push(error!(Exp::try_from(&mut *value), "Exp")?)
                                        };
                                    }
                                }
                                Token::ParenClose => {
                                    value.pop_front();
                                    break exp;
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    Some(&Token::Keyword(Keywords::LeftArrow) | &Token::CurlyOpen) => {
                        Self::TypeCreation(error!(TypeCreation::try_from(&mut *value), "Exp")?)
                    }
                    _ => {
                        value.pop_front();
                        let func = error!("Exp", value);
                        let mut args = vec![];

                        loop {
                            let peek = value.first_err("Exp")?;

                            if peek == &Token::ParenClose {
                                value.pop_front();
                                break;
                            }

                            args.push(error!(Self::try_from(&mut *value), "exp")?)
                        }

                        Self::FuncCall(Box::new(Self::Variable(func)), args)
                    }
                },
                _ => {
                    value.pop_front();

                    if let Ok(infix) = Infix::try_from(&mut *value) {
                        let infix = Self::Infix(Box::new(infix));
                        let _ = error!("Exp", value.pop_front(), [Token::ParenClose])?;

                        return Ok(infix);
                    }

                    let mut exp = error!(Self::try_from(&mut *value), "Exp")?;

                    let peek = value.first_err("Exp")?;
                    if peek == &Token::ParenClose {
                        value.pop_front();
                        exp
                    } else if peek == &Token::Char('.') {
                        loop {
                            exp = match error!(
                                "Exp",
                                value.first(),
                                [Token::Slash, Token::Char('.'), Token::ParenClose]
                            )? {
                                Token::Slash => {
                                    value.pop_front();
                                    Self::Field(
                                        Box::new(exp),
                                        error!("Exp", value),
                                    )
                                }
                                Token::Char('.') => {
                                    value.pop_front();
                                    let method = error!(TurboIden::try_from(&mut *value), "Exp")?;

                                    if matches!(
                                        value.first(),
                                        Some(&Token::Slash | &Token::Char('.'))
                                    ) {
                                        Self::MethodCall(Box::new(exp), method, vec![])
                                    } else {
                                        let mut args = vec![];

                                        break loop {
                                            let peek = value.first_err("Exp")?;

                                            if peek == &Token::ParenClose {
                                                value.pop_front();
                                                break Self::MethodCall(
                                                    Box::new(exp),
                                                    method,
                                                    args,
                                                );
                                            }

                                            args.push(error!(Exp::try_from(&mut *value), "Exp")?)
                                        };
                                    }
                                }
                                Token::ParenClose => {
                                    value.pop_front();
                                    break exp;
                                }
                                _ => unreachable!(),
                            }
                        }
                    } else {
                        let mut params = vec![];

                        loop {
                            let peek = value.first_err("Exp")?;

                            if peek == &Token::ParenClose {
                                value.pop_front();
                                break;
                            }

                            params.push(error!(Self::try_from(&mut *value), "Exp")?)
                        }

                        Self::FuncCall(Box::new(exp), params)
                    }
                }
            },
            _ => unreachable!(),
        };

        loop {
            ret = match value.first() {
                Some(&Token::Char('?')) => {
                    value.pop_front();
                    Self::ErrorOut(Box::new(ret))
                }
                Some(&Token::DoubleDot) => {
                    value.pop_front();
                    let inclusive = if value.first() == Some(&Token::Char('=')) {
                        value.pop_front();
                        true
                    } else {
                        false
                    };

                    if matches!(value.first(), Some(&Token::Identifier(ref iden)) if iden ==  "_")
                        && !inclusive
                    {
                        value.pop_front();
                        Self::Range(Box::new(Range::Infinite(ret)))
                    } else {
                        let end = error!(Exp::try_from(&mut *value), "Exp")?;

                        Self::Range(Box::new(if inclusive {
                            Range::Inclusive(ret, end)
                        } else {
                            Range::Normal(ret, end)
                        }))
                    }
                }
                Some(&Token::Slash) => {
                    value.pop_front();
                    Self::Field(Box::new(ret), error!("Exp", value))
                }
                _ => return Ok(ret),
            };
        }
    }
}

impl ToString for Exp {
    fn to_string(&self) -> String {
        match self {
            Self::Lambda(lambda) => format!("({})", lambda.to_string()),
            Self::If(r#if) => format!("{}", r#if.to_string()),
            Self::Match(r#match) => format!("{}", r#match.to_string()),
            Self::Let(r#let) => format!("{{{}}}", r#let.to_string()),
            Self::As(r#as) => format!("({})", r#as.to_string()),
            Self::FuncCall(func, args) => format!(
                "{}({})",
                func.to_string(),
                &if args.is_empty() {
                    format!(", ")
                } else {
                    args.iter().fold(String::new(), |str, arg| {
                        format!("{str}, {}", arg.to_string())
                    })
                }[2..]
            ),
            Self::MethodCall(exp, method, args) => format!(
                "{}.{}({})",
                exp.to_string(),
                method.to_string(),
                &if args.is_empty() {
                    format!(", ")
                } else {
                    args.iter().fold(String::new(), |str, arg| {
                        format!("{str}, {}", arg.to_string())
                    })
                }[2..]
            ),
            Self::Ref(exp) => format!("&{}", exp.to_string()),
            Self::MutRef(exp) => format!("&mut {}", exp.to_string()),
            Self::Deref(exp) => format!("*{}", exp.to_string()),
            Self::Not(exp) => format!("!{}", exp.to_string()),
            Self::Positive(exp) => format!("+{}", exp.to_string()),
            Self::Negation(exp) => format!("-{}", exp.to_string()),
            Self::Infix(infix) => format!("{}", infix.to_string()),
            Self::Range(range) => range.to_string(),
            Self::Do(r#do) => format!("{}", r#do.to_string()),
            Self::Variable(var) => var.to_string(),
            Self::Field(exp, field) => format!("{}.{field}", exp.to_string()),
            Self::Literal(literal) => literal.to_string(),
            Self::TypeCreation(creation) => format!("{}", creation.to_string()),
            Self::Return(exp) => format!("return {}", exp.to_string()),
            Self::ErrorOut(exp) => format!("{}?", exp.to_string()),
            Self::TurboFish(turbofish) => turbofish.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum TypeCreation {
    Simple(NamespacedType),
    Vars(NamespacedType, Vec<Exp>),
    Struct(NamespacedType, Vec<(String, Exp)>),
    Touple(Vec<Exp>),
    Array(Vec<Exp>),
    ArrayLen(Box<Exp>, usize),
}

impl TryFrom<&mut Parser> for TypeCreation {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut paren = false;
        let peek = value.first_err("TypeCreation")?;
        if peek == &Token::ParenOpen {
            value.pop_front();
            paren = true;
        } else if peek == &Token::AngleBracketOpen {
            value.pop_front();
            let mut exps = vec![];

            loop {
                let peek = value.first_err("TypeCreation")?;

                if peek == &Token::AngleBracketClose {
                    value.pop_front();
                    break;
                }

                exps.push(error!(Exp::try_from(&mut *value), "TypeCreation")?)
            }

            TypeCreation::Touple(exps);
        } else if peek == &Token::BracketOpen {
            value.pop_front();
            let mut exps = vec![];

            loop {
                let peek = value.first_err("TypeCreation")?;

                match peek {
                    Token::BracketClose => {
                        value.pop_front();
                        return Ok(TypeCreation::Array(exps));
                    }
                    Token::Char(';') => {
                        value.pop_front();

                        let ret = match error!(
                            "TypeCreation",
                            value.pop_front(),
                            [Token::Literal(Literals::Int(Int(false, _)))]
                        )? {
                            Token::Literal(Literals::Int(Int(false, len))) => Ok(Self::ArrayLen(
                                Box::new(exps.into_iter().next().ok_or(error!(
                                    "TypeCreation",
                                    Error::Other(format!("Expected an expression"))
                                ))?),
                                len as usize,
                            )),
                            _ => unreachable!(),
                        };
                        let _ = error!("TypeCreation", value.pop_front(), [Token::BracketClose])?;

                        return ret;
                    }
                    _ => exps.push(error!(Exp::try_from(&mut *value), "TypeCreation")?),
                }
            }
        }

        let name = error!(NamespacedType::try_from(&mut *value), "TypeCreation")?;

        let ret = match value.first() {
            Some(Token::CurlyOpen) => {
                value.pop_front();
                let mut fields = vec![];

                loop {
                    if value.first_err("TypeCreation")? == &Token::CurlyClose {
                        value.pop_front();
                        break;
                    }

                    let name = error!("TypeCreation", value);
                    let _ = error!(
                        "TypeCreation",
                        value.pop_front(),
                        [Token::Keyword(Keywords::LeftArrow)]
                    )?;
                    let exp = error!(Exp::try_from(&mut *value), "TypeCreation")?;

                    fields.push((name, exp));
                }

                Ok(Self::Struct(name, fields))
            }
            Some(_) if paren => {
                let mut args = vec![];

                loop {
                    if value.first_err("TypeCreation")? == &Token::ParenClose {
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
            let _ = error!("TypeCreation", value.pop_front(), [Token::ParenClose])?;
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
                &if args.is_empty() {
                    format!(", ")
                } else {
                    args.iter().fold(String::new(), |str, arg| {
                        format!("{str}, {}", arg.to_string())
                    })
                }[2..]
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
            TypeCreation::Array(exps) => format!(
                "[{}]",
                &if exps.is_empty() {
                    format!(", ")
                } else {
                    exps.iter().fold(String::new(), |str, exp| {
                        format!("{str}, {}", exp.to_string())
                    })
                }[2..]
            ),
            TypeCreation::ArrayLen(exp, len) => {
                format!("[{};{}]", exp.to_string(), len.to_string())
            }
        }
    }
}

#[derive(Debug)]
pub enum Infix {
    Add(Exp, Exp),
    Subtract(Exp, Exp),
    Multiply(Exp, Exp),
    Divide(Exp, Exp),
    Modulo(Exp, Exp),
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
            match error!(
                "Infix",
                value.first(),
                [
                    Token::Char('+' | '-' | '*' | '%' | '='),
                    Token::Slash,
                    Token::Keyword(
                        Keywords::And
                            | Keywords::Or
                            | Keywords::Xor
                            | Keywords::BitwiseAnd
                            | Keywords::BitwiseOr
                    ),
                    Token::AngleBracketOpen,
                    Token::AngleBracketClose
                ]
            )? {
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
                Token::Char('%') => {
                    value.pop_front();
                    Self::Modulo(
                        error!(Exp::try_from(&mut *value), "Infix")?,
                        error!(Exp::try_from(&mut *value), "Infix")?,
                    )
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
                _ => unreachable!(),
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
            Infix::Modulo(left, right) => {
                format!("({}) % ({})", left.to_string(), right.to_string())
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
