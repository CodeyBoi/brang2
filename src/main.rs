use std::fs::read_to_string;

use compiler::compile;

mod compiler;
mod parser;
mod tokenizer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    let contents = read_to_string(filename).expect("Unable to read file");
    let compiler = compile(&contents);
}
