use std::collections::HashMap;

use crate::{token::{
    Token, TokenType
}, Lox};

pub struct Scanner {
    start: usize,
    current: usize,
    line: usize,
    source: String,
    tokens: Vec<Token>,
    keywords: HashMap<String, TokenType>
}

impl Scanner {
    pub fn new(source: String) -> Self {
        let mut keywords: HashMap<String, TokenType> = HashMap::new();
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("var".to_string(), TokenType::Var);
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("print".to_string(), TokenType::Print);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("nil".to_string(), TokenType::Nil);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("else".to_string(), TokenType::Else);

        Scanner {
            source, start: 0,
            current: 0, line: 1,
            tokens: Vec::new(),
            keywords
        }
    }

    pub fn scan_tokens(&mut self) -> Option<&Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            if let Err(_) = self.scan_token() {
                return None;
            }
        }

        self.tokens.push(
            Token::new(TokenType::EOF, "\0".to_string(), self.line)
        );

        return Some(&self.tokens);
    }

    fn scan_token(&mut self) -> Result<(), ()> {
        let c = self.advance();
        match c {
            '(' => {
                self.add_token(TokenType::LeftParen, "(".to_string(), self.line)
            },
            ')' => {
                self.add_token(TokenType::RightParen, ")".to_string(), self.line)
            },
            '{' => {
                self.add_token(TokenType::LeftBrace, "{".to_string(), self.line)
            },
            '}' => {
                self.add_token(TokenType::RightBrace, "}".to_string(), self.line)
            },
            ',' => {
                self.add_token(TokenType::Comma, ",".to_string(), self.line)
            },
            '.' => {
                self.add_token(TokenType::Dot, ".".to_string(), self.line)
            },
            '-' => {
                self.add_token(TokenType::Minus, "-".to_string(), self.line)
            },
            '+' => {
                self.add_token(TokenType::Plus, "+".to_string(), self.line)
            },
            ';' => {
                self.add_token(TokenType::Semicolon, ";".to_string(), self.line)
            },
            '*' => {
                self.add_token(TokenType::Star, "*".to_string(), self.line)
            },
            '?' => {
                self.add_token(TokenType::Qmark, "?".to_string(), self.line)
            },
            ':' => {
                self.add_token(TokenType::Colon, ":".to_string(), self.line)
            },
            '/' => {
                if self.match_lexeme('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(())
                } else  if self.match_lexeme('*') {
                    while self.peek() != '*' && self.peek_next() != '/' && !self.is_at_end() {
                        // if self.peek() == '\n' { self.line += 1; }
                        self.advance();
                    }
                    self.advance();
                    self.advance();
                    Ok(())
                } else {
                    self.add_token(TokenType::Slash, "/".to_string(), self.line)
                }
            },
            '=' => {
                if self.match_lexeme('=') {
                    self.add_token(TokenType::EqualEqual, "==".to_string(), self.line)
                } else {
                    self.add_token(TokenType::Equal, "=".to_string(), self.line)
                }
            },
            '!' => {
                if self.match_lexeme('=') {
                    self.add_token(TokenType::BangEqual, "!=".to_string(), self.line)
                } else {
                    self.add_token(TokenType::Bang, "!".to_string(), self.line)
                }
            },
            '<' => {
                if self.match_lexeme('=') {
                    self.add_token(TokenType::LessEqual, "<=".to_string(), self.line)
                } else {
                    self.add_token(TokenType::Less, "<".to_string(), self.line)
                }
            },
            '>' => {
                if self.match_lexeme('=') {
                    self.add_token(TokenType::GreaterEqual, ">=".to_string(), self.line)
                } else {
                    self.add_token(TokenType::Greater, ">".to_string(), self.line)
                }
            },
            '\0' => {
                self.add_token(TokenType::EOF, "\0".to_string(), self.line)
            },
            ' ' | '\t' | '\r' => { Ok(()) },
            '`' => { self.string('`') },
            '"' => { self.string('"') },
            '\'' => { self.string('\'') },
            '\n' => { self.line += 1; Ok(()) },
            _ => {
                if Self::is_digit(c) {
                    self.number()
                } else if Self::is_alpha(c) {
                    self.identifier()
                }else {
                    Lox::error(self.line, "Unexpected Character.".to_string());
                    return Err(());
                }
            }
        }
    }

    fn identifier(&mut self) -> Result<(), ()> {
        while Self::is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let lexeme = self.source[self.start..self.current].to_string();
        let token_type = self.keywords.get(&lexeme)
            .unwrap_or(&TokenType::Identifier);

        self.add_token(*token_type, lexeme, self.line)
    }

    fn number(&mut self) -> Result<(), ()> {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            self.advance();
            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        let lexeme = self.source[self.start..self.current].to_string();

        self.add_token(TokenType::Number, lexeme, self.line)
    }

    fn string(&mut self, ch: char) -> Result<(), ()> {
        while self.peek() != ch && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }

        if self.is_at_end() {
            Lox::error(self.line, "Unterminated string.".to_string());
            return Err(());
        }

        self.advance();
        let lexeme = self.source[self.start+1..self.current-1].to_string();

        self.add_token(TokenType::String, lexeme, self.line)
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source
            .chars()
            .nth(self.current - 1)
            .unwrap_or('\0')
    }

    fn add_token(
        &mut self, token_type: TokenType,
        lexeme: String, line: usize
    ) -> Result<(), ()> {
        let token = Token::new(token_type, lexeme, line);
        self.tokens.push(token);
        Ok(())
    }

    fn match_lexeme(&mut self, ch: char) -> bool {
        if self.is_at_end() { return false; }
        if self.source.chars().nth(self.current).unwrap() != ch {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    fn is_alpha(ch: char) -> bool {
        ch.is_ascii_alphabetic()
    }

    fn is_alpha_numeric(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn peek(&self) -> char {
        self.source
            .chars()
            .nth(self.current)
            .unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.source
            .chars()
            .nth(self.current + 1)
            .unwrap_or('\0')
    }
}
