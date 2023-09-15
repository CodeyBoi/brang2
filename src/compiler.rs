use std::collections::HashMap;

use crate::{
    parser::{parse, Expr, Program},
    tokenizer::tokenize,
};

pub(crate) struct Compiler {
    program: Program,
    ptr: isize,
    stack_ptr: isize,
    output: Vec<char>,
    name_table: HashMap<String, usize>,
}

impl Compiler {
    fn new(program: Program) -> Self {
        Self {
            program,
            ptr: 0,
            stack_ptr: 0,
            output: Vec::new(),
            name_table: HashMap::new(),
        }
    }

    fn emit(&mut self, src: &str) {
        for c in src.chars() {
            self.output.push(c);
        }
    }

    fn alloc(&mut self, size: usize) -> usize {
        let index = self.stack_ptr as usize;
        self.stack_ptr += size as isize;
        self.set_val(index, 0);
        index
    }

    fn dealloc(&mut self, size: usize) {
        self.stack_ptr -= size as isize;
    }

    fn alloc_var(&mut self, name: &str) -> usize {
        let index = self.alloc(1);
        self.name_table.insert(name.to_string(), index);
        index
    }

    fn dealloc_var(&mut self, name: &str) {
        let index = self.name_table.remove(name).unwrap();
        self.dealloc(index);
    }

    fn move_ptr(&mut self, offset: isize) {
        self.ptr += offset;
        if offset > 0 {
            for _ in 0..offset {
                self.emit(">");
            }
        } else {
            for _ in 0..(-offset) {
                self.emit("<");
            }
        }
    }

    fn set_ptr(&mut self, index: usize) {
        let offset = index as isize - self.ptr;
        self.move_ptr(offset);
    }

    fn set_val(&mut self, index: usize, value: u8) {
        self.set_ptr(index);
        self.emit("[-]");
        for _ in 0..value {
            self.emit("+");
        }
    }

    fn move_val(&mut self, src: usize, dest: usize) {
        self.set_val(dest, 0);
        self.set_ptr(src);
        self.emit("[-");
        self.set_ptr(dest);
        self.emit("+");
        self.set_ptr(src);
        self.emit("]");
    }

    fn copy_val(&mut self, src: usize, dest: usize) {
        let tmp = self.alloc(1);
        self.set_val(dest, 0);
        // Move value from src to tmp and dest
        self.set_ptr(src);
        self.emit("[-");
        self.set_ptr(dest);
        self.emit("+");
        self.set_ptr(tmp);
        self.emit("+");
        self.set_ptr(src);
        self.emit("]");
        // Move value back from tmp to src
        self.move_val(tmp, src);
        self.dealloc(1);
    }

    fn compile(&mut self) -> Result<(), String> {
        for stmt in &self.program.statements {
            use crate::parser::Statement as S;
            match stmt {
                S::FunctionDefinition { name, params, body } => {
                    todo!("Function definitions are not yet supported")
                }
                S::VariableDefinition { name, initializer } => {
                    variable_definition(self, name, initializer)?
                }
                S::Return(_) => todo!(),
                S::Print(_) => todo!(),
                S::Block(_) => todo!(),
                S::If {
                    condition,
                    then_branch,
                    else_branch,
                } => todo!(),
                S::While { condition, body } => todo!(),
            }
        }
        Ok(())
    }

    fn variable_definition(
        &mut self,
        name: &str,
        initializer: Option<Box<Expr>>,
    ) -> Result<(), String> {
        let index = self.alloc_var(name);
        if let Some(init) = initializer {
            self.expression(&init)?;
            self.copy_val(self.ptr as usize, index);
        }
        Ok(())
    }

    fn expression(&mut self, expr: &Expr) -> Result<(), String> {
        use crate::parser::Expr as E;
        match expr {
            E::Unary { op, rhs } => todo!(),
            E::Binary { lhs, op, rhs } => todo!(),
            E::Number(n) => todo!(),
            E::String(_) => todo!("Strings are not yet supported"),
            E::Identifier(_) => todo!(),
            E::Call { callee, args } => todo!(),
        }
    }
}

pub fn compile(src: &str) -> Result<String, String> {
    let tokens = tokenize(src);
    let program = parse(&tokens)?;
    let mut compiler = Compiler::new(program);
    compiler.compile()?;
    Ok("".to_string())
}
