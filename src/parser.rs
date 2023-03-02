use either::Either;
use peek_nth::{IteratorExt, PeekableNth};

use crate::pattern::{PatternToken, PatternTree};
use crate::tokenizer::{Keywords, Literals, Token, Tokens, Types};

#[derive(Debug)]
struct Defun {
    scope: Scope,
    name: String,
    args: ArgsTyped,
    return_type: Types,
    body: Exp,
}

impl TryFrom<&mut Parser> for Defun {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(ParserError::new("Defun", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(ParserError::new(
                "Defun",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let scope = Scope::try_from(&mut *value).unwrap_or(Scope::File);

        let next = value.next().ok_or(ParserError::new("Defun keyword", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Defun) {
            return Err(ParserError::new("Defun keyword", format!("Expected Defun keyword, got {next:#?}")));
        }

        let next = value.next().ok_or(ParserError::new(
            "Defun name",
            format!("Expected more tokens"),
        ))?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(ParserError::new(
                "Defun name",
                format!("Expected Identifier, got {next:#?}"),
            ));
        };

        let args = ArgsTyped::try_from(&mut *value).map_err(|err| err.push("Defun args"))?;

        let next = value.next().ok_or(ParserError::new(
            "Defun arrow",
            format!("Expected more tokens"),
        ))?;
        if next != Token::Keyword(Keywords::Arrow) {
            return Err(ParserError::new(
                "Defun arrow",
                format!("Expected Arrow keyword, got {next:#?}"),
            ));
        }

        let next = value.next().ok_or(ParserError::new(
            "Defun return",
            format!("Expected more tokens"),
        ))?;
        let return_type = if let Token::Type(ret) = next {
            ret
        } else {
            return Err(ParserError::new(
                "Defun return",
                format!("Expected Arrow keyword, got {next:#?}"),
            ));
        };

        let body = Exp::try_from(&mut *value).map_err(|err| err.push("Defun body"))?;

        Ok(Defun {
            scope,
            name,
            args,
            return_type,
            body,
        })
    }
}

#[derive(Debug)]
enum Scope {
    File,
    Crate,
    Full,
}

impl TryFrom<&mut Parser> for Scope {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value.peek().ok_or(ParserError::new("Scope", format!("Expected more tokens")))?;
        match next {
            Token::ParenOpen => {
                value.next();
                let next = value.next().ok_or(ParserError::new("Scope", format!("Expected more tokens")))?;
                if let Token::Identifier(iden) = next {
                    if &iden[..] != "pub" {
                        return Err(ParserError::new("Scope iden", format!("Expected pub, got {iden:#?}")));
                    }

                    let peek = value.peek().ok_or(ParserError::new("Scope crate", format!("Expected more tokens")))?;
                    if *peek == Token::ParenClose {
                        Ok(Scope::Full)
                    } else if let Token::Identifier(iden) = value.next().unwrap() {
                        if &iden[..] == "crate" {
                            Ok(Scope::Crate)
                        } else {
                            Err(ParserError::new("Scope crate", format!("Expected crate, got {iden:#?}")))
                        }
                    } else {
                        Err(ParserError::new("Scope", format!("Expected identifier, got {:#?}", value.next().unwrap())))
                    }
                } else {
                    Err(ParserError::new("Scope", format!("Expected identifier, got {next:#?}")))
                }
            }
            Token::Identifier(iden) => {
                match &iden[..] {
                    "pub" => {
                        value.next();
                        Ok(Scope::Full)
                    },
                    iden => Err(ParserError::new("Scope iden", format!("Expected pub, got {iden:#?}")))
                }
            }
            _ => Err(ParserError::new("Scope", format!("Expected identifier, got {next:#?}"))),
        }
    }
}

#[derive(Debug)]
struct ArgsTyped(Vec<(String, Types)>);

impl TryFrom<&mut Parser> for ArgsTyped {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut args = vec![];

        let next = value.next().ok_or(ParserError::new("ArgsTyped OpenParen", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(ParserError::new("ArgsTyped OpenParen", format!("Expected ParenOpen, got {next:#?}")));
        }

        loop {
            let next = value.next().ok_or(ParserError::new("ArgsTyped name|CloseParen", format!("Expected more tokens")))?;
            let name = if let Token::Identifier(iden) = next {
                iden
            } else if next == Token::ParenClose {
                break;
            } else {
                return Err(ParserError::new("ArgsTyped name", format!("Expected identifier, got {next:#?}")))
            };

            let next = value.next().ok_or(ParserError::new("ArgsTyped arrow", format!("Expected more tokens")))?;
            if next != Token::Keyword(Keywords::Arrow) {
                return Err(ParserError::new("ArgsTyped arrow", format!("Expected Arrow keyword, got {next:#?}")))
            }

            let next = value.next().ok_or(ParserError::new("ArgsTyped type", format!("Expected more tokens")))?;
            let arg_type = if let Token::Type(arg_type) = next {
                arg_type
            } else {
                return Err(ParserError::new("ArgsTyped arrow", format!("Expected Arrow keyword, got {next:#?}")));
            };

            args.push((name, arg_type))
        }

        Ok(ArgsTyped(args))
    }
}

#[derive(Debug)]
enum Exp {
    Call(Box<Call>),
    If(Box<If>),
    Match(Box<Match>),
    Lambda(Box<Lambda>),
    Literal(Literals),
    Identifier(String),
}

impl TryFrom<&mut Parser> for Exp {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let peek = value
            .peek()
            .ok_or(ParserError::new("Exp", format!("Expected more tokens")))?;

        match *peek {
            Token::ParenOpen => {
                match value.peek_nth(1).ok_or(ParserError::new(
                    "Exp paren",
                    format!("Expected more tokens"),
                ))? {
                    Token::Keyword(Keywords::If) => Ok(If::try_from(&mut *value)
                        .map(|v| Exp::If(Box::new(v)))
                        .map_err(|err| err.push("Exp if"))?),
                    Token::Keyword(Keywords::Match) => Ok(Match::try_from(&mut *value)
                        .map(|v| Exp::Match(Box::new(v)))
                        .map_err(|err| err.push("Exp match"))?),
                    Token::Keyword(Keywords::Lambda) => Ok(Lambda::try_from(&mut *value)
                        .map(|v| Exp::Lambda(Box::new(v)))
                        .map_err(|err| err.push("Exp lambda"))?),
                    Token::Literal(_) => {
                        value.next();
                        Ok(Exp::Literal(
                            if let Token::Literal(literal) = value.next().unwrap() {
                                literal
                            } else {
                                panic!("wtf")
                            },
                        ))
                    }
                    Token::Identifier(_) => {
                        value.next();
                        let func = if let Token::Identifier(iden) = value.next().unwrap() {
                            iden
                        } else {
                            panic!("wtf")
                        };
                        let mut params = vec![];

                        loop {
                            let peek = value.peek().ok_or(ParserError::new(
                                "Exp call/iden",
                                format!("Expected more tokens"),
                            ))?;

                            if *peek == Token::ParenClose {
                                value.next();
                                break;
                            } else {
                                params.push(
                                    Exp::try_from(&mut *value)
                                        .map_err(|err| err.push("Exp call/iden"))?,
                                )
                            }
                        }

                        Ok(Exp::Call(Box::new(Call {
                            func: Either::Right(func),
                            params,
                        })))
                    }
                    Token::ParenOpen => {
                        value.next();
                        let lambda = Lambda::try_from(&mut *value)
                            .map_err(|err| err.push("Exp call/lambda"))?;
                        let mut params = vec![];

                        loop {
                            let peek = value.peek().ok_or(ParserError::new(
                                "Exp call/iden",
                                format!("Expected more tokens"),
                            ))?;

                            if *peek == Token::ParenClose {
                                value.next();
                                break;
                            } else {
                                params.push(
                                    Exp::try_from(&mut *value)
                                        .map_err(|err| err.push("Exp call/iden"))?,
                                )
                            }
                        }

                        Ok(Exp::Call(Box::new(Call {
                            func: Either::Left(lambda),
                            params,
                        })))
                    }
                    _ => unreachable!(),
                }
            }
            Token::Identifier(_) => {
                let iden = value.next().unwrap();
                if let Token::Identifier(iden) = iden {
                    Ok(Exp::Identifier(iden))
                } else {
                    panic!("how")
                }
            }
            _ => {
                let next = value.next().unwrap();
                if let Token::Literal(literal) = next {
                    Ok(Exp::Literal(literal))
                } else {
                    Err(ParserError::new(
                        "Exp literal",
                        format!("Expected a literal, got {next:#?}"),
                    ))
                }
            }
        }
    }
}

#[derive(Debug)]
struct Call {
    func: Either<Lambda, String>,
    params: Vec<Exp>,
}

#[derive(Debug)]
struct If {
    condition: Exp,
    true_branch: Exp,
    false_branch: Exp,
}

impl TryFrom<&mut Parser> for If {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(ParserError::new("If", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(ParserError::new(
                "If",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value
            .next()
            .ok_or(ParserError::new("If", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::If) {
            return Err(ParserError::new(
                "If",
                format!("Expected If keyword, got {next:#?}"),
            ));
        }

        let condition = Exp::try_from(&mut *value).map_err(|err| err.push("If condition"))?;
        let true_branch = Exp::try_from(&mut *value).map_err(|err| err.push("If true"))?;
        let false_branch = Exp::try_from(&mut *value).map_err(|err| err.push("If false"))?;

        Ok(Self {
            condition,
            true_branch,
            false_branch,
        })
    }
}

#[derive(Debug)]
struct Match {
    against: Exp,
    branches: Vec<Branch>,
}

impl TryFrom<&mut Parser> for Match {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(ParserError::new("Match", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(ParserError::new(
                "Match",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value
            .next()
            .ok_or(ParserError::new("Match", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Match) {
            return Err(ParserError::new(
                "Match",
                format!("Expected Match keyword, got {next:#?}"),
            ));
        }

        let against = Exp::try_from(&mut *value).map_err(|err| err.push("Match against"))?;
        let mut branches = vec![];

        loop {
            let peek = value.peek().ok_or(ParserError::new(
                "Match branches",
                format!("Expected more tokens"),
            ))?;

            if *peek == Token::ParenClose {
                value.next();
                break;
            } else {
                branches
                    .push(Branch::try_from(&mut *value).map_err(|err| err.push("Match branch"))?);
            }
        }

        Ok(Match { against, branches })
    }
}

#[derive(Debug)]
struct Branch {
    pattern: PatternTree,
    check: Option<Exp>,
    ret: Exp,
}

impl TryFrom<&mut Parser> for Branch {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(ParserError::new("Branch", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(ParserError::new(
                "Branch",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value.next().ok_or(ParserError::new(
            "Branch single?",
            format!("Expected more tokens"),
        ))?;
        let ret = match next {
            Token::Literal(literal) => Ok(Branch {
                pattern: PatternTree::new(PatternToken::Literal(literal)),
                check: None,
                ret: Exp::try_from(&mut *value).map_err(|err| err.push("Branch literal"))?,
            }),
            Token::Identifier(iden) => Ok(Branch {
                pattern: PatternTree::new(PatternToken::Identifier(iden)),
                check: None,
                ret: Exp::try_from(&mut *value).map_err(|err| err.push("Branch literal"))?,
            }),
            Token::ParenOpen => {
                let mut tokens = vec![];

                while !matches!(
                    value.peek(),
                    Some(Token::ParenClose | Token::Keyword(Keywords::If))
                ) {
                    tokens.push(value.next().unwrap())
                }

                let pattern = PatternTree::try_from(&mut Parser {
                    tokens: tokens.into_iter().peekable_nth(),
                })
                .map_err(|err| err.push("Branch paren"))?;

                let next = value.next().ok_or(ParserError::new(
                    "Branch if?",
                    format!("Expected more tokens"),
                ))?;
                match next {
                    Token::Keyword(Keywords::If) => {
                        let check = Exp::try_from(&mut *value)
                            .map_err(|err| err.push("Branch paren/if-condition"))?;
                        let next = value.next().ok_or(ParserError::new(
                            "Branch paren/if/close",
                            format!("Expected more tokens"),
                        ))?;
                        if next != Token::ParenClose {
                            return Err(ParserError::new(
                                "Branch paren/if/close",
                                format!("Expected ParenClose, got {next:#?}"),
                            ));
                        }
                        let ret = Ok(Branch {
                            pattern,
                            check: Some(check),
                            ret: Exp::try_from(&mut *value)
                                .map_err(|err| err.push("Branch paren/if-body"))?,
                        });

                        ret
                    }
                    Token::ParenClose => Ok(Branch {
                        pattern,
                        check: None,
                        ret: Exp::try_from(&mut *value)
                            .map_err(|err| err.push("Branch paren/close"))?,
                    }),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };

        let next = value.next().ok_or(ParserError::new(
            "Branch close",
            format!("Expected more tokens"),
        ))?;
        if next != Token::ParenClose {
            Err(ParserError::new(
                "Branch close",
                format!("Expected ParenClose, got {next:#?}"),
            ))
        } else {
            ret
        }
    }
}

#[derive(Debug)]
struct Lambda {
    args: Args,
    body: Exp,
}

impl TryFrom<&mut Parser> for Lambda {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(ParserError::new("Lambda", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(ParserError::new(
                "Lambda",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value
            .next()
            .ok_or(ParserError::new("Lambda", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Lambda) {
            return Err(ParserError::new(
                "Lambda",
                format!("Expected Lambda Keyword, got {next:#?}"),
            ));
        }

        let args = Args::try_from(&mut *value).map_err(|err| err.push("Lambda args"))?;
        let body = Exp::try_from(&mut *value).map_err(|err| err.push("Lambda body"))?;

        let next = value
            .next()
            .ok_or(ParserError::new("Lambda", format!("Expected more tokens")))?;
        if next != Token::ParenClose {
            return Err(ParserError::new(
                "Lambda",
                format!("Expected ParenClose, got {next:#?}"),
            ));
        }

        Ok(Lambda { args, body })
    }
}

#[derive(Debug)]
struct Args(Vec<String>);

impl TryFrom<&mut Parser> for Args {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(ParserError::new("Args", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(ParserError::new(
                "Args",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let mut args = vec![];
        loop {
            let token = value.next().ok_or(ParserError::new(
                "Args",
                format!("Expected ParenClose, got nothing"),
            ))?;

            if token == Token::ParenClose {
                break;
            } else if let Token::Identifier(iden) = token {
                args.push(iden)
            } else {
                return Err(ParserError::new(
                    "Args",
                    format!("Expected ParenClose or Identifier, got {token:#?}"),
                ));
            }
        }

        Ok(Self(args))
    }
}

#[derive(Debug)]
pub struct ParserError {
    stack: Vec<&'static str>,
    err: String,
}

impl ParserError {
    fn new(initial: &'static str, err: String) -> Self {
        Self {
            stack: vec![initial],
            err,
        }
    }

    fn push(mut self, func: &'static str) -> Self {
        self.stack.push(func);
        self
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: PeekableNth<std::vec::IntoIter<Token>>,
}

impl Parser {
    pub fn parse(tokens: Tokens) -> Result<(), ParserError> {
        let mut parser = Self {
            tokens: tokens.0.into_iter().peekable_nth(),
        };

        println!("{:#?}", Defun::try_from(&mut parser)?);
        Ok(())
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn peek_nth(&mut self, nth: usize) -> Option<&Token> {
        self.tokens.peek_nth(nth)
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }
}
