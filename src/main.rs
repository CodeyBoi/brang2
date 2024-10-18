use std::{
    fs::File,
    io::{BufWriter, Write},
};

use clap::{Parser, Subcommand};

mod brainfuck;
mod compiler;
mod interpreter;
mod parser;
mod tokenizer;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone)]
enum Command {
    Make {
        input: String,
        #[clap(short, long, default_value = "out.bf")]
        output: String,
    },
    Run {
        srcfile: String,
    },
    Interpret {
        srcfile: String,
    },
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Command::Make { input, output } => {
            let src = std::fs::read_to_string(&input).expect("Could not read source code file");
            let compilation = compiler::compile(&src).expect("Could not compile source code");
            let outfile = File::create(&output).expect("Could not create output file");
            Write::write_all(&mut BufWriter::new(outfile), compilation.as_bytes())
                .expect("Could not write to output file");
        }
        Command::Run { srcfile } => brainfuck::run_file(srcfile).expect("Error when running file"),
        Command::Interpret { srcfile } => interpreter::run(&srcfile),
    }
}
