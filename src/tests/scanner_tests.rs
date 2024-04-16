#![cfg(test)]

use crate::scanner::Scanner;
use crate::token::{ TokenType, Token };

#[test]
fn test_scan_tokens() {
    let source = "(){}+-=!=!*/
        if else ident return
            /* multiline comment */
        ".to_string();
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    assert!(tokens.is_some());
    let tokens = tokens.unwrap();

    let tests = vec![
        Token::new(TokenType::LeftParen, "(".to_string(), 1),
        Token::new(TokenType::RightParen, ")".to_string(), 1),
        Token::new(TokenType::LeftBrace, "{".to_string(), 1),
        Token::new(TokenType::RightBrace, "}".to_string(), 1),
        Token::new(TokenType::Plus, "+".to_string(), 1),
        Token::new(TokenType::Minus, "-".to_string(), 1),
        Token::new(TokenType::Equal, "=".to_string(), 1),
        Token::new(TokenType::BangEqual, "!=".to_string(), 1),
        Token::new(TokenType::Bang, "!".to_string(), 1),
        Token::new(TokenType::Star, "*".to_string(), 1),
        Token::new(TokenType::Slash, "/".to_string(), 1),
        Token::new(TokenType::If, "if".to_string(), 2),
        Token::new(TokenType::Else, "else".to_string(), 2),
        Token::new(TokenType::Identifier, "ident".to_string(), 2),
        Token::new(TokenType::EOF, "\0".to_string(), 4),
    ];

    for (i, tt) in tests.iter().enumerate() {
        if &tokens[i] != tt {
            panic!("test[{}] failed. Expected {:?}. Got {:?}",
                i, tokens[i], tt);
        }
    }
}
