use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Less,
    Percent,
    Greater,
    Equal,
    Not,
    And,
    Or,

    // Two-character tokens.
    EqualEqual,
    LessEqual,
    GreaterEqual,
    NotEqual,
    AndAnd,
    OrOr,

    // Literals.
    Identifier(String),
    String(String),
    Number(u8),
    Boolean(bool),

    // Keywords.
    If,
    Else,
    For,
    While,
    Return,
    Function,
    Let,
    Print,

    // Misc
    Eof,
    Error(String),
    Comment(String),
}

impl Token {
    pub(crate) fn is_ignorable(&self) -> bool {
        matches!(self, Token::Comment(_))
    }

    pub(crate) fn is_binary_op(&self) -> bool {
        matches!(
            self,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::EqualEqual
                | Token::NotEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual
                | Token::And
                | Token::Or
        )
    }
}

pub struct TokenStream<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.chars.next() {
            let token = match c {
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                ',' => Token::Comma,
                '.' => Token::Dot,
                '-' => Token::Minus,
                '+' => Token::Plus,
                ';' => Token::Semicolon,
                '*' => Token::Star,
                '%' => Token::Percent,
                '!' => {
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        Token::NotEqual
                    } else {
                        Token::Not
                    }
                }
                '=' => {
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '<' => {
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '/' => {
                    if self.chars.peek() == Some(&'/') {
                        self.chars.next();
                        Token::Comment(read_comment(&mut self.chars))
                    } else {
                        Token::Slash
                    }
                }
                '&' => {
                    if self.chars.peek() == Some(&'&') {
                        self.chars.next();
                        Token::AndAnd
                    } else {
                        Token::And
                    }
                }
                '|' => {
                    if self.chars.peek() == Some(&'|') {
                        self.chars.next();
                        Token::OrOr
                    } else {
                        Token::Or
                    }
                }
                '"' => Token::String(read_string(&mut self.chars)),
                ' ' | '\n' | '\t' | '\r' => return self.next(),
                n if n.is_ascii_digit() => Token::Number(read_number(&mut self.chars, n)),
                n if n.is_ascii() => read_identifier(&mut self.chars, n),
                _ => Token::Error(format!("Unexpected character: {}", c)),
            };
            Some(token)
        } else {
            None
        }
    }
}

pub fn tokenize(src: &str) -> TokenStream<'_> {
    TokenStream {
        chars: src.chars().peekable(),
    }
}

fn read_number(chars: &mut Peekable<Chars<'_>>, first_num: char) -> u8 {
    let mut number = String::new();
    number.push(first_num);
    while let Some(c) = chars.peek() {
        if c.is_ascii_digit() {
            number.push(*c);
            chars.next();
        } else {
            break;
        }
    }
    number.parse().unwrap()
}

fn read_string(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut string = String::new();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => {
                if let Some(c) = chars.next() {
                    match c {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        _ => string.push(c),
                    }
                }
            }
            _ => string.push(c),
        }
    }
    string
}

fn read_comment(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut comment = String::new();
    for c in chars.by_ref() {
        match c {
            '\n' => break,
            _ => comment.push(c),
        }
    }
    comment
}

fn read_identifier(chars: &mut Peekable<Chars<'_>>, first_char: char) -> Token {
    let mut identifier = String::new();
    identifier.push(first_char);
    while let Some(c) = chars.peek() {
        if c.is_alphanumeric() {
            identifier.push(*c);
            chars.next();
        } else {
            break;
        }
    }
    match identifier.as_str() {
        "if" => Token::If,
        "else" => Token::Else,
        "for" => Token::For,
        "while" => Token::While,
        "return" => Token::Return,
        "fn" => Token::Function,
        "let" => Token::Let,
        "print" => Token::Print,
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),
        _ => Token::Identifier(identifier),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsStr,
        fs::{read_dir, read_to_string, File},
        io::{self, Write},
    };

    use super::*;

    #[test]
    fn test_tokenizer() -> Result<(), io::Error> {
        let dir = read_dir("tests/tokenizer")?.filter(|e| {
            if let Ok(e) = e {
                e.path().extension() == Some(OsStr::new("in"))
            } else {
                false
            }
        });
        for path in dir {
            let path = path?;
            let input = read_to_string(path.path())?;
            let tokens = tokenize(&input);
            let mut out = String::new();
            for token in tokens {
                out.push_str(&format!("{:?}\n", token));
            }
            let expected_path = path.path().with_extension("out");
            println!("Output path: {:?}", path);
            println!("Expected path: {:?}", expected_path);
            if let Ok(expected) = read_to_string(&expected_path) {
                assert_eq!(out, expected);
            } else {
                let mut out_file = File::create(&expected_path)?;
                println!("Expected file not found, creating it...");
                write!(out_file, "{}", out)?;
            };
        }
        Ok(())
    }
}
