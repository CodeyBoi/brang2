use std::collections::HashMap;

use crate::{
    parser::{parse, Expr, Program, Statement},
    tokenizer::tokenize,
};

pub(crate) struct Compiler {
    ptr: isize,
    stack_ptr: isize,
    output: Vec<char>,
    name_table: HashMap<String, usize>,
}

impl Compiler {
    fn new() -> Self {
        Self {
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

    /// Allocates `size` cells on the stack and returns the index of the first cell.
    /// The cells are initialized to 0.
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
        self.dadd(src, dest);
    }

    fn copy_val(&mut self, src: usize, dests: &[usize]) {
        for dest in dests {
            self.set_val(*dest, 0);
        }
        let tmp = self.alloc(1);
        // Move value from src to tmp and dest
        self.set_ptr(src);
        self.emit("[-");
        for dest in dests {
            self.set_ptr(*dest);
            self.emit("+");
        }
        self.set_ptr(tmp);
        self.emit("+");
        self.set_ptr(src);
        self.emit("]");
        // Move value back from tmp to src
        self.move_val(tmp, src);
        self.dealloc(1);
    }

    fn add(&mut self, src: usize, dest: usize) {
        let tmp = self.alloc(1);
    }

    /// Adds the value at `src` to the value at `dest` and writes it to `dest`.
    /// The value at `src` is set to 0.
    fn dadd(&mut self, src: usize, dest: usize) {
        self.set_ptr(src);
        self.emit("[-");
        self.set_ptr(dest);
        self.emit("+");
        self.set_ptr(src);
        self.emit("]");
    }

    /// Subtracts the value at `src` from the value at `dest` and writes it to `dest`.
    /// The value at `src` is set to 0.
    fn dsub(&mut self, src: usize, dest: usize) {
        self.set_ptr(src);
        self.emit("[-");
        self.set_ptr(dest);
        self.emit("-");
        self.set_ptr(src);
        self.emit("]");
    }

    // Multiplies the value at `src` with the value at `dest` and writes it to `dest`.
    // Makes the value at `src` unusable.
    fn dmul(&mut self, src: usize, dest: usize) {
        let dest_copy = self.alloc(1);
        let tmp = self.alloc(1);
        self.copy_val(dest, &[dest_copy, tmp]);
        self.set_val(dest, 0);
        self.set_ptr(src);
        self.emit("[-");
        self.set_ptr(tmp);
        self.emit("[-");
        self.set_ptr(dest);
        self.emit("+");
        self.set_ptr(tmp);
        self.emit("]");
        self.copy_val(src, dests)
    }

    fn div(&mut self, src: usize, dest: usize) {
        todo!("Division is not yet supported")
    }

    fn modulo(&mut self, src: usize, dest: usize) {
        todo!("Modulo is not yet supported")
    }

    fn eq(&mut self, src: usize, dest: usize) {
        todo!("Equality is not yet supported")
    }

    fn neq(&mut self, src: usize, dest: usize) {
        todo!("Inequality is not yet supported")
    }

    fn lt(&mut self, src: usize, dest: usize) {
        todo!("Less than is not yet supported")
    }

    fn leq(&mut self, src: usize, dest: usize) {
        todo!("Less than or equal is not yet supported")
    }

    fn gt(&mut self, src: usize, dest: usize) {
        todo!("Greater than is not yet supported")
    }

    fn geq(&mut self, src: usize, dest: usize) {
        todo!("Greater than or equal is not yet supported")
    }

    fn and(&mut self, src: usize, dest: usize) {
        todo!("Logical and is not yet supported")
    }

    fn or(&mut self, src: usize, dest: usize) {
        todo!("Logical or is not yet supported")
    }

    fn compile(&mut self, statements: &[Statement]) -> Result<(), String> {
        for stmt in statements {
            use crate::parser::Statement as S;
            match stmt {
                S::FunctionDefinition { name, params, body } => {
                    self.function_declaration(&name, &params, body)?
                }
                S::VariableDefinition { name, initializer } => {
                    self.variable_definition(&name, initializer.as_ref())?
                }
                S::Return(_) => todo!(),
                S::Print(_) => todo!(),
                S::Block(block_statements) => self.compile(&block_statements)?,
                S::If {
                    condition,
                    then_branch,
                    else_branch,
                } => self.if_statement(&condition, &then_branch, else_branch.as_deref())?,
                S::While { condition, body } => self.while_statement(&condition, &body)?,
            }
        }
        Ok(())
    }

    fn function_declaration(
        &mut self,
        name: &str,
        params: &[String],
        body: &Statement,
    ) -> Result<(), String> {
        todo!("Function declarations are not yet supported")
    }

    fn variable_definition(
        &mut self,
        name: &str,
        initializer: Option<&Expr>,
    ) -> Result<(), String> {
        let index = self.alloc_var(name);
        if let Some(init) = initializer {
            let expr_index = self.alloc(1);
            self.evaluate_expression(&init, expr_index)?;
            self.move_val(expr_index, index);
            self.dealloc(1);
        }
        Ok(())
    }

    fn if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &Statement,
        else_branch: Option<&Statement>,
    ) -> Result<(), String> {
        todo!("If statements are not yet supported")
    }

    fn while_statement(&mut self, condition: &Expr, body: &Statement) -> Result<(), String> {
        todo!("While statements are not yet supported")
    }

    /// Evaluates an expression and writes the output to `dest`.
    /// The value at `dest` is assumed to be 0.
    fn evaluate_expression(&mut self, expr: &Expr, dest: usize) -> Result<(), String> {
        use crate::parser::BinaryOp as BO;
        use crate::parser::Expr as E;
        match expr {
            E::Unary { op, rhs } => todo!(),
            E::Binary {
                lhs: lhs_expr,
                op,
                rhs: rhs_expr,
            } => {
                let lhs = self.alloc(1);
                let rhs = self.alloc(1);
                self.evaluate_expression(&lhs_expr, lhs)?;
                self.evaluate_expression(&rhs_expr, rhs)?;
                self.dadd(lhs, dest);
                match op {
                    BO::Add => self.dadd(rhs, dest),
                    BO::Sub => self.dsub(rhs, dest),
                    BO::Mul => self.dmul(rhs, dest),
                    BO::Div => self.div(rhs, dest),
                    BO::Mod => self.modulo(rhs, dest),
                    BO::Eq => self.eq(rhs, dest),
                    BO::Neq => self.neq(rhs, dest),
                    BO::Lt => self.lt(rhs, dest),
                    BO::Leq => self.leq(rhs, dest),
                    BO::Gt => self.gt(rhs, dest),
                    BO::Geq => self.geq(rhs, dest),
                    BO::And => self.and(rhs, dest),
                    BO::Or => self.or(rhs, dest),
                }
                self.dealloc(2);
            }
            E::Number(n) => self.set_val(dest, *n),
            E::String(_) => todo!("Strings are not yet supported"),
            E::Identifier(name) => match self.name_table.get(name) {
                Some(index) => self.copy_val(*index, &[dest]),
                None => return Err(format!("Variable {} is not defined", name)),
            },
            E::Call { callee, args } => todo!(),
        }
        Ok(())
    }
}

pub fn compile(src: &str) -> Result<String, String> {
    let tokens = tokenize(src);
    let program = parse(&tokens)?;
    let mut compiler = Compiler::new();
    compiler.compile(&program.statements)?;
    Ok(compiler.output.iter().collect())
}
