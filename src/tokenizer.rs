use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct Tokens(pub(crate) Vec<Token>);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Token {
    Literal(Literals),
    Keyword(Keywords),
    Type(Types),
    ParenOpen,
    ParenClose,
    Identifier(String),
}

impl From<Vec<Token>> for Tokens {
    fn from(value: Vec<Token>) -> Self {
        Tokens(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Literals {
    Int(bool, u128),
    String(String),
    Char(char),
    Bool(bool),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Keywords {
    If,
    Match,
    Defun,
    Lambda,
    Arrow,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Types {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    String,
    Char,
    Bool,
}

impl FromStr for Tokens {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = vec![];
        let mut chars = s.chars().peekable();

        'l: while let Some(char) = chars.next() {
            match char {
                '(' => tokens.push(Token::ParenOpen),
                ')' => tokens.push(Token::ParenClose),
                '\'' => {
                    let ch = chars.next().unwrap();

                    if !matches!(chars.next(), Some('\'')) {
                        return Err("unfinished closing single quote");
                    }

                    tokens.push(Token::Literal(Literals::Char(ch)));
                }
                '"' => {
                    let mut literal = vec![chars.next().unwrap()];
                    let mut backslash = false;

                    while let Some(char) = chars.next() {
                        if char == '"' && !backslash {
                            break;
                        } else if char == '\\' && !backslash {
                            backslash = true
                        } else {
                            literal.push(char);
                            backslash = false
                        }
                    }

                    tokens.push(Token::Literal(Literals::String(
                        literal.into_iter().collect(),
                    )));
                }
                char => {
                    if char == '(' || char == ')' || char == ' ' {
                        continue;
                    }
                    let mut chs = vec![char];

                    while let Some(char) = chars.peek() {
                        if *char == '(' || *char == ')' || *char == ' ' {
                            break;
                        } else if *char == '-' {
                            let char = chars.next().unwrap();
                            if matches!(chars.peek(), Some('>')) {
                                chars.next();
                                tokens.append(&mut token_from_str(
                                    &chs.into_iter().collect::<String>()[..],
                                ));
                                tokens.push(Token::Keyword(Keywords::Arrow));
                                continue 'l;
                            } else {
                                chs.push(char);
                                chs.push(chars.next().unwrap());
                            }
                        } else {
                            chs.push(chars.next().unwrap())
                        }
                    }

                    tokens.append(&mut token_from_str(
                        &chs.into_iter().collect::<String>()[..],
                    ));
                }
            }
        }

        Ok(Tokens::from(tokens))
    }
}

fn token_from_str(str: &str) -> Vec<Token> {
    match str {
        "if" => vec![Token::Keyword(Keywords::If)],
        "match" => vec![Token::Keyword(Keywords::Match)],
        "defun" => vec![Token::Keyword(Keywords::Defun)],
        arrow if arrow.len() >= 2 && &arrow[..2] == "->" => {
            let mut ret = vec![Token::Keyword(Keywords::Arrow)];
            ret.append(&mut token_from_str(&str[2..]));
            ret
        }
        "lambda" => vec![Token::Keyword(Keywords::Lambda)],
        "true" => vec![Token::Literal(Literals::Bool(true))],
        "false" => vec![Token::Literal(Literals::Bool(false))],
        "u8" => vec![Token::Type(Types::U8)],
        "u16" => vec![Token::Type(Types::U16)],
        "u32" => vec![Token::Type(Types::U32)],
        "u64" => vec![Token::Type(Types::U64)],
        "u128" => vec![Token::Type(Types::U128)],
        "i8" => vec![Token::Type(Types::I8)],
        "i16" => vec![Token::Type(Types::I16)],
        "i32" => vec![Token::Type(Types::I32)],
        "i64" => vec![Token::Type(Types::I64)],
        "i128" => vec![Token::Type(Types::I128)],
        "string" => vec![Token::Type(Types::String)],
        "char" => vec![Token::Type(Types::Char)],
        "bool" => vec![Token::Type(Types::Bool)],
        num if num.len() > 1 && &num[..1] == "-" && num[1..].parse::<u128>().is_ok() => {
            vec![Token::Literal(Literals::Int(
                true,
                num[1..].parse().unwrap(),
            ))]
        }
        num if num.parse::<u128>().is_ok() => {
            vec![Token::Literal(Literals::Int(false, num.parse().unwrap()))]
        }
        identifier
            if identifier.len() > 0
                && identifier.chars().all(|c| {
                    c.is_ascii_alphanumeric()
                        || [
                            '!', '@', '#', '$', '%', '^', '&', '*', '-', '_', '=', '+', ',', '.',
                            '<', '>', '?',
                        ]
                        .contains(&c)
                }) =>
        {
            vec![Token::Identifier(identifier.to_string())]
        }
        _ => vec![],
    }
}
