use crate::tokenizer::Token;

#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) statements: Vec<Statement>,
}

impl Program {
    fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

#[derive(Debug)]
pub(crate) enum Statement {
    FunctionDefinition {
        name: String,
        params: Vec<String>,
        body: Box<Statement>,
    },
    VariableDefinition {
        name: String,
        initializer: Option<Expr>,
    },
    Assignment {
        name: String,
        value: Expr,
    },
    Return(Option<Expr>),
    Print(Expr),
    Block(Vec<Statement>),
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expr,
        body: Box<Statement>,
    },
}

#[derive(Debug)]
pub(crate) enum Expr {
    Unary {
        op: UnaryOp,
        rhs: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },
    Number(u8),
    String(String),
    Identifier(String),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
}

#[derive(Debug)]
pub(crate) enum UnaryOp {
    Neg,
    Not,
}

impl From<Token> for UnaryOp {
    fn from(token: Token) -> Self {
        use Token as T;
        use UnaryOp as U;
        match token {
            T::Minus => U::Neg,
            T::Not => U::Not,
            _ => panic!("Expected unary operator, found {:?}", token),
        }
    }
}

#[derive(Debug)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
    And,
    Or,
}

impl From<Token> for BinaryOp {
    fn from(token: Token) -> Self {
        use BinaryOp as B;
        use Token as T;
        match token {
            T::Plus => B::Add,
            T::Minus => B::Sub,
            T::Star => B::Mul,
            T::Slash => B::Div,
            T::Percent => B::Mod,
            T::EqualEqual => B::Eq,
            T::NotEqual => B::Neq,
            T::Less => B::Lt,
            T::LessEqual => B::Leq,
            T::Greater => B::Gt,
            T::GreaterEqual => B::Geq,
            T::And => B::And,
            T::Or => B::Or,
            _ => panic!("Expected binary operator, found {:?}", token),
        }
    }
}

impl BinaryOp {
    fn precedence(&self) -> u8 {
        use BinaryOp as B;
        match self {
            B::Or => 1,
            B::And => 2,
            B::Eq | B::Neq => 3,
            B::Lt | B::Leq | B::Gt | B::Geq => 4,
            B::Add | B::Sub => 5,
            B::Mul | B::Div | B::Mod => 6,
        }
    }
}

pub(crate) fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut parser = Parser::new(tokens);
    parser.program()
}

struct Parser {
    tokens: Vec<Token>,
    errors: Vec<String>,
    current: usize,
}

impl Parser {
    fn new(tokens: &[Token]) -> Self {
        let tokens = Vec::from(tokens)
            .iter()
            .filter(|t| !t.is_ignorable())
            .cloned()
            .collect::<Vec<_>>();
        Self {
            tokens,
            errors: Vec::new(),
            current: 0,
        }
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn consume(&mut self) -> Token {
        let token = self.peek();
        self.current += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Token::Eof || self.current >= self.tokens.len()
    }

    fn expect(&mut self, token: Token) -> Result<Token, String> {
        if self.peek() == token {
            Ok(self.consume())
        } else {
            Err(format!("Expected {:?}, found {:?}", token, self.peek()))
        }
    }

    fn program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => errors.push(error),
            }
        }
        println!("Errors: {:?}", errors);
        if errors.is_empty() {
            Ok(Program::new(statements))
        } else {
            Err(errors.join("\n"))
        }
    }

    // Parsing statements

    fn statement(&mut self) -> Result<Statement, String> {
        use Token as T;
        match self.peek() {
            T::Let => self.variable_definition(),
            T::Print => self.print(),
            T::Return => self.return_statement(),
            T::LeftBrace => self.block(),
            T::If => self.if_statement(),
            T::While => self.while_statement(),
            T::Identifier(_) => self.assignment(),
            T::Function => self.function_declaration(),
            _ => Err(format!("Expected statement, found {:?}", self.consume())),
        }
    }

    fn function_declaration(&mut self) -> Result<Statement, String> {
        self.expect(Token::Function)?; // fn
        let name = if let Token::Identifier(name) = self.consume() {
            name
        } else {
            return Err("Expected function name".to_string());
        };
        self.expect(Token::LeftParen)?; // (
        let mut params = Vec::new();
        if self.peek() != Token::RightParen {
            loop {
                if let Token::Identifier(param) = self.consume() {
                    params.push(param);
                } else {
                    return Err("Expected parameter name".to_string());
                }
                if self.peek() == Token::RightParen {
                    break;
                }
                self.expect(Token::Comma)?; // ,
            }
        }
        self.expect(Token::RightParen)?; // )
        self.expect(Token::LeftBrace)?; // {
        let body = self.block()?;
        Ok(Statement::FunctionDefinition {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn variable_definition(&mut self) -> Result<Statement, String> {
        self.expect(Token::Let)?; // let
        let name = if let Token::Identifier(name) = self.consume() {
            name
        } else {
            return Err("Expected variable name".to_string());
        };
        let initializer = if self.peek() == Token::Equal {
            self.consume(); // =
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?; // ;
        Ok(Statement::VariableDefinition { name, initializer })
    }

    fn print(&mut self) -> Result<Statement, String> {
        self.expect(Token::Print)?; // print
        self.expect(Token::LeftParen)?; // (
        let expr = self.expression()?;
        self.expect(Token::RightParen)?; // )
        self.expect(Token::Semicolon)?; // ;
        Ok(Statement::Print(expr))
    }

    fn return_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::Return)?; // return
        let expr = if self.peek() != Token::Semicolon {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?; // ;
        Ok(Statement::Return(expr))
    }

    fn block(&mut self) -> Result<Statement, String> {
        let mut statements = Vec::new();
        self.expect(Token::LeftBrace)?; // {
        while self.peek() != Token::RightBrace && !self.is_at_end() {
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => self.errors.push(error),
            }
        }
        self.expect(Token::RightBrace)?; // }
        Ok(Statement::Block(statements))
    }

    fn if_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::If)?; // if
        let condition = self.expression()?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.peek() == Token::Else {
            self.consume(); // else
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::While)?; // while
        let condition = self.expression()?;
        let body = Box::new(self.statement()?);
        Ok(Statement::While { condition, body })
    }

    fn assignment(&mut self) -> Result<Statement, String> {
        let name = if let Token::Identifier(name) = self.consume() {
            name
        } else {
            return Err("Expected variable name".to_string());
        };
        self.expect(Token::Equal)?; // =
        let value = self.expression()?;
        self.expect(Token::Semicolon)?; // ;
        Ok(Statement::Assignment { name, value })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        use Token as T;
        use UnaryOp as U;
        let expr = match self.consume() {
            T::Number(n) => Expr::Number(n),
            T::String(s) => Expr::String(s),
            T::Identifier(name) => match self.peek() {
                T::LeftParen => {
                    self.expect(T::LeftParen)?; // (
                    let mut args = Vec::new();
                    if self.peek() != T::RightParen {
                        loop {
                            args.push(self.expression()?);
                            if self.peek() == T::RightParen {
                                break;
                            }
                            self.expect(T::Comma)?; // ,
                        }
                    }
                    self.expect(T::RightParen)?; // )
                    Expr::Call {
                        callee: Box::new(Expr::Identifier(name)),
                        args,
                    }
                }
                _ => Expr::Identifier(name),
            },
            T::LeftParen => {
                let expr = self.expression()?;
                self.expect(T::RightParen)?; // )
                expr
            }
            T::Minus => Expr::Unary {
                op: U::Neg,
                rhs: Box::new(self.expression()?),
            },
            T::Not => Expr::Unary {
                op: U::Not,
                rhs: Box::new(self.expression()?),
            },
            _ => return Err("Expected expression".to_string()),
        };
        // Check for binary expressions
        let expr = if self.peek().is_binary_op() {
            let mut lhs = expr;
            let mut op: BinaryOp = self.consume().into();
            let mut rhs = self.expression()?;
            while self.peek().is_binary_op()
                && op.precedence() < BinaryOp::from(self.peek()).precedence()
            {
                let next_op = self.consume().into();
                let next_rhs = self.expression()?;
                lhs = Expr::Binary {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                };
                op = next_op;
                rhs = next_rhs;
            }
            Expr::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        } else {
            expr
        };
        Ok(expr)
    }
}
