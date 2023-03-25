use clap::{Parser, Subcommand};
use list::{parser::Parser as lP, tokenizer::Tokens};

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
    },
    Token {
        path: String,
    },
}

#[derive(Subcommand, Debug)]
enum ParserType {
    File { path: String },
    Enum { path: String },
    Struct { path: String },
    Function { path: String },
    Expression { path: String },
}

impl ParserType {
    fn from_tokens_to_string(&self, file: bool) -> String {
        let tokens = input(file, self.get_path());
        match self {
            Self::File { .. } => lP::file(tokens),
            Self::Enum { .. } => lP::r#enum(tokens),
            Self::Struct { .. } => lP::r#struct(tokens),
            Self::Function { .. } => lP::r#function(tokens),
            Self::Expression { .. } => lP::r#expression(tokens),
        }
    }

    fn get_path(&self) -> &String {
        match self {
            Self::File { path } | Self::Enum { path } | Self::Struct { path } | Self::Function { path } | Self::Expression { path } => path,
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.token_or_parser {
        TokenOrParser::Parser { parsertype } => println!("{}", parsertype.from_tokens_to_string(args.file)),
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
