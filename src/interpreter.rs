use core::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn run(filepath: &str) {
    let mut interpreter = Interpreter::new(filepath);

    while interpreter.step() {
        println!("{}\n", interpreter);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

struct Interpreter {
    memory: Vec<u8>,
    memory_ptr: usize,
    instructions: Vec<char>,
    instruction_ptr: usize,
    brackets: Vec<usize>,
    output: String,
}

fn getchar() -> Option<char> {
    std::io::stdin()
        .bytes()
        .next()
        .and_then(|res| res.ok())
        .map(|byte| byte as char)
}

impl Interpreter {
    fn new(filepath: &str) -> Self {
        let mut file = File::open(Path::new(filepath)).expect("could not open file");
        let mut code = String::new();
        file.read_to_string(&mut code)
            .expect("could not read file to string");
        let instructions: Vec<char> = code.chars().collect();
        Self {
            memory: vec![0; 1],
            memory_ptr: 0,
            instructions,
            instruction_ptr: 0,
            brackets: Vec::new(),
            output: String::new(),
        }
    }

    fn step(&mut self) -> bool {
        if self.instruction_ptr == self.instructions.len() {
            return false;
        }
        let instruction = self.instructions[self.instruction_ptr];
        match instruction {
            '>' => {
                if self.memory.len() - 1 == self.memory_ptr {
                    self.memory.push(0);
                }
                self.memory_ptr += 1;
            }
            '<' => self.memory_ptr -= 1,
            '+' => self.memory[self.memory_ptr] = self.memory[self.memory_ptr].wrapping_add(1),
            '-' => self.memory[self.memory_ptr] = self.memory[self.memory_ptr].wrapping_sub(1),
            ',' => {
                let c = loop {
                    if let Some(c) = getchar() {
                        break c;
                    }
                };
                self.memory[self.memory_ptr] = c as u8;
            }
            '.' => self.output.push(self.memory[self.memory_ptr] as char),
            '[' => {
                if self.memory[self.memory_ptr] != 0 {
                    self.brackets.push(self.instruction_ptr);
                } else {
                    let mut depth = 0;
                    loop {
                        self.instruction_ptr += 1;
                        if self.instructions[self.instruction_ptr] == ']' {
                            if depth == 0 {
                                break;
                            } else {
                                depth -= 1;
                            }
                        } else if self.instructions[self.instruction_ptr] == '[' {
                            depth += 1;
                        }
                    }
                }
            }
            ']' => {
                if self.memory[self.memory_ptr] != 0 {
                    self.instruction_ptr = *self
                        .brackets
                        .last()
                        .expect("found unmatched closing square bracket");
                } else {
                    self.brackets.pop();
                }
            }
            _ => self.instruction_ptr += 1,
        }
        self.instruction_ptr += 1;
        true
    }
}

impl fmt::Display for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const WIDTH: usize = 148;
        const DELTA: usize = 4;

        // let left_limit = if self.memory_ptr <= WIDTH / (DELTA * 2) {
        //     0
        // } else {
        //     self.memory_ptr - WIDTH / (DELTA * 2)
        // };

        write!(f, "Memory:")?;

        for (i, m) in self.memory.iter().enumerate() {
            if i % (WIDTH / (DELTA * 2)) == 0 {
                writeln!(f)?;
            }
            if i == self.memory_ptr {
                write!(f, "[{:02x}", m)?;
            } else if i == self.memory_ptr + 1 {
                write!(f, "]{:02x}", m)?;
            } else {
                write!(f, " {:02x}", m)?;
            }
        }

        if self.memory_ptr == self.memory.len() - 1 {
            write!(f, "]")?;
        }

        write!(f, "\n\nInstructions:")?;

        for (i, c) in self.instructions.iter().enumerate() {
            if i % WIDTH == 0 {
                if self.instruction_ptr >= i && self.instruction_ptr < i + WIDTH {
                    writeln!(f, "\n{:>1$}", "v", self.instruction_ptr % WIDTH + 1)?;
                } else {
                    writeln!(f)?;
                }
            }
            write!(f, "{}", c)?;
        }

        write!(f, "\n\nOutput:\n{}", self.output)
    }
}
