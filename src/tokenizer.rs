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
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(u8),
    Boolean(bool),

    // Keywords.
    If,
    Else,
    ElseIf,
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

pub fn tokenize(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(token) = next_token(&mut chars) {
        tokens.push(token);
    }
    tokens.push(Token::Eof);
    tokens
}

fn next_token(chars: &mut Peekable<Chars<'_>>) -> Option<Token> {
    if let Some(c) = chars.next() {
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
            '!' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::BangEqual
                } else {
                    Token::Bang
                }
            }
            '=' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::EqualEqual
                } else {
                    Token::Equal
                }
            }
            '<' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '/' => {
                if chars.peek() == Some(&'/') {
                    chars.next();
                    Token::Comment(read_comment(chars))
                } else {
                    Token::Slash
                }
            }
            '"' => Token::String(read_string(chars)),
            n if n >= '0' && n <= '9' => Token::Number(read_number(chars, n)),
            n if n >= 'a' && n <= 'z' || n >= 'A' && n <= 'Z' => read_identifier(chars, n),
            ' ' | '\n' | '\t' | '\r' => return next_token(chars),
            _ => Token::Error(format!("Unexpected character: {}", c)),
        };
        Some(token)
    } else {
        None
    }
}

fn read_number(chars: &mut Peekable<Chars<'_>>, first_num: char) -> u8 {
    let mut number = String::new();
    number.push(first_num);
    while let Some(c) = chars.peek() {
        if c.is_digit(10) {
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
            _ => string.push(c),
        }
    }
    string
}

fn read_comment(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut comment = String::new();
    while let Some(c) = chars.next() {
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
        "elseif" => Token::ElseIf,
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
