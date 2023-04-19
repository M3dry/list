use std::io::Write;

use list::build_get_files;

fn main() {
    let files = build_get_files().unwrap();

    for (file, path) in files {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&file.into_bytes()).unwrap();
    }
}