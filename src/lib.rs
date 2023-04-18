use std::path::PathBuf;

use parser::{from_file, ParserError};

pub mod tokenizer;
pub mod parser;

pub fn build_get_files() -> Result<Vec<(String, PathBuf)>, ParserError> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let list_dir = std::env::var("LIST_DIR").expect("The env var 'LIST_DIR' needs to be set") + "/";
    let mut files = vec![];

    for path in std::fs::read_dir(&list_dir).unwrap() {
        let mut path = path.unwrap().path();
        let file = from_file(&path);
        path.set_extension("rs");
        path = PathBuf::from(
            out_dir.to_string()
                + "/"
                + &path.to_str().unwrap()[list_dir.len()..],
        );

        files.push((file, path))
    }

    Ok(files)
}
