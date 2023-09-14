use crate::tokenizer::Token;

pub enum Node {
    Program(Vec<Node>),

    // Top-level
    FunctionDefinition {
        name: String,
        params: Vec<String>,
        body: Box<Node>,
    },

    // Statements
    VariableDefinition {
        name: String,
        initializer: Option<Box<Node>>,
    },
    Return(Option<Box<Node>>),
    Print(Box<Node>),

    // Expressions
    Expr(Box<Node>),
    UnaryExpr {
        op: UnaryOp,
        rhs: Box<Node>,
    },
    BinaryExpr {
        lhs: Box<Node>,
        op: BinaryOp,
        rhs: Box<Node>,
    },
    Number(u8),
    String(String),
    Identifier(String),

    // Blocks and statements
    Block(Vec<Node>),
    If {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    While {
        condition: Box<Node>,
        body: Box<Node>,
    },

    // Functions
    Call {
        callee: Box<Node>,
        args: Vec<Node>,
    },
}

pub enum UnaryOp {
    Neg,
    Not,
}

pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
}

pub fn parse(tokens: &[Token]) -> Result<Node, String> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser<'a> {
    tokens: &'a [Token],
    errors: Vec<String>,
    current: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &[Token]) -> Self {
        Self {
            tokens,
            errors: Vec::new(),
            current: 0,
        }
    }

    fn peek(&self) -> Token {
        self.tokens[self.current]
    }

    fn consume(&mut self) -> Token {
        let token = self.peek();
        self.current += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Token::Eof
    }

    fn parse(&mut self) -> Result<Node, String> {
        self.program()
    }

    fn expect(&mut self, token: Token) -> Option<Token> {
        if self.peek() == token {
            Some(self.consume())
        } else {
            self.errors
                .push(format!("Expected {:?}, found {:?}", token, self.peek()));
            None
        }
    }

    fn program(&mut self) -> Result<Node, String> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            match self.function_declaration() {
                Ok(statement) => statements.push(statement),
                Err(error) => errors.push(error),
            }
        }
        Ok(Node::Program(statements))
    }

    fn function_declaration(&mut self) -> Result<Node, String> {
        self.expect(Token::Function); // fn
        let name = if let Token::Identifier(name) = self.consume() {
            name
        } else {
            return Err("Expected function name".to_string());
        };
        self.expect(Token::LeftParen); // (
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
                self.expect(Token::Comma); // ,
            }
        }
        self.expect(Token::RightParen); // )
        self.expect(Token::LeftBrace); // {
        let body = self.block()?;
        Ok(Node::FunctionDefinition {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn block(&mut self) -> Result<Node, String> {
        let mut statements = Vec::new();
        while self.peek() != Token::RightBrace && !self.is_at_end() {
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => self.errors.push(error),
            }
        }
        self.expect(Token::RightBrace); // }
        Ok(Node::Block(statements))
    }

    fn statement(&mut self) -> Result<Node, String> {
        match self.peek() {
            Token::Let => self.variable_definition(),
            Token::Print => self.print(),
            Token::Return => self.return_statement(),
            Token::LeftBrace => self.block(),
            Token::If => self.if_statement(),
            Token::While => self.while_statement(),
            _ => self.expression_statement(),
        }
    }

    fn variable_definition(&mut self) -> Result<Node, String> {
        self.expect(Token::Let); // let
        let name = if let Token::Identifier(name) = self.consume() {
            name
        } else {
            return Err("Expected variable name".to_string());
        };
        let initializer = if self.peek() == Token::Equal {
            self.consume(); // =
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.expect(Token::Semicolon); // ;
        Ok(Node::VariableDefinition { name, initializer })
    }
}
