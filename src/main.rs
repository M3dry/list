use clap::{Parser, Subcommand};
use list::{parser::{Parser as lP, args::{Args as aArgs, ArgsTyped}, r#type::Type, file::File, r#enum::Enum, r#struct::Struct, r#match::{Match, Pattern}, defun::Defun, exp::Exp, r#use::Use, lambda::Lambda}, tokenizer::Tokens};

macro_rules! tostrrr {
    ($type:tt, $str:ident, $parser:ident) => {{
        match $type::try_from(&mut $parser) {
            Ok(res) if $str => res.to_string(),
            Err(res) => format!("{res}"),
            res => format!("{res:#?}"),
        }
    }};
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: bool,
    #[clap(subcommand)]
    token_or_parser: TokenOrParser,
}

#[derive(Subcommand, Debug)]
enum TokenOrParser {
    Parser {
        #[clap(subcommand)]
        parsertype: ParserType,
        #[arg(short, long)]
        to_string: bool,
    },
    Token {
        path: String,
    },
}

#[derive(Subcommand, Debug)]
enum ParserType {
    ArgsTyped { path: String },
    Args { path: String },
    Type { path: String },
    File { path: String },
    Enum { path: String },
    Struct { path: String },
    Use { path: String },
    Match { path: String },
    Function { path: String },
    Expression { path: String },
    Lambda { path: String },
    Pattern { path: String },
}

impl ParserType {
    fn from_tokens_to_string(&self, file: bool, to_string: bool) -> String {
        let tokens = input(file, self.get_path());
        let mut parser = lP::new(tokens);
        match self {
            Self::ArgsTyped { .. } => tostrrr!(ArgsTyped, to_string, parser),
            Self::Args { .. } => tostrrr!(aArgs, to_string, parser),
            Self::Type { .. } => tostrrr!(Type, to_string, parser),
            Self::File { .. } => tostrrr!(File, to_string, parser),
            Self::Enum { .. } => tostrrr!(Enum, to_string, parser),
            Self::Use { .. } => tostrrr!(Use, to_string, parser),
            Self::Struct { .. } => tostrrr!(Struct, to_string, parser),
            Self::Match { .. } => tostrrr!(Match, to_string, parser),
            Self::Function { .. } => tostrrr!(Defun, to_string, parser),
            Self::Expression { .. } => tostrrr!(Exp, to_string, parser),
            Self::Lambda { .. } => tostrrr!(Lambda, to_string, parser),
            Self::Pattern { .. } => tostrrr!(Pattern, to_string, parser),
        }
    }

    fn get_path(&self) -> &String {
        match self {
            Self::ArgsTyped { path }
            | Self::Args { path }
            | Self::Type { path }
            | Self::File { path }
            | Self::Enum { path }
            | Self::Use { path }
            | Self::Struct { path }
            | Self::Match { path }
            | Self::Function { path }
            | Self::Expression { path }
            | Self::Lambda { path }
            | Self::Pattern { path } => path,
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.token_or_parser {
        TokenOrParser::Parser {
            parsertype,
            to_string,
        } => println!("{}", parsertype.from_tokens_to_string(args.file, to_string)),
        TokenOrParser::Token { path } => println!("{:#?}", input(args.file, &path)),
    }
}

fn input(file: bool, path: &String) -> Tokens {
    if file {
        std::fs::read_to_string(&path).unwrap().parse().unwrap()
    } else {
        path.parse().unwrap()
    }
}
