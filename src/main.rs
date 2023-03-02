use list::{tokenizer::Tokens, parser::Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 3 && args[1] == "-i" {
        if args.len() >= 4 && args[2] == "-p" {
            println!("{:#?}", Parser::parse(args[3].parse().unwrap()))
        } else {
            println!("{:#?}", args[2].parse::<Tokens>());
        }
    } else {
        println!("{:#?}", r#"(define multiply (i32 i32)->(Option i32)(x y) (* x y))"#.parse::<Tokens>());
    }
}
