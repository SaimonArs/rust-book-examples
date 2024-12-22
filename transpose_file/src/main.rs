use std::fs::File;
use std::io::{self, BufRead};

fn main() {
    let file = File::open("file.txt").unwrap();
    let reader = io::BufReader::new(file);

    let mut columns_line: Vec<String> = Vec::new();

    let mut iter_lines = reader.lines();

    if let Some(fi) = iter_lines.next() {
        let line = fi.unwrap();
        let fields = line.split_whitespace().map(String::from);

        for field in fields{
            columns_line.push(String::from(field));
        }
    }

    for line in iter_lines {
        let line = line.unwrap();
        let fields = line.split_whitespace().map(String::from);

        for (idx, field) in fields.enumerate() {
            columns_line[idx].push_str(format!(" {}", field).as_str());
        }
    }

    for el in columns_line{
        println!("{}", el);
    }
}
