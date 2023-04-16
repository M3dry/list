use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Tokens(pub(crate) Vec<Token>);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Token {
    BackTick,
    Literal(Literals),
    Keyword(Keywords),
    Type(BuiltinTypes),
    DoubleDot,
    Ref,
    Char(char),
    Slash,
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    AngleBracketOpen,
    AngleBracketClose,
    CurlyOpen,
    CurlyClose,
    Identifier(String),
}

impl From<Vec<Token>> for Tokens {
    fn from(value: Vec<Token>) -> Self {
        Tokens(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Int(pub bool, pub u128);

impl Int {
    fn digs(start: u128, chars: &mut Peekable<Chars>) -> u128 {
        let mut num = start;

        loop {
            match chars.peek() {
                Some(c) => match c.to_digit(10) {
                    Some(dig) => {
                        chars.next();
                        num = dig as u128 + num * 10
                    }
                    None => break num,
                },
                None => break num,
            }
        }
    }
}

impl ToString for Int {
    fn to_string(&self) -> String {
        match self {
            Self(true, num) => format!("-{}", &num.to_string()),
            Self(false, num) => num.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literals {
    Int(Int),
    String(String),
    Char(char),
    Bool(bool),
}

impl ToString for Literals {
    fn to_string(&self) -> String {
        match self {
            Self::Int(int) => int.to_string(),
            Self::String(str) => format!(r#""{str}""#),
            Self::Char(char) => format!("'{char}'"),
            Self::Bool(true) => "true".to_string(),
            Self::Bool(false) => "false".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Keywords {
    TurboStart,
    If,
    Else,
    Elif,
    Match,
    Defun,
    Lambda,
    Let,
    Struct,
    Enum,
    Use,
    For,
    Loop,
    While,
    Break,
    Impl,
    As,
    Continue,
    LeftArrow,
    RightArrow,
    Return,
    Do,
    Mut,
    Deref,
    Not,
    And,
    Or,
    Xor,
    BitwiseAnd,
    BitwiseOr,
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
                '<' if chars.peek() == Some(&'-') => {
                    chars.next();
                    tokens.push(Token::Keyword(Keywords::RightArrow))
                }
                '<' => tokens.push(Token::AngleBracketOpen),
                '>' => tokens.push(Token::AngleBracketClose),
                ']' => tokens.push(Token::BracketClose),
                '{' => tokens.push(Token::CurlyOpen),
                '}' => tokens.push(Token::CurlyClose),
                '/' => tokens.push(Token::Slash),
                '\'' => {
                    let ch = chars.next().unwrap();

                    if !matches!(chars.next(), Some('\'')) {
                        return Err("unfinished closing single quote");
                    }

                    tokens.push(Token::Literal(Literals::Char(ch)));
                }
                '"' => {
                    let mut literal = vec![];
                    let mut backslash = false;

                    while let Some(char) = chars.next() {
                        if char == '"' && !backslash {
                            break;
                        } else if char == '\\' && !backslash {
                            backslash = true;
                            literal.push(char)
                        } else {
                            literal.push(char);
                            backslash = false
                        }
                    }

                    tokens.push(Token::Literal(Literals::String(
                        literal.into_iter().collect(),
                    )));
                }
                '&' => tokens.push(Token::Ref),
                '.' if chars.peek() == Some(&'.') => {
                    chars.next();
                    tokens.push(Token::DoubleDot)
                }
                '-' if chars.peek() == Some(&'-') => {
                    chars.next();
                    while chars.next() != Some('\n') {
                    }
                }
                '-' if matches!(chars.peek(), Some(c) if c.is_ascii_digit()) => tokens.push(
                    Token::Literal(Literals::Int(Int(true, Int::digs(0, &mut chars)))),
                ),
                '-' if chars.peek() == Some(&'>') => {
                    chars.next();
                    tokens.push(Token::Keyword(Keywords::LeftArrow))
                }
                '\n' => continue,
                '*' if chars.peek() != Some(&' ') => tokens.push(Token::Keyword(Keywords::Deref)),
                char if char.is_ascii_digit() => tokens.push(Token::Literal(Literals::Int(Int(
                    false,
                    Int::digs(char.to_digit(10).unwrap() as u128, &mut chars),
                )))),
                char if char.is_ascii_alphanumeric() || char == '_' => {
                    let mut chs = vec![char];

                    while let Some(char) = chars.peek() {
                        if !char.is_ascii_alphanumeric()
                            && *char != '-'
                            && *char != '_'
                            && *char != '!'
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

                    if chars.peek() == Some(&'<') {
                        chars.next();
                        tokens.push(Token::Keyword(Keywords::TurboStart))
                    }
                }
                char if char != ' ' => tokens.push(Token::Char(char)),
                _ => continue,
            }
        }

        Ok(Tokens::from(tokens))
    }
}

fn token_from_str(str: &str) -> Vec<Token> {
    match str {
        "`" => vec![Token::BackTick],
        "if" => vec![Token::Keyword(Keywords::If)],
        "else" => vec![Token::Keyword(Keywords::Else)],
        "elif" => vec![Token::Keyword(Keywords::Elif)],
        "match" => vec![Token::Keyword(Keywords::Match)],
        "defun" => vec![Token::Keyword(Keywords::Defun)],
        "lambda" => vec![Token::Keyword(Keywords::Lambda)],
        "let" => vec![Token::Keyword(Keywords::Let)],
        "struct" => vec![Token::Keyword(Keywords::Struct)],
        "enum" => vec![Token::Keyword(Keywords::Enum)],
        "use" => vec![Token::Keyword(Keywords::Use)],
        "for" => vec![Token::Keyword(Keywords::For)],
        "loop" => vec![Token::Keyword(Keywords::Loop)],
        "while" => vec![Token::Keyword(Keywords::While)],
        "break" => vec![Token::Keyword(Keywords::Break)],
        "impl" => vec![Token::Keyword(Keywords::Impl)],
        "as" => vec![Token::Keyword(Keywords::As)],
        "continue" => vec![Token::Keyword(Keywords::Continue)],
        "mut" => vec![Token::Keyword(Keywords::Mut)],
        "return" => vec![Token::Keyword(Keywords::Return)],
        "do" => vec![Token::Keyword(Keywords::Do)],
        "not" => vec![Token::Keyword(Keywords::Not)],
        "and" => vec![Token::Keyword(Keywords::And)],
        "or" => vec![Token::Keyword(Keywords::Or)],
        "xor" => vec![Token::Keyword(Keywords::Xor)],
        "band" => vec![Token::Keyword(Keywords::BitwiseAnd)],
        "bor" => vec![Token::Keyword(Keywords::BitwiseOr)],
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
        Some('>') => Ok(Triple::Token(Token::Keyword(Keywords::LeftArrow))),
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
