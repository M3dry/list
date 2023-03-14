use list::{tokenizer::Tokens, parser::Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 3 && args[1] == "-i" {
        if args.len() >= 5 && args[2] == "-p" {
            let _ = match &args[3][..] {
                "-f" if args.len() >= 6 && args[4] == "-e" => Parser::parse(std::fs::read_to_string(&args[5]).unwrap().parse().unwrap(), true),
                "-f" if args.len() >= 6 && args[4] == "-e" => Parser::parse(std::fs::read_to_string(&args[5]).unwrap().parse().unwrap(), false),
                "-e" => Parser::parse(args[4].parse().unwrap(), true),
                "-d" => Parser::parse(args[4].parse().unwrap(), false),
                _ => unimplemented!()
            };
        } else if args.len() >= 4 && args[2] == "-f" {
            println!("{:#?}", std::fs::read_to_string(&args[3]).unwrap().parse::<Tokens>());
        } else {
            println!("{:#?}", args[2].parse::<Tokens>());
        }
    }
}
