use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Tokens(pub(crate) Vec<Token>);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Token {
    Literal(Literals),
    Keyword(Keywords),
    Type(BuiltinTypes),
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    AngleBracketOpen,
    AngleBracketClose,
    CurlyOpen,
    CurlyClose,
    Identifier(String),
    Generic(String),
}

impl From<Vec<Token>> for Tokens {
    fn from(value: Vec<Token>) -> Self {
        Tokens(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literals {
    Int(bool, u128),
    String(String),
    Char(char),
    Bool(bool),
}

impl ToString for Literals {
    fn to_string(&self) -> String {
        match self {
            Literals::Int(true, num) => format!("-{}", &num.to_string()),
            Literals::Int(false, num) => num.to_string(),
            Literals::String(str) => format!(r#""{str}""#),
            Literals::Char(char) => format!("'{char}'"),
            Literals::Bool(true) => "true".to_string(),
            Literals::Bool(false) => "false".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Keywords {
    If,
    Match,
    Defun,
    Lambda,
    Let,
    Struct,
    Enum,
    Arrow,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinTypes {
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

impl ToString for BuiltinTypes {
    fn to_string(&self) -> String {
        match self {
            BuiltinTypes::U8 => "u8",
            BuiltinTypes::U16 => "u16",
            BuiltinTypes::U32 => "u32",
            BuiltinTypes::U64 => "u64",
            BuiltinTypes::U128 => "k128",
            BuiltinTypes::I8 => "i8",
            BuiltinTypes::I16 => "i16",
            BuiltinTypes::I32 => "i32",
            BuiltinTypes::I64 => "i64",
            BuiltinTypes::I128 => "i128",
            BuiltinTypes::String => "String",
            BuiltinTypes::Char => "char",
            BuiltinTypes::Bool => "bool",
        }
        .to_string()
    }
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
                '[' => tokens.push(Token::BracketOpen),
                '<' => tokens.push(Token::AngleBracketOpen),
                '>' => tokens.push(Token::AngleBracketClose),
                ']' => tokens.push(Token::BracketClose),
                '{' => tokens.push(Token::CurlyOpen),
                '}' => tokens.push(Token::CurlyClose),
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
                    if char == '('
                        || char == ')'
                        || char == '['
                        || char == ']'
                        || char == '<'
                        || char == '>'
                        || char == ' '
                    {
                        continue;
                    }
                    let mut chs = vec![];

                    if char == '-' {
                        match check_arrow(&mut chars)? {
                            Triple::Token(token) => {
                                tokens.push(token);
                                continue 'l;
                            }
                            Triple::Char(char) => {
                                chs.push(char);
                            }
                            Triple::None => {}
                        }
                    } else {
                        chs.push(char)
                    }

                    while let Some(char) = chars.peek() {
                        if *char == '('
                            || *char == ')'
                            || *char == '['
                            || *char == ']'
                            || *char == '<'
                            || *char == '>'
                            || *char == ' '
                        {
                            break;
                        } else if *char == '-' {
                            let ch = chars.next().unwrap();
                            match check_arrow(&mut chars)? {
                                Triple::Token(token) => {
                                    tokens.append(&mut token_from_str(
                                        &chs.into_iter().collect::<String>()[..],
                                    ));
                                    tokens.push(token);
                                    continue 'l;
                                }
                                Triple::Char(char) => {
                                    chs.push(ch);
                                    chs.push(char);
                                }
                                Triple::None => {}
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
        "lambda" => vec![Token::Keyword(Keywords::Lambda)],
        "let" => vec![Token::Keyword(Keywords::Let)],
        "struct" => vec![Token::Keyword(Keywords::Struct)],
        "enum" => vec![Token::Keyword(Keywords::Enum)],
        "true" => vec![Token::Literal(Literals::Bool(true))],
        "false" => vec![Token::Literal(Literals::Bool(false))],
        "u8" => vec![Token::Type(BuiltinTypes::U8)],
        "u16" => vec![Token::Type(BuiltinTypes::U16)],
        "u32" => vec![Token::Type(BuiltinTypes::U32)],
        "u64" => vec![Token::Type(BuiltinTypes::U64)],
        "u128" => vec![Token::Type(BuiltinTypes::U128)],
        "i8" => vec![Token::Type(BuiltinTypes::I8)],
        "i16" => vec![Token::Type(BuiltinTypes::I16)],
        "i32" => vec![Token::Type(BuiltinTypes::I32)],
        "i64" => vec![Token::Type(BuiltinTypes::I64)],
        "i128" => vec![Token::Type(BuiltinTypes::I128)],
        "string" => vec![Token::Type(BuiltinTypes::String)],
        "char" => vec![Token::Type(BuiltinTypes::Char)],
        "bool" => vec![Token::Type(BuiltinTypes::Bool)],
        arrow if arrow.len() >= 2 && &arrow[..2] == "->" => {
            let mut ret = vec![Token::Keyword(Keywords::Arrow)];
            ret.append(&mut token_from_str(&str[2..]));
            ret
        }
        num if num.len() > 1 && &num[..1] == "-" && num[1..].parse::<u128>().is_ok() => {
            vec![Token::Literal(Literals::Int(
                true,
                num[1..].parse().unwrap(),
            ))]
        }
        num if num.parse::<u128>().is_ok() => {
            vec![Token::Literal(Literals::Int(false, num.parse().unwrap()))]
        }
        generic if generic.len() > 1 && &generic[..1] == ":" => {
            vec![Token::Generic(generic[1..].to_string())]
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

enum Triple {
    Token(Token),
    Char(char),
    None,
}

fn check_arrow(chars: &mut Peekable<Chars>) -> Result<Triple, &'static str> {
    match chars.next() {
        Some('-') => {
            while !matches!(chars.next(), Some('\n') | None) {}
            Ok(Triple::None)
        }
        Some('>') => Ok(Triple::Token(Token::Keyword(Keywords::Arrow))),
        Some(char) => Ok(Triple::Char(char)),
        _ => Err("Expected more chars"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! snapshot {
        ($name:tt, $path:tt) => {
            #[test]
            fn $name() {
                let contents = include_str!($path);
                let mut settings = insta::Settings::clone_current();
                settings.set_snapshot_path("../testdata/tokenizer/");
                settings.bind(|| {
                    insta::assert_snapshot!(contents
                        .lines()
                        .filter_map(|line| if line != "" {
                            Some(format!("{line}\n{:#?}", Tokens::from_str(line)))
                        } else {
                            None
                        })
                        .collect::<Vec<String>>()
                        .join("\n\n"));
                });
            }
        };
    }

    snapshot!(test_calling, "../testdata/input/calling.lt");
    snapshot!(test_if, "../testdata/input/if.lt");
    snapshot!(test_match, "../testdata/input/match.lt");
    snapshot!(test_defun, "../testdata/input/defun.lt");
    snapshot!(test_lambda, "../testdata/input/lambda.lt");
    snapshot!(test_let, "../testdata/input/let.lt");
    snapshot!(test_struct, "../testdata/input/struct.lt");
    snapshot!(test_enum, "../testdata/input/enum.lt");
}
