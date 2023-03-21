use either::Either;
use peek_nth::{IteratorExt, PeekableNth};

use crate::pattern::{PatternToken, PatternTree};
use crate::tokenizer::{BuiltinTypes, Keywords, Literals, Token, Tokens};

macro_rules! error {
    ($initial:literal, $err:expr$(,)?) => {
        ParserError {
            stack: vec![ParserErrorStack {
                name: $initial,
                location: (line!(), column!()),
            }],
            err: $err,
        }
    };
    ($error:expr, $name:literal) => {
        $error.map_err(|mut err| {
            err.stack.push(ParserErrorStack {
                name: $name,
                location: (line!(), column!()),
            });
            err
        })
    };
}

#[derive(Debug)]
struct File {
    structs: Vec<Struct>,
    functions: Vec<Defun>,
}

impl TryFrom<&mut Parser> for File {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut functions = vec![];
        let mut structs = vec![];

        loop {
            let peek = value.peek_nth(1);

            if peek.is_none() {
                break;
            }

            match peek {
                Some(Token::Keyword(Keywords::Defun)) => {
                    functions.push(error!(Defun::try_from(&mut *value), "File")?)
                }
                Some(Token::Keyword(Keywords::Struct)) => {
                    structs.push(error!(Struct::try_from(&mut *value), "File")?)
                }
                token => {
                    return Err(error!(
                        "Defun",
                        format!("Expected a Defun keyword or a typedef, got {token:#?}")
                    ))
                }
            }
        }

        Ok(File { structs, functions })
    }
}

#[derive(Debug)]
struct Struct {
    name: String,
    generics: Vec<String>,
    fields: StructFields,
}

impl TryFrom<&mut Parser> for Struct {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("Struct", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Struct",
                format!("Expected ParenOpen, got {next:#?}")
            ));
        }

        let next = value
            .next()
            .ok_or(error!("Struct", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Struct) {
            return Err(error!(
                "Struct",
                format!("Expected Struct keyword, got {next:#?}")
            ));
        }

        let next = value
            .next()
            .ok_or(error!("Struct", format!("Expected more tokens")))?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(error!(
                "Struct",
                format!("Expected identifier, got {next:#?}")
            ));
        };

        let mut generics = vec![];

        loop {
            let peek = value
                .peek()
                .ok_or(error!("Struct", format!("Expected more tokens")))?;

            if let Token::Generic(_) = peek {
                generics.push(if let Token::Generic(generic) = value.next().unwrap() {
                    generic
                } else {
                    panic!("wtf")
                })
            } else {
                break;
            }
        }

        let fields = error!(StructFields::try_from(&mut *value), "Struct")?;

        let next = value
            .next()
            .ok_or(error!("Struct", format!("Expected more tokens")))?;
        if next != Token::ParenClose {
            return Err(error!(
                "Struct",
                format!("Expected ParenClose, got {next:#?}")
            ));
        }

        Ok(Struct {
            name,
            generics,
            fields,
        })
    }
}

#[derive(Debug)]
struct StructFields(Vec<StructField>);

impl TryFrom<&mut Parser> for StructFields {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("StructFields", format!("Expected more tokens")))?;
        if next != Token::CurlyOpen {
            return Err(error!(
                "StructFields",
                format!("Expected CurlyOpen, got {next:#?}")
            ));
        }

        let mut fields = vec![];

        loop {
            let peek = value
                .peek()
                .ok_or(error!("StructFields", format!("Expected more tokens")))?;
            if peek == &Token::CurlyClose {
                value.next();
                break;
            }

            fields.push(error!(StructField::try_from(&mut *value), "StructFields")?);
        }

        Ok(StructFields(fields))
    }
}

#[derive(Debug)]
struct StructField {
    name: String,
    r#type: Type,
}

impl TryFrom<&mut Parser> for StructField {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("StructField", format!("Expected more tokens")))?;
        if let Token::Identifier(iden) = next {
            let name = iden;

            let next = value
                .next()
                .ok_or(error!("StructField", format!("Expected more tokens")))?;
            if next != Token::Keyword(Keywords::Arrow) {
                return Err(error!(
                    "StructField",
                    format!("Expected arror keyword, got {next:#?}")
                ))?;
            }

            let r#type = error!(Type::try_from(&mut *value), "StructField")?;

            Ok(StructField { name, r#type })
        } else {
            Err(error!(
                "StructField",
                format!("Expected identifier, got {next:#?}")
            ))
        }
    }
}

#[derive(Debug)]
struct Enum {
    name: String,
    generics: Vec<String>,
    variants: Vec<Variant>,
}

impl TryFrom<&mut Parser> for Enum {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("Enum", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!("Enum", format!("Expected ParenOpen, got {next:#?}")));
        }

        let next = value
            .next()
            .ok_or(error!("Enum", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Enum) {
            return Err(error!(
                "Enum",
                format!("Expected Enum keyword, got {next:#?}")
            ));
        }

        let next = value
            .next()
            .ok_or(error!("Enum", format!("Expected more tokens")))?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(error!(
                "Enum",
                format!("Expected identifier, got {next:#?}")
            ));
        };

        let mut generics = vec![];

        loop {
            let peek = value
                .peek()
                .ok_or(error!("Enum", format!("Expected more tokens")))?;

            if let Token::Generic(_) = peek {
                generics.push(if let Token::Generic(generic) = value.next().unwrap() {
                    generic
                } else {
                    panic!("wtf")
                })
            } else {
                break;
            }
        }

        let mut variants = vec![];

        loop {
            let peek = value
                .peek()
                .ok_or(error!("Enum", format!("Expected more tokens")))?;
            if peek == &Token::ParenClose {
                value.next();
                break;
            }

            variants.push(error!(Variant::try_from(&mut *value), "Enum")?);
        }

        Ok(Self {
            name,
            generics,
            variants,
        })
    }
}

#[derive(Debug)]
enum Variant {
    Simple(String),
    WithType(String, Vec<Type>),
    Struct(String, StructFields),
}

impl TryFrom<&mut Parser> for Variant {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
struct Defun {
    scope: Scope,
    name: String,
    args: ArgsTyped,
    return_type: Type,
    body: Exp,
}

impl TryFrom<&mut Parser> for Defun {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("Defun", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Defun",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let scope = Scope::try_from(&mut *value).unwrap_or(Scope::File);

        let next = value
            .next()
            .ok_or(error!("Defun", format!("Expected more tokens"),))?;
        if next != Token::Keyword(Keywords::Defun) {
            return Err(error!(
                "Defun",
                format!("Expected Defun keyword, got {next:#?}"),
            ));
        }

        let next = value
            .next()
            .ok_or(error!("Defun name", format!("Expected more tokens"),))?;
        let name = if let Token::Identifier(iden) = next {
            iden
        } else {
            return Err(error!(
                "Defun name",
                format!("Expected Identifier, got {next:#?}"),
            ));
        };

        let args = error!(ArgsTyped::try_from(&mut *value), "Defun")?;

        let next = value
            .next()
            .ok_or(error!("Defun arrow", format!("Expected more tokens"),))?;
        if next != Token::Keyword(Keywords::Arrow) {
            return Err(error!(
                "Defun arrow",
                format!("Expected Arrow keyword, got {next:#?}"),
            ));
        }

        let return_type = error!(Type::try_from(&mut *value), "Defun")?;
        let body = error!(Exp::try_from(&mut *value), "Defun")?;

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
        let next = value
            .peek()
            .ok_or(error!("Scope", format!("Expected more tokens")))?;
        match next {
            Token::ParenOpen => {
                value.next();
                let next = value
                    .next()
                    .ok_or(error!("Scope", format!("Expected more tokens")))?;
                if let Token::Identifier(iden) = next {
                    if &iden[..] != "pub" {
                        return Err(error!("Scope iden", format!("Expected pub, got {iden:#?}"),));
                    }

                    let peek = value
                        .peek()
                        .ok_or(error!("Scope crate", format!("Expected more tokens"),))?;
                    if *peek == Token::ParenClose {
                        Ok(Scope::Full)
                    } else if let Token::Identifier(iden) = value.next().unwrap() {
                        if &iden[..] == "crate" {
                            Ok(Scope::Crate)
                        } else {
                            Err(error!(
                                "Scope crate",
                                format!("Expected crate, got {iden:#?}"),
                            ))
                        }
                    } else {
                        Err(error!(
                            "Scope",
                            format!("Expected identifier, got {:#?}", value.next().unwrap()),
                        ))
                    }
                } else {
                    Err(error!(
                        "Scope",
                        format!("Expected identifier, got {next:#?}"),
                    ))
                }
            }
            Token::Identifier(iden) => match &iden[..] {
                "pub" => {
                    value.next();
                    Ok(Scope::Full)
                }
                iden => Err(error!("Scope iden", format!("Expected pub, got {iden:#?}"),)),
            },
            _ => Err(error!(
                "Scope",
                format!("Expected identifier, got {next:#?}"),
            )),
        }
    }
}

#[derive(Debug)]
enum Type {
    Builtin(BuiltinTypes),
    Generic(String),
    Custom(String),
    Complex(String, Vec<Type>),
}

impl TryFrom<&mut Parser> for Type {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("Type", format!("Expected more tokens")))?;
        match next {
            Token::Type(builtin) => Ok(Type::Builtin(builtin)),
            Token::Generic(generic) => Ok(Type::Generic(generic)),
            Token::Identifier(iden) => Ok(Type::Custom(iden)),
            Token::ParenOpen => {
                let next = value
                    .next()
                    .ok_or(error!("Type complex", format!("Expected more tokens"),))?;
                match next {
                    Token::Type(builtin) => {
                        if value.next() != Some(Token::ParenClose) {
                            Err(error!(
                                "Type complex/builtin",
                                format!("Expected an identifier, got a builtin type"),
                            ))
                        } else {
                            Ok(Type::Builtin(builtin))
                        }
                    }
                    Token::Identifier(iden) => {
                        let mut types = vec![];

                        while value.peek() != Some(&Token::ParenClose) {
                            types.push(error!(Type::try_from(&mut *value), "Type")?);
                        }
                        value.next();

                        Ok(Type::Complex(iden, types))
                    }
                    next => Err(error!(
                        "Type complex/other",
                        format!("Expected an identifier, got {next:#?}"),
                    )),
                }
            }
            _ => Err(error!(
                "Type",
                format!("Expected type, indentifier or OpenParen, got {next:#?}"),
            )),
        }
    }
}

#[derive(Debug)]
enum Exp {
    Call(Box<Call>),
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
            .peek()
            .ok_or(error!("Exp", format!("Expected more tokens")))?;

        match *peek {
            Token::ParenOpen => {
                match value
                    .peek_nth(1)
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
                            let peek = value
                                .peek()
                                .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

                            if *peek == Token::ParenClose {
                                value.next();
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
                    Token::ParenOpen => {
                        value.next();
                        let lambda = error!(Lambda::try_from(&mut *value), "Exp call/lambda")?;
                        let mut params = vec![];

                        loop {
                            let peek = value
                                .peek()
                                .ok_or(error!("Exp call/iden", format!("Expected more tokens"),))?;

                            if *peek == Token::ParenClose {
                                value.next();
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
                    Err(error!(
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
            .ok_or(error!("If", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!("If", format!("Expected ParenOpen, got {next:#?}"),));
        }

        let next = value
            .next()
            .ok_or(error!("If", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::If) {
            return Err(error!("If", format!("Expected If keyword, got {next:#?}"),));
        }

        let condition = error!(Exp::try_from(&mut *value), "If")?;
        let true_branch = error!(Exp::try_from(&mut *value), "If")?;
        let false_branch = error!(Exp::try_from(&mut *value), "If")?;

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
            .ok_or(error!("Match", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Match",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value
            .next()
            .ok_or(error!("Match", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Match) {
            return Err(error!(
                "Match",
                format!("Expected Match keyword, got {next:#?}"),
            ));
        }

        let against = error!(Exp::try_from(&mut *value), "Match")?;
        let mut branches = vec![];

        loop {
            let peek = value
                .peek()
                .ok_or(error!("Match branches", format!("Expected more tokens"),))?;

            if *peek == Token::ParenClose {
                value.next();
                break;
            } else {
                branches.push(error!(Branch::try_from(&mut *value), "Match")?);
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
            .ok_or(error!("Branch", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Branch",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value
            .next()
            .ok_or(error!("Branch single?", format!("Expected more tokens"),))?;
        let ret = match next {
            Token::Literal(literal) => Ok(Branch {
                pattern: PatternTree::new(PatternToken::Literal(literal)),
                check: None,
                ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
            }),
            Token::Identifier(iden) => Ok(Branch {
                pattern: PatternTree::new(PatternToken::Identifier(iden)),
                check: None,
                ret: error!(Exp::try_from(&mut *value), "Branch literal")?,
            }),
            Token::ParenOpen => {
                let mut tokens = vec![];

                while !matches!(
                    value.peek(),
                    Some(Token::ParenClose | Token::Keyword(Keywords::If))
                ) {
                    tokens.push(value.next().unwrap())
                }

                let pattern = error!(
                    PatternTree::try_from(&mut Parser {
                        tokens: tokens.into_iter().peekable_nth(),
                    }),
                    "Branch"
                )?;

                let next = value
                    .next()
                    .ok_or(error!("Branch if?", format!("Expected more tokens"),))?;
                match next {
                    Token::Keyword(Keywords::If) => {
                        let check =
                            error!(Exp::try_from(&mut *value), "Branch paren/if-condition")?;
                        let next = value.next().ok_or(error!(
                            "Branch paren/if/close",
                            format!("Expected more tokens"),
                        ))?;
                        if next != Token::ParenClose {
                            return Err(error!(
                                "Branch paren/if/close",
                                format!("Expected ParenClose, got {next:#?}"),
                            ));
                        }
                        let ret = Ok(Branch {
                            pattern,
                            check: Some(check),
                            ret: error!(Exp::try_from(&mut *value), "Branch")?,
                        });

                        ret
                    }
                    Token::ParenClose => Ok(Branch {
                        pattern,
                        check: None,
                        ret: error!(Exp::try_from(&mut *value), "Branch")?,
                    }),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };

        let next = value
            .next()
            .ok_or(error!("Branch close", format!("Expected more tokens"),))?;
        if next != Token::ParenClose {
            Err(error!(
                "Branch close",
                format!("Expected ParenClose, got {next:#?}"),
            ))
        } else {
            ret
        }
    }
}

#[derive(Debug)]
struct Let {
    vars: Vec<(String, Exp)>,
    body: Exp,
}

impl TryFrom<&mut Parser> for Let {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("Let", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!("Let", format!("Expected ParenOpen, got {next:#?}"),));
        }

        let next = value
            .next()
            .ok_or(error!("Let", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Let) {
            return Err(error!(
                "Let keyword",
                format!("Expected Let keyword, got {next:#?}"),
            ));
        }

        let mut vars = vec![];

        if let Token::Identifier(_) = value
            .peek_nth(1)
            .ok_or(error!("Let var", format!("Expected more tokens")))?
        {
            value.next();
            let iden = if let Token::Identifier(iden) = value.next().unwrap() {
                iden
            } else {
                panic!("this shouldn't happen")
            };

            vars.push((iden, Exp::try_from(&mut *value)?));
            value.next();
        } else if value.next().unwrap() == Token::ParenOpen {
            loop {
                let next = value
                    .next()
                    .ok_or(error!("Let vars", format!("Expected more tokens"),))?;
                if next == Token::ParenClose {
                    break;
                }
                if next != Token::ParenOpen {
                    return Err(error!(
                        "Let vars",
                        format!("Expected ParenOpen, got {next:#?}"),
                    ));
                }

                let next = value
                    .next()
                    .ok_or(error!("Let vars", format!("Expected more tokens"),))?;
                let iden = if let Token::Identifier(iden) = next {
                    iden
                } else {
                    return Err(error!(
                        "Let vars/name",
                        format!("Expected identifier, got {next:#?}"),
                    ));
                };

                let exp = Exp::try_from(&mut *value)?;

                let next = value
                    .next()
                    .ok_or(error!("Let vars", format!("Expected more tokens"),))?;
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
            .ok_or(error!("Lambda", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!(
                "Lambda",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        let next = value
            .next()
            .ok_or(error!("Lambda", format!("Expected more tokens")))?;
        if next != Token::Keyword(Keywords::Lambda) {
            return Err(error!(
                "Lambda",
                format!("Expected Lambda Keyword, got {next:#?}"),
            ));
        }

        let args = error!(Args::try_from(&mut *value), "Lambda")?;
        let body = error!(Exp::try_from(&mut *value), "Lambda")?;

        let next = value
            .next()
            .ok_or(error!("Lambda", format!("Expected more tokens")))?;
        if next != Token::ParenClose {
            return Err(error!(
                "Lambda",
                format!("Expected ParenClose, got {next:#?}"),
            ));
        }

        Ok(Lambda { args, body })
    }
}

#[derive(Debug)]
struct ArgsTyped {
    generics: Vec<String>,
    args: Vec<(String, Type)>,
}

impl TryFrom<&mut Parser> for ArgsTyped {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let mut args = vec![];
        let mut generics = vec![];

        let next = value.next().ok_or(error!(
            "ArgsTyped OpenParen",
            format!("Expected more tokens"),
        ))?;
        if next != Token::ParenOpen {
            return Err(error!(
                "ArgsTyped",
                format!("Expected ParenOpen, got {next:#?}"),
            ));
        }

        while let Some(Token::Generic(_)) = value.peek() {
            if let Some(Token::Generic(generic)) = value.next() {
                generics.push(generic);
            } else {
                panic!("wtf")
            }
        }

        loop {
            if value.peek() == Some(&Token::ParenClose) {
                value.next();
                break;
            }
            let arg = error!(Arg::try_from(&mut *value), "ArgsTyped")?;
            match arg {
                Arg::Generic(_) => return Err(error!("ArgsTyped", format!("Expected a named arg, got a generic, those should be defined before named args"))),
                Arg::Simple(_) => {
                    return Err(error!(
                        "ArgsTyped",
                        format!("Expected named arg, got a simple arg"),
                    ))
                }
                Arg::Named(name, arg_type) => args.push((name, arg_type)),
            }
        }

        Ok(ArgsTyped { generics, args })
    }
}

#[derive(Debug)]
struct Args(Vec<String>);

impl TryFrom<&mut Parser> for Args {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("Args", format!("Expected more tokens")))?;
        if next != Token::ParenOpen {
            return Err(error!("Args", format!("Expected ParenOpen, got {next:#?}"),));
        }

        let mut args = vec![];
        loop {
            if value.peek() == Some(&Token::ParenClose) {
                value.next();
                break;
            }
            let arg = error!(Arg::try_from(&mut *value), "Args")?;
            match arg {
                Arg::Simple(name) => args.push(name),
                _ => {
                    return Err(error!(
                        "Args arg",
                        format!("Expected just simple and generic args"),
                    ))
                }
            }
        }

        Ok(Self(args))
    }
}

#[derive(Debug)]
enum Arg {
    Generic(String),
    Named(String, Type),
    Simple(String),
}

impl TryFrom<&mut Parser> for Arg {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        let next = value
            .next()
            .ok_or(error!("Arg", format!("Expected more tokens")))?;
        let name = match next {
            Token::Identifier(iden) => iden,
            Token::Generic(generic) => return Ok(Arg::Generic(generic)),
            _ => {
                return Err(error!(
                    "Arg name",
                    format!("Expected identifier, got {next:#?}"),
                ));
            }
        };

        let next = value.peek();
        if next != Some(&Token::Keyword(Keywords::Arrow)) {
            return Ok(Arg::Simple(name));
        }
        value.next();

        let arg_type = error!(Type::try_from(&mut *value), "Arg type")?;

        Ok(Arg::Named(name, arg_type))
    }
}

#[derive(Debug)]
pub struct ParserError {
    stack: Vec<ParserErrorStack>,
    err: String,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.err)?;
        for stack in &self.stack {
            write!(f, "{stack}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ParserErrorStack {
    name: &'static str,
    location: (u32, u32),
}

impl std::fmt::Display for ParserErrorStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({};{})", self.name, self.location.0, self.location.0)
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: PeekableNth<std::vec::IntoIter<Token>>,
}

impl Parser {
    // pub fn parse(tokens: Tokens, exp: bool) -> Result<(), ParserError> {
    //     let mut parser = Self {
    //         tokens: tokens.0.into_iter().peekable_nth(),
    //     };

    //     if exp {
    //         println!("{:#?}", Exp::try_from(&mut parser));
    //     } else {
    //         println!("{:#?}", File::try_from(&mut parser));
    //     }
    //     Ok(())
    // }

    pub fn file(tokens: Tokens) -> String {
        format!("{:#?}", File::try_from(&mut Self::new(tokens)))
    }

    pub fn r#struct(tokens: Tokens) -> String {
        format!("{:#?}", Struct::try_from(&mut Self::new(tokens)))
    }

    pub fn function(tokens: Tokens) -> String {
        format!("{:#?}", Defun::try_from(&mut Self::new(tokens)))
    }

    pub fn expression(tokens: Tokens) -> String {
        format!("{:#?}", Exp::try_from(&mut Self::new(tokens)))
    }

    fn new(tokens: Tokens) -> Self {
        Self {
            tokens: tokens.0.into_iter().peekable_nth(),
        }
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
